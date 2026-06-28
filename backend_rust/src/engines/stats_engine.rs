use chrono::{NaiveDate, Utc, DateTime, Datelike};
use sqlx::SqlitePool;
use crate::models::{Asset, Transaction, HistoricalPrice};
use crate::services::currency_service::CurrencyService;
use anyhow::Result;
use std::collections::HashMap;

pub struct StatsEngine;

impl StatsEngine {
    pub async fn sync_historical_prices(_pool: &SqlitePool, _symbols: &[String], _start_date: NaiveDate) -> Result<()> {
        // Implementation can be added here later using reqwest
        Ok(())
    }

    pub async fn get_historical_price_matrix(
        pool: &SqlitePool,
        symbols: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(NaiveDate, String, f64)>> {
        let mut all_prices = Vec::new();
        for symbol in symbols {
            let prices = sqlx::query_as::<_, HistoricalPrice>(
                "SELECT * FROM historical_prices WHERE symbol = ? AND date >= ? AND date <= ?"
            )
            .bind(symbol)
            .bind(start_date)
            .bind(end_date)
            .fetch_all(pool)
            .await?;
            for p in prices {
                all_prices.push((p.date, p.symbol, p.close_price));
            }
        }
        Ok(all_prices)
    }

    pub async fn calculate_portfolio_performance(
        pool: &SqlitePool,
        assets: &[Asset],
        transactions: &[Transaction],
        base_currency: &str,
        currency_service: &CurrencyService,
    ) -> Result<serde_json::Value> {
        if assets.is_empty() || transactions.is_empty() {
            return Ok(serde_json::json!({
                "history": [],
                "correlation_matrix": {},
                "metrics": {
                    "volatility": 0.0,
                    "sharpe_ratio": 0.0,
                    "beta": 1.0,
                    "portfolio_value": 0.0,
                    "unrealized_pnl": 0.0,
                    "realized_pnl": 0.0,
                    "beta_adjusted_exposure": 0.0
                }
            }));
        }

        let tx_dates: Vec<NaiveDate> = transactions.iter().map(|tx| tx.date.date_naive()).collect();
        let start_date = *tx_dates.iter().min().unwrap();
        let end_date = Utc::now().date_naive();

        let symbols: Vec<String> = assets.iter().map(|a| a.symbol.clone()).collect();
        let prices_data = Self::get_historical_price_matrix(pool, &symbols, start_date, end_date).await?;
        
        let mut price_map: HashMap<(NaiveDate, String), f64> = HashMap::new();
        for (date, symbol, price) in prices_data {
            price_map.insert((date, symbol), price);
        }

        let mut dates = Vec::new();
        let mut curr = start_date;
        while curr <= end_date {
            dates.push(curr);
            curr = curr.succ_opt().unwrap();
        }

        let mut history = Vec::new();
        let mut portfolio_values = Vec::new();
        let mut daily_returns = Vec::new();
        let mut twr_cumulative = Vec::new();
        let mut twr_acc = 1.0;

        let mut asset_qtys: HashMap<String, f64> = HashMap::new();
        let mut sorted_txs = transactions.to_vec();
        sorted_txs.sort_by_key(|tx| tx.date);

        let mut tx_idx = 0;
        for &date in &dates {
            while tx_idx < sorted_txs.len() && sorted_txs[tx_idx].date.date_naive() <= date {
                let tx = &sorted_txs[tx_idx];
                let symbol = assets.iter().find(|a| a.id == tx.asset_id).map(|a| a.symbol.as_str()).unwrap_or("");
                let entry = asset_qtys.entry(symbol.to_string()).or_insert(0.0);
                if tx.r#type.to_uppercase() == "BUY" {
                    *entry += tx.quantity;
                } else if tx.r#type.to_uppercase() == "SELL" {
                    *entry = (*entry - tx.quantity).max(0.0);
                }
                tx_idx += 1;
            }

            let mut daily_val = 0.0;
            for asset in assets {
                let qty = asset_qtys.get(&asset.symbol).cloned().unwrap_or(0.0);
                let price = price_map.get(&(date, asset.symbol.clone())).cloned().unwrap_or(0.0);
                
                let mut final_price = price;
                if asset.currency != base_currency {
                    // Simple date handling for currency service
                    let date_utc = DateTime::<Utc>::from_utc(NaiveDate::from_ymd_opt(date.year(), date.month(), date.day()).unwrap().and_hms_opt(0,0,0).unwrap(), Utc);
                    let rate = currency_service.get_rate(&asset.currency, base_currency, date_utc).await.unwrap_or(1.0);
                    final_price *= rate;
                }
                daily_val += qty * final_price;
            }
            
            portfolio_values.push(daily_val);
            
            let daily_ret = if portfolio_values.len() > 1 {
                let prev_val = portfolio_values[portfolio_values.len()-2];
                if prev_val > 0.0 { (daily_val - prev_val) / prev_val } else { 0.0 }
            } else {
                0.0
            };
            
            twr_acc *= 1.0 + daily_ret;
            daily_returns.push(daily_ret);
            twr_cumulative.push(twr_acc - 1.0);

            history.push(serde_json::json!({
                "date": date.to_string(),
                "value": daily_val,
                "daily_return": daily_ret,
                "twr": twr_acc - 1.0
            }));
        }

        let final_val = portfolio_values.last().cloned().unwrap_or(0.0);

        Ok(serde_json::json!({
            "history": history,
            "correlation_matrix": {},
            "metrics": {
                "volatility": 0.0, // Would require std dev of daily_returns
                "sharpe_ratio": 0.0,
                "beta": 1.0,
                "portfolio_value": final_val,
                "beta_adjusted_exposure": final_val,
            }
        }))
    }
}

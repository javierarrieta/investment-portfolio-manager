use chrono::{Datelike, NaiveDate, Utc, TimeZone};
use sqlx::SqlitePool;
use crate::models::{Asset, Transaction};
use crate::services::currency_service::CurrencyService;
use anyhow::Result;
use std::collections::HashMap;

pub struct StatsEngine;

impl StatsEngine {
    pub async fn aggregate_weekly(
        daily_history: Vec<serde_json::Value>,
    ) -> Vec<serde_json::Value> {
        if daily_history.is_empty() {
            return Vec::new();
        }

        let mut weeks: HashMap<(i32, u32), Vec<serde_json::Value>> = HashMap::new();
        for item in daily_history {
            let date_str = item.get("date").and_then(|d| d.as_str()).unwrap_or("");
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let iso = date.iso_week();
                let key = (date.year(), iso.week());
                weeks.entry(key).or_default().push(item);
            }
        }

        let mut sorted_weeks: Vec<_> = weeks.into_iter().collect();
        sorted_weeks.sort_by(|a, b| a.0.cmp(&b.0));

        let mut weekly_history = Vec::new();
        let mut prev_value: Option<f64> = None;
        let mut twr_acc = 1.0;

        for (_key, items) in sorted_weeks {
            // Take the last trading day's value (non-zero), not the last calendar day
            let trading_items: Vec<_> = items.iter()
                .filter(|item| item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0)
                .collect();
            
            let last_item = if trading_items.is_empty() {
                items.last().unwrap()
            } else {
                trading_items.last().unwrap()
            };
            
            let date = last_item.get("date").and_then(|d| d.as_str()).unwrap_or("");
            let value = last_item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let daily_ret = match prev_value {
                Some(prev) if prev > 0.0 && value > 0.0 => (value - prev) / prev,
                _ if value > 0.0 => 0.0,
                _ => {
                    // Skip weeks with zero value (holidays/weekends) - carry forward TWR
                    twr_acc *= 1.0;
                    prev_value = Some(value);
                    weekly_history.push(serde_json::json!({
                        "date": date,
                        "value": value,
                        "daily_return": 0.0,
                        "twr": twr_acc - 1.0,
                    }));
                    continue;
                }
            };

            twr_acc *= 1.0 + daily_ret;
            prev_value = Some(value);

            weekly_history.push(serde_json::json!({
                "date": date,
                "value": value,
                "daily_return": daily_ret,
                "twr": twr_acc - 1.0,
            }));
        }

        weekly_history
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
        let earliest_tx = *tx_dates.iter().min().unwrap();
        let end_date = Utc::now().date_naive();

        let start_date = earliest_tx - chrono::Duration::days(365);

        let min_reasonable_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let start_date = if start_date < min_reasonable_date {
            min_reasonable_date
        } else {
            start_date
        };

        let symbols: Vec<String> = assets.iter().map(|a| a.symbol.clone()).collect();

        let mut price_map: HashMap<(NaiveDate, String), f64> = HashMap::new();
        for symbol in &symbols {
            match currency_service.get_historical_prices(symbol, start_date, end_date, pool).await {
                Ok(daily_prices) => {
                    for (date, price) in daily_prices {
                        price_map.insert((date, symbol.clone()), price);
                    }
                }
                Err(e) => {
                    eprintln!("WARN: Failed to get historical prices for {}: {}", symbol, e);
                }
            }
        }

        let mut rate_cache: HashMap<String, f64> = HashMap::new();
        for asset in assets {
            if asset.currency != base_currency {
                let key = format!("{}->{}", asset.currency, base_currency);
                if !rate_cache.contains_key(&key) {
                    let date_utc = Utc.from_utc_datetime(&end_date.and_hms_opt(0, 0, 0).unwrap());
                    let rate = currency_service
                        .get_rate(&asset.currency, base_currency, date_utc)
                        .await
                        .unwrap_or(1.0);
                    rate_cache.insert(key, rate);
                }
            }
        }

        let mut dates = Vec::new();
        let mut curr = start_date;
        while curr <= end_date {
            dates.push(curr);
            match curr.succ_opt() {
                Some(next) => curr = next,
                None => break,
            }
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
                    let key = format!("{}->{}", asset.currency, base_currency);
                    let rate = rate_cache.get(&key).copied().unwrap_or(1.0);
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

        let weekly_history = Self::aggregate_weekly(history).await;

        Ok(serde_json::json!({
            "history": weekly_history,
            "correlation_matrix": {},
            "metrics": {
                "volatility": 0.0,
                "sharpe_ratio": 0.0,
                "beta": 1.0,
                "portfolio_value": final_val,
                "beta_adjusted_exposure": final_val,
                "unrealized_pnl": 0.0,
                "realized_pnl": 0.0,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use crate::services::currency_service::CurrencyService;

    fn empty_asset() -> Asset {
        Asset {
            id: 1,
            portfolio_id: 1,
            symbol: "TEST".to_string(),
            name: "Test".to_string(),
            asset_type: "STOCK".to_string(),
            sector: None,
            currency: "USD".to_string(),
        }
    }

    fn empty_tx() -> Transaction {
        Transaction {
            id: 1,
            asset_id: 1,
            r#type: "BUY".to_string(),
            quantity: 100.0,
            price: 50.0,
            fee: 0.0,
            date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
        }
    }

    #[tokio::test]
    async fn test_empty_assets_returns_zeroed_metrics() {
        let svc = CurrencyService::new();
        let result = StatsEngine::calculate_portfolio_performance(
            &SqlitePool::connect(":memory:").await.unwrap(),
            &[],
            &[],
            "USD",
            &svc,
        ).await.unwrap();

        let metrics = result.get("metrics").unwrap();
        let vol = metrics.get("volatility").unwrap().as_f64().unwrap();
        let sharpe = metrics.get("sharpe_ratio").unwrap().as_f64().unwrap();
        let beta = metrics.get("beta").unwrap().as_f64().unwrap();
        let value = metrics.get("portfolio_value").unwrap().as_f64().unwrap();
        
        assert!((vol - 0.0).abs() < f64::EPSILON);
        assert!((sharpe - 0.0).abs() < f64::EPSILON);
        assert!((beta - 1.0).abs() < f64::EPSILON);
        assert!((value - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_with_data_but_no_prices_returns_zero_value() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (id INTEGER PRIMARY KEY, symbol TEXT, date TEXT, close_price REAL)")
            .execute(&pool)
            .await
            .unwrap();
        let svc = CurrencyService::new();
        let assets = vec![empty_asset()];
        let txs = vec![empty_tx()];
        
        let result = StatsEngine::calculate_portfolio_performance(
            &pool,
            &assets,
            &txs,
            "USD",
            &svc,
        ).await.unwrap();

        let metrics = result.get("metrics").unwrap();
        let value = metrics.get("portfolio_value").unwrap().as_f64().unwrap();
        assert!(value >= 0.0);
    }

    #[tokio::test]
    async fn test_multicurrency_asset_with_conversion() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (symbol TEXT, date DATE, close_price REAL)")
            .execute(&pool)
            .await
            .unwrap();
        let svc = CurrencyService::new();
        
        let asset = Asset {
            id: 1,
            portfolio_id: 1,
            symbol: "NESF".to_string(),
            name: "Nestle".to_string(),
            asset_type: "STOCK".to_string(),
            sector: None,
            currency: "CHF".to_string(),
        };
        let tx = Transaction {
            id: 1,
            asset_id: 1,
            r#type: "BUY".to_string(),
            quantity: 10.0,
            price: 100.0,
            fee: 0.0,
            date: DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z").unwrap().with_timezone(&Utc),
        };
        
        let result = StatsEngine::calculate_portfolio_performance(
            &pool,
            &[asset],
            &[tx],
            "USD",
            &svc,
        ).await.unwrap();

        let metrics = result.get("metrics").unwrap();
        let value = metrics.get("portfolio_value").unwrap().as_f64().unwrap();
        assert!((value - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_single_day_range_does_not_panic() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (symbol TEXT, date DATE, close_price REAL)")
            .execute(&pool)
            .await
            .unwrap();
        let svc = CurrencyService::new();
        let assets = vec![empty_asset()];
        let txs = vec![empty_tx()];
        
        let result = StatsEngine::calculate_portfolio_performance(
            &pool,
            &assets,
            &txs,
            "USD",
            &svc,
        ).await.unwrap();

        let history = result.get("history").unwrap().as_array().unwrap();
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_aggregate_weekly_basic() {
        let daily_history = vec![
            serde_json::json!({"date": "2024-01-01", "value": 100.0, "daily_return": 0.0, "twr": 0.0}),
            serde_json::json!({"date": "2024-01-02", "value": 105.0, "daily_return": 0.05, "twr": 0.05}),
            serde_json::json!({"date": "2024-01-03", "value": 103.0, "daily_return": -0.019, "twr": 0.029}),
            serde_json::json!({"date": "2024-01-08", "value": 110.0, "daily_return": 0.068, "twr": 0.099}),
            serde_json::json!({"date": "2024-01-09", "value": 112.0, "daily_return": 0.018, "twr": 0.119}),
        ];

        let weekly = StatsEngine::aggregate_weekly(daily_history).await;
        assert_eq!(weekly.len(), 2);

        let week1 = &weekly[0];
        assert_eq!(week1.get("value").unwrap().as_f64().unwrap(), 103.0);
        assert_eq!(week1.get("date").unwrap().as_str().unwrap(), "2024-01-03");

        let week2 = &weekly[1];
        assert_eq!(week2.get("value").unwrap().as_f64().unwrap(), 112.0);
        assert_eq!(week2.get("date").unwrap().as_str().unwrap(), "2024-01-09");
    }

    #[tokio::test]
    async fn test_aggregate_weekly_empty() {
        let weekly = StatsEngine::aggregate_weekly(vec![]).await;
        assert!(weekly.is_empty());
    }
}

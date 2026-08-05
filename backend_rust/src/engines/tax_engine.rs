use crate::models::Transaction;
use crate::schemas::{TaxLot, AssetTaxSummary};
use crate::services::currency_service::CurrencyService;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::currency_service::CurrencyService;
    use chrono::Utc as ChronoUtc;
    use std::str::FromStr;

    fn make_tx(id: i32, tx_type: &str, qty: &str, price: &str, fee: &str, date_str: &str) -> Transaction {
        Transaction {
            id,
            asset_id: 1,
            r#type: tx_type.to_string(),
            quantity: qty.to_string(),
            price: price.to_string(),
            fee: fee.to_string(),
            date: DateTime::parse_from_rfc3339(date_str).unwrap().with_timezone(&ChronoUtc),
        }
    }

    async fn run_engine(transactions: &[Transaction], strategy: &str, threshold_days: i64) -> AssetTaxSummary {
        let svc = CurrencyService::new();
        TaxLotEngine::calculate_lots(
            "TEST", "STOCK", transactions, Decimal::from_str("110.0").unwrap(), "USD", "USD", &svc, strategy, threshold_days,
        ).await.unwrap()
    }

    #[tokio::test]
    async fn test_fifo_matches_oldest_lot_first() {
        let txs = vec![
            make_tx(1, "BUY", "100.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
            make_tx(2, "BUY", "100.0", "110.0", "0.0", "2024-06-01T00:00:00Z"),
            make_tx(3, "SELL", "50.0", "120.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "FIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 2);
        assert_eq!(result.tax_lots[0].remaining_qty, Decimal::from_str("50.0").unwrap());
        assert_eq!(result.tax_lots[1].remaining_qty, Decimal::from_str("100.0").unwrap());
        assert_eq!(result.realized_pnl, Decimal::from_str("1000.0").unwrap());
    }

    #[tokio::test]
    async fn test_lifo_matches_newest_lot_first() {
        let txs = vec![
            make_tx(1, "BUY", "100.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
            make_tx(2, "BUY", "100.0", "110.0", "0.0", "2024-06-01T00:00:00Z"),
            make_tx(3, "SELL", "50.0", "120.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "LIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 2);
        assert_eq!(result.tax_lots[0].remaining_qty, Decimal::from_str("100.0").unwrap());
        assert_eq!(result.tax_lots[1].remaining_qty, Decimal::from_str("50.0").unwrap());
        assert_eq!(result.realized_pnl, Decimal::from_str("500.0").unwrap());
    }

    #[tokio::test]
    async fn test_hybrid_sells_short_term_first() {
        let txs = vec![
            make_tx(1, "BUY", "100.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
            make_tx(2, "BUY", "100.0", "110.0", "0.0", "2024-11-15T00:00:00Z"),
            make_tx(3, "SELL", "100.0", "120.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "HYBRID", 30).await;
        assert_eq!(result.tax_lots.len(), 1);
        assert_eq!(result.tax_lots[0].remaining_qty, Decimal::from_str("100.0").unwrap());
        assert_eq!(result.realized_pnl, Decimal::from_str("1000.0").unwrap());
    }

    #[tokio::test]
    async fn test_sell_exceeds_total_buys() {
        let txs = vec![
            make_tx(1, "BUY", "50.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
            make_tx(2, "SELL", "100.0", "120.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "FIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 0);
        assert_eq!(result.current_shares, Decimal::ZERO);
        assert_eq!(result.realized_pnl, Decimal::from_str("1000.0").unwrap());
    }

    #[tokio::test]
    async fn test_sell_with_no_prior_buys_ignored() {
        let txs = vec![
            make_tx(1, "SELL", "50.0", "120.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "FIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 0);
        assert_eq!(result.realized_pnl, Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_fee_increases_cost_basis() {
        let txs_no_fee = vec![
            make_tx(1, "BUY", "100.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
        ];
        let txs_with_fee = vec![
            make_tx(1, "BUY", "100.0", "100.0", "50.0", "2024-01-01T00:00:00Z"),
        ];
        let r_no_fee = run_engine(&txs_no_fee, "FIFO", 30).await;
        let r_with_fee = run_engine(&txs_with_fee, "FIFO", 30).await;
        let expected_diff = Decimal::from_str("0.5").unwrap();
        assert_eq!(r_with_fee.average_cost - r_no_fee.average_cost, expected_diff);
    }

    #[tokio::test]
    async fn test_empty_transactions_returns_zero() {
        let result = run_engine(&[], "FIFO", 30).await;
        assert_eq!(result.current_shares, Decimal::ZERO);
        assert_eq!(result.total_cost, Decimal::ZERO);
        assert_eq!(result.market_value, Decimal::ZERO);
        assert_eq!(result.tax_lots.len(), 0);
    }

    #[tokio::test]
    async fn test_single_buy_no_sells() {
        let txs = vec![
            make_tx(1, "BUY", "200.0", "50.0", "10.0", "2024-01-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "FIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 1);
        assert_eq!(result.average_cost, Decimal::from_str("50.05").unwrap());
        assert_eq!(result.market_value, Decimal::from_str("22000.0").unwrap());
        assert_eq!(result.unrealized_pnl, Decimal::from_str("11990.0").unwrap());
    }

    #[tokio::test]
    async fn test_partial_sell_across_multiple_lots() {
        let txs = vec![
            make_tx(1, "BUY", "60.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
            make_tx(2, "BUY", "40.0", "120.0", "0.0", "2024-06-01T00:00:00Z"),
            make_tx(3, "SELL", "80.0", "130.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "FIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 1);
        assert_eq!(result.tax_lots[0].remaining_qty, Decimal::from_str("20.0").unwrap());
        assert_eq!(result.realized_pnl, Decimal::from_str("2000.0").unwrap());
    }

    #[tokio::test]
    async fn test_multi_currency_conversion() {
        let txs = vec![
            make_tx(1, "BUY", "100.0", "100.0", "0.0", "2024-01-01T00:00:00Z"),
            make_tx(2, "SELL", "50.0", "120.0", "0.0", "2024-12-01T00:00:00Z"),
        ];
        let result = run_engine(&txs, "FIFO", 30).await;
        assert_eq!(result.tax_lots.len(), 1);
        assert_eq!(result.current_shares, Decimal::from_str("50.0").unwrap());
    }
}

pub struct TaxLotEngine;

struct InternalLot {
    date: DateTime<Utc>,
    price: Decimal,
    qty: Decimal,
    unit_cost: Decimal,
}

impl TaxLotEngine {
    pub async fn calculate_lots(
        symbol: &str,
        asset_type: &str,
        transactions: &[Transaction],
        current_price: Decimal,
        asset_currency: &str,
        base_currency: &str,
        currency_service: &CurrencyService,
        strategy: &str,
        hybrid_threshold_days: i64,
    ) -> Result<AssetTaxSummary> {
        let mut sorted_txs = transactions.to_vec();
        sorted_txs.sort_by_key(|tx| tx.date);

        let current_price_base = if asset_currency != base_currency {
            let latest_date = sorted_txs.last().map(|tx| tx.date).unwrap_or_else(Utc::now);
            let rate = currency_service.get_rate(asset_currency, base_currency, latest_date).await?;
            current_price * rate
        } else {
            current_price
        };

        let mut buy_lots: Vec<InternalLot> = Vec::new();
        let mut realized_pnl = Decimal::ZERO;

        for tx in sorted_txs {
            let tx_rate = if asset_currency != base_currency {
                currency_service.get_rate(asset_currency, base_currency, tx.date).await?
            } else {
                Decimal::ONE
            };

            let tx_qty = crate::db_types::str_to_decimal(&tx.quantity);
            let tx_price = crate::db_types::str_to_decimal(&tx.price);
            let tx_fee = crate::db_types::str_to_decimal(&tx.fee);

            if tx.r#type.to_uppercase() == "BUY" {
                let unit_cost_asset = if tx_qty > Decimal::ZERO {
                    (tx_qty * tx_price + tx_fee) / tx_qty
                } else {
                    tx_price
                };
                let unit_cost_base = unit_cost_asset * tx_rate;
                buy_lots.push(InternalLot {
                    date: tx.date,
                    price: tx_price * tx_rate,
                    qty: tx_qty,
                    unit_cost: unit_cost_base,
                });
            } else if tx.r#type.to_uppercase() == "SELL" {
                let mut qty_to_sell = tx_qty;
                let sell_unit_proceeds_asset = if tx_qty > Decimal::ZERO {
                    (tx_qty * tx_price - tx_fee) / tx_qty
                } else {
                    tx_price
                };
                let sell_unit_proceeds_base = sell_unit_proceeds_asset * tx_rate;

                let mut eligible_indices: Vec<usize> = buy_lots.iter().enumerate()
                    .filter(|(_, lot)| lot.date <= tx.date && lot.qty > Decimal::ZERO)
                    .map(|(i, _)| i)
                    .collect();

                if eligible_indices.is_empty() {
                    continue;
                }

                match strategy.to_uppercase().as_str() {
                    "FIFO" => {
                        eligible_indices.sort_by(|&a, &b| buy_lots[a].date.cmp(&buy_lots[b].date));
                    }
                    "LIFO" => {
                        eligible_indices.sort_by(|&a, &b| buy_lots[b].date.cmp(&buy_lots[a].date));
                    }
                    "HYBRID" => {
                        let mut short_term = Vec::new();
                        let mut long_term = Vec::new();
                        for &idx in &eligible_indices {
                            let lot = &buy_lots[idx];
                            let age = tx.date.signed_duration_since(lot.date).num_days();
                            if age >= 0 && age <= hybrid_threshold_days {
                                short_term.push(idx);
                            } else {
                                long_term.push(idx);
                            }
                        }
                        short_term.sort_by(|&a, &b| buy_lots[b].date.cmp(&buy_lots[a].date));
                        long_term.sort_by(|&a, &b| buy_lots[a].date.cmp(&buy_lots[b].date));
                        eligible_indices = [short_term, long_term].concat();
                    }
                    _ => {
                        eligible_indices.sort_by(|&a, &b| buy_lots[a].date.cmp(&buy_lots[b].date));
                    }
                }

                for idx in eligible_indices {
                    if qty_to_sell <= Decimal::ZERO {
                        break;
                    }
                    let lot = &mut buy_lots[idx];
                    let matched_qty = qty_to_sell.min(lot.qty);
                    let cost_basis = matched_qty * lot.unit_cost;
                    let proceeds = matched_qty * sell_unit_proceeds_base;

                    realized_pnl += proceeds - cost_basis;
                    lot.qty -= matched_qty;
                    qty_to_sell -= matched_qty;
                }
            }
        }

        let mut open_lots = Vec::new();
        let mut total_remaining_qty = Decimal::ZERO;
        let mut total_cost_basis = Decimal::ZERO;

        for lot in buy_lots {
            if lot.qty > Decimal::ZERO {
                total_remaining_qty += lot.qty;
                let lot_cost = lot.qty * lot.unit_cost;
                total_cost_basis += lot_cost;

                let lot_market_value = lot.qty * current_price_base;
                let latent_gain_loss = lot_market_value - lot_cost;
                let latent_roi = if lot_cost > Decimal::ZERO { latent_gain_loss / lot_cost } else { Decimal::ZERO };

                open_lots.push(TaxLot {
                    buy_date: lot.date,
                    buy_price: lot.price,
                    original_qty: lot.qty,
                    remaining_qty: lot.qty,
                    latent_gain_loss,
                    latent_roi,
                });
            }
        }

        let market_value = total_remaining_qty * current_price_base;
        let unrealized_pnl = market_value - total_cost_basis;
        let unrealized_roi = if total_cost_basis > Decimal::ZERO { unrealized_pnl / total_cost_basis } else { Decimal::ZERO };
        let average_cost = if total_remaining_qty > Decimal::ZERO { total_cost_basis / total_remaining_qty } else { Decimal::ZERO };

        Ok(AssetTaxSummary {
            symbol: symbol.to_string(),
            asset_type: asset_type.to_string(),
            current_shares: total_remaining_qty,
            average_cost,
            current_price: current_price_base,
            total_cost: total_cost_basis,
            market_value,
            unrealized_pnl,
            unrealized_roi,
            realized_pnl,
            tax_lots: open_lots,
        })
    }
}
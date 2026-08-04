use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::db_types::decimal_json;

// --- Transaction ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransactionCreate {
    pub r#type: String,
    #[serde(with = "decimal_json")]
    pub quantity: Decimal,
    #[serde(with = "decimal_json")]
    pub price: Decimal,
    #[serde(with = "decimal_json")]
    pub fee: Decimal,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransactionOut {
    pub id: i32,
    pub asset_id: i32,
    pub r#type: String,
    #[serde(with = "decimal_json")]
    pub quantity: Decimal,
    #[serde(with = "decimal_json")]
    pub price: Decimal,
    #[serde(with = "decimal_json")]
    pub fee: Decimal,
    pub date: DateTime<Utc>,
}

// --- Asset ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetCreate {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
    #[serde(default)]
    pub isin: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetOut {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
    pub isin: Option<String>,
    pub transactions: Vec<TransactionOut>,
}

// --- Portfolio ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortfolioCreate {
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortfolioOut {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub assets: Vec<AssetOut>,
}

// --- Portfolio Update ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortfolioUpdate {
    pub currency: String,
}

// --- Asset Update ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetUpdate {
    pub currency: String,
}

// --- Tax Lot Out ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaxLot {
    pub buy_date: DateTime<Utc>,
    #[serde(with = "decimal_json")]
    pub buy_price: Decimal,
    #[serde(with = "decimal_json")]
    pub original_qty: Decimal,
    #[serde(with = "decimal_json")]
    pub remaining_qty: Decimal,
    #[serde(with = "decimal_json")]
    pub latent_gain_loss: Decimal,
    #[serde(with = "decimal_json")]
    pub latent_roi: Decimal,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetTaxSummary {
    pub symbol: String,
    pub asset_type: String,
    #[serde(with = "decimal_json")]
    pub current_shares: Decimal,
    #[serde(with = "decimal_json")]
    pub average_cost: Decimal,
    #[serde(with = "decimal_json")]
    pub current_price: Decimal,
    #[serde(with = "decimal_json")]
    pub total_cost: Decimal,
    #[serde(with = "decimal_json")]
    pub market_value: Decimal,
    #[serde(with = "decimal_json")]
    pub unrealized_pnl: Decimal,
    #[serde(with = "decimal_json")]
    pub unrealized_roi: Decimal,
    #[serde(with = "decimal_json")]
    pub realized_pnl: Decimal,
    pub tax_lots: Vec<TaxLot>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn sample_datetime() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z").unwrap().with_timezone(&Utc)
    }

    #[test]
    fn test_transaction_create_roundtrip() {
        let tx = TransactionCreate {
            r#type: "BUY".to_string(),
            quantity: Decimal::from_str("100.0").unwrap(),
            price: Decimal::from_str("150.5").unwrap(),
            fee: Decimal::from_str("9.99").unwrap(),
            date: sample_datetime(),
        };
        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: TransactionCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.r#type, "BUY");
        assert!((deserialized.quantity - Decimal::from_str("100.0").unwrap()).abs() < Decimal::from_str("0.01").unwrap());
        assert!((deserialized.price - Decimal::from_str("150.5").unwrap()).abs() < Decimal::from_str("0.01").unwrap());
        assert!((deserialized.fee - Decimal::from_str("9.99").unwrap()).abs() < Decimal::from_str("0.01").unwrap());
    }

    #[test]
    fn test_transaction_out_roundtrip() {
        let tx = TransactionOut {
            id: 42,
            asset_id: 7,
            r#type: "SELL".to_string(),
            quantity: Decimal::from_str("25.0").unwrap(),
            price: Decimal::from_str("200.0").unwrap(),
            fee: Decimal::from_str("5.0").unwrap(),
            date: sample_datetime(),
        };
        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: TransactionOut = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.asset_id, 7);
        assert_eq!(deserialized.r#type, "SELL");
    }

    #[test]
    fn test_asset_create_roundtrip() {
        let a = AssetCreate {
            symbol: "TSLA".to_string(),
            name: "Tesla Inc".to_string(),
            asset_type: "STOCK".to_string(),
            sector: Some("Auto".to_string()),
            currency: "USD".to_string(),
            isin: "US88160R1014".to_string(),
        };
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: AssetCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "TSLA");
        assert_eq!(deserialized.asset_type, "STOCK");
        assert_eq!(deserialized.currency, "USD");
        assert_eq!(deserialized.isin, "US88160R1014");
        assert_eq!(deserialized.sector, Some("Auto".to_string()));
    }

    #[test]
    fn test_portfolio_create_with_none_description() {
        let p = PortfolioCreate {
            name: "My Portfolio".to_string(),
            description: None,
            currency: "USD".to_string(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: PortfolioCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "My Portfolio");
        assert_eq!(deserialized.description, None);
        assert_eq!(deserialized.currency, "USD");
    }

    #[test]
    fn test_portfolio_out_roundtrip() {
        let p = PortfolioOut {
            id: 1,
            name: "Test".to_string(),
            description: Some("A test portfolio".to_string()),
            currency: "EUR".to_string(),
            assets: vec![],
        };
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: PortfolioOut = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.name, "Test");
        assert_eq!(deserialized.description, Some("A test portfolio".to_string()));
    }

    #[test]
    fn test_asset_out_roundtrip() {
        let a = AssetOut {
            id: 1,
            portfolio_id: 1,
            symbol: "AAPL".to_string(),
            name: "Apple Inc".to_string(),
            asset_type: "STOCK".to_string(),
            sector: Some("Technology".to_string()),
            currency: "USD".to_string(),
            isin: Some("US0378331005".to_string()),
            transactions: vec![],
        };
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: AssetOut = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "AAPL");
        assert_eq!(deserialized.currency, "USD");
        assert_eq!(deserialized.isin, Some("US0378331005".to_string()));
        assert_eq!(deserialized.sector, Some("Technology".to_string()));
    }

    #[test]
    fn test_tax_lot_roundtrip() {
        let lot = TaxLot {
            buy_date: sample_datetime(),
            buy_price: Decimal::from_str("150.0").unwrap(),
            original_qty: Decimal::from_str("100.0").unwrap(),
            remaining_qty: Decimal::from_str("75.0").unwrap(),
            latent_gain_loss: Decimal::from_str("500.0").unwrap(),
            latent_roi: Decimal::from_str("0.0444").unwrap(),
        };
        let json = serde_json::to_string(&lot).unwrap();
        let deserialized: TaxLot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.remaining_qty, Decimal::from_str("75.0").unwrap());
        assert_eq!(deserialized.latent_gain_loss, Decimal::from_str("500.0").unwrap());
    }

    #[test]
    fn test_asset_tax_summary_roundtrip() {
        let summary = AssetTaxSummary {
            symbol: "AAPL".to_string(),
            asset_type: "STOCK".to_string(),
            current_shares: Decimal::from_str("100.0").unwrap(),
            average_cost: Decimal::from_str("150.0").unwrap(),
            current_price: Decimal::from_str("160.0").unwrap(),
            total_cost: Decimal::from_str("15000.0").unwrap(),
            market_value: Decimal::from_str("16000.0").unwrap(),
            unrealized_pnl: Decimal::from_str("1000.0").unwrap(),
            unrealized_roi: Decimal::from_str("0.0667").unwrap(),
            realized_pnl: Decimal::from_str("200.0").unwrap(),
            tax_lots: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: AssetTaxSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "AAPL");
        assert_eq!(deserialized.unrealized_pnl, Decimal::from_str("1000.0").unwrap());
    }

    #[test]
    fn test_portfolio_update_roundtrip() {
        let u = PortfolioUpdate {
            currency: "EUR".to_string(),
        };
        let json = serde_json::to_string(&u).unwrap();
        let deserialized: PortfolioUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.currency, "EUR");
    }

    #[test]
    fn test_asset_update_roundtrip() {
        let u = AssetUpdate {
            currency: "GBP".to_string(),
        };
        let json = serde_json::to_string(&u).unwrap();
        let deserialized: AssetUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.currency, "GBP");
    }
}

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

// --- Transaction ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransactionCreate {
    pub r#type: String,
    pub quantity: f64,
    pub price: f64,
    pub fee: f64,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransactionOut {
    pub id: i32,
    pub asset_id: i32,
    pub r#type: String,
    pub quantity: f64,
    pub price: f64,
    pub fee: f64,
    pub date: DateTime<Utc>,
}

// --- Asset ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetCreate {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetOut {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
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

// --- Tax Lot Out ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TaxLot {
    pub buy_date: DateTime<Utc>,
    pub buy_price: f64,
    pub original_qty: f64,
    pub remaining_qty: f64,
    pub latent_gain_loss: f64,
    pub latent_roi: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetTaxSummary {
    pub symbol: String,
    pub asset_type: String,
    pub current_shares: f64,
    pub average_cost: f64,
    pub current_price: f64,
    pub total_cost: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
    pub unrealized_roi: f64,
    pub realized_pnl: f64,
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
            quantity: 100.0,
            price: 150.5,
            fee: 9.99,
            date: sample_datetime(),
        };
        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: TransactionCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.r#type, "BUY");
        assert!((deserialized.quantity - 100.0).abs() < f64::EPSILON);
        assert!((deserialized.price - 150.5).abs() < f64::EPSILON);
        assert!((deserialized.fee - 9.99).abs() < f64::EPSILON);
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
            transactions: vec![],
        };
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: AssetOut = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "AAPL");
        assert_eq!(deserialized.sector, Some("Technology".to_string()));
    }

    #[test]
    fn test_tax_lot_roundtrip() {
        let lot = TaxLot {
            buy_date: sample_datetime(),
            buy_price: 150.0,
            original_qty: 100.0,
            remaining_qty: 75.0,
            latent_gain_loss: 500.0,
            latent_roi: 0.0444,
        };
        let json = serde_json::to_string(&lot).unwrap();
        let deserialized: TaxLot = serde_json::from_str(&json).unwrap();
        assert!((deserialized.remaining_qty - 75.0).abs() < f64::EPSILON);
        assert!((deserialized.latent_gain_loss - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_asset_tax_summary_roundtrip() {
        let summary = AssetTaxSummary {
            symbol: "AAPL".to_string(),
            asset_type: "STOCK".to_string(),
            current_shares: 100.0,
            average_cost: 150.0,
            current_price: 160.0,
            total_cost: 15000.0,
            market_value: 16000.0,
            unrealized_pnl: 1000.0,
            unrealized_roi: 0.0667,
            realized_pnl: 200.0,
            tax_lots: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: AssetTaxSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "AAPL");
        assert!((deserialized.unrealized_pnl - 1000.0).abs() < f64::EPSILON);
    }
}

use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Portfolio {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub base_currency: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Asset {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Transaction {
    pub id: i32,
    pub asset_id: i32,
    pub r#type: String, // 'type' is a keyword in Rust
    pub quantity: f64,
    pub price: f64,
    pub fee: f64,
    pub date: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct HistoricalPrice {
    pub symbol: String,
    pub date: NaiveDate,
    pub close_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc, NaiveDate};

    fn sample_dt() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z").unwrap().with_timezone(&Utc)
    }

    #[test]
    fn test_portfolio_roundtrip() {
        let p = Portfolio {
            id: 1,
            name: "Test".to_string(),
            description: Some("desc".to_string()),
            currency: "USD".to_string(),
            base_currency: "USD".to_string(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: Portfolio = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.currency, "USD");
    }

    #[test]
    fn test_asset_roundtrip() {
        let a = Asset {
            id: 1,
            portfolio_id: 1,
            symbol: "AAPL".to_string(),
            name: "Apple".to_string(),
            asset_type: "STOCK".to_string(),
            sector: Some("Tech".to_string()),
            currency: "USD".to_string(),
        };
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: Asset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "AAPL");
    }

    #[test]
    fn test_transaction_roundtrip() {
        let tx = Transaction {
            id: 1,
            asset_id: 1,
            r#type: "BUY".to_string(),
            quantity: 50.0,
            price: 200.0,
            fee: 5.0,
            date: sample_dt(),
        };
        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.r#type, "BUY");
        assert!((deserialized.quantity - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_historical_price_roundtrip() {
        let hp = HistoricalPrice {
            symbol: "SPY".to_string(),
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            close_price: 480.5,
        };
        let json = serde_json::to_string(&hp).unwrap();
        let deserialized: HistoricalPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "SPY");
        assert!((deserialized.close_price - 480.5).abs() < f64::EPSILON);
    }
}

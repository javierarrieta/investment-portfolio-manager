use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub base_currency: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Asset {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i32,
    pub asset_id: i32,
    pub r#type: String, // 'type' is a keyword in Rust
    pub quantity: f64,
    pub price: f64,
    pub fee: f64,
    pub date: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct HistoricalPrice {
    pub symbol: String,
    pub date: NaiveDate,
    pub close_price: f64,
}

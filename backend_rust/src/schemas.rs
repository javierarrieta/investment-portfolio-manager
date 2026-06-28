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

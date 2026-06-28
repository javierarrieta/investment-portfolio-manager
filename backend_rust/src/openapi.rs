use utoipa::OpenApi;
use crate::api_routes::{portfolios, transactions, analytics};
use crate::schemas;
use crate::models;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api_routes::portfolios::create_portfolio,
        crate::api_routes::portfolios::list_portfolios,
        crate::api_routes::portfolios::get_portfolio,
        crate::api_routes::portfolios::delete_portfolio,
        crate::api_routes::transactions::create_asset,
        crate::api_routes::transactions::delete_asset,
        crate::api_routes::transactions::create_transaction,
        crate::api_routes::transactions::list_portfolio_transactions,
        crate::api_routes::transactions::delete_transaction,
        crate::api_routes::analytics::get_portfolio_tax_summary,
        crate::api_routes::analytics::get_portfolio_performance,
    ),

    components(
        schemas(
            models::Portfolio,
            models::Asset,
            models::Transaction,
            schemas::PortfolioCreate,
            schemas::PortfolioOut,
            schemas::AssetCreate,
            schemas::AssetOut,
            schemas::TransactionCreate,
            schemas::TransactionOut,
            schemas::TaxLot,
            schemas::AssetTaxSummary,
        )
    )
)]
pub struct ApiDoc;

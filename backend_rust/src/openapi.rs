use utoipa::OpenApi;
use crate::schemas;
use crate::models;
use crate::api_routes::lookup::AssetLookupResult;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api_routes::portfolios::create_portfolio,
        crate::api_routes::portfolios::list_portfolios,
        crate::api_routes::portfolios::get_portfolio,
        crate::api_routes::portfolios::delete_portfolio,
        crate::api_routes::portfolios::update_portfolio,
        crate::api_routes::analytics::get_portfolio_tax_summary,
        crate::api_routes::analytics::get_portfolio_performance,
        crate::api_routes::lookup::lookup_isin,
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
            AssetLookupResult,
        )
    )
)]
pub struct ApiDoc;

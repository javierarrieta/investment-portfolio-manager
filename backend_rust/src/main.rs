#[macro_use] extern crate rocket;

mod models;
mod schemas;
mod services;
mod engines;
mod openapi;
mod api_routes {
    pub mod portfolios;
    pub mod transactions;
    pub mod analytics;
}

use rocket::serde::json::Json;
use sqlx::sqlite::SqlitePool;
use crate::services::currency_service::CurrencyService;
use utoipa::OpenApi;
use crate::openapi::ApiDoc;

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Investment Portfolio Manager API (Rust)"
}

#[launch]
async fn rocket() -> _ {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:../portfolio.db?mode=rwc".to_string());
    
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let currency_service = CurrencyService::new();

    rocket::build()
        .manage(pool)
        .manage(currency_service)
        .mount("/", routes![index])
        .mount("/api/portfolios", routes![
            api_routes::portfolios::create_portfolio,
            api_routes::portfolios::list_portfolios,
            api_routes::portfolios::get_portfolio,
            api_routes::portfolios::delete_portfolio,
            api_routes::portfolios::update_portfolio
        ])
        .mount("/api", routes![
            api_routes::transactions::create_asset,
            api_routes::transactions::update_asset,
            api_routes::transactions::delete_asset,
            api_routes::transactions::create_transaction,
            api_routes::transactions::list_portfolio_transactions,
            api_routes::transactions::delete_transaction
        ])
        .mount("/api/portfolios", routes![
            api_routes::analytics::get_portfolio_tax_summary,
            api_routes::analytics::get_portfolio_performance
        ])
        .mount("/api-docs", routes![openapi_json])
}

#[get("/openapi.json")]
fn openapi_json() -> Json<serde_json::Value> {
    Json(serde_json::to_value(ApiDoc::openapi()).unwrap())
}

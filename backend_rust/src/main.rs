#[macro_use] extern crate rocket;

mod models;
mod schemas;
mod services;
mod engines;
mod routes {
    pub mod portfolios;
    pub mod transactions;
    pub mod analytics;
}

use rocket::{State, serde::json::Json};
use sqlx::sqlite::SqlitePool;
use crate::services::currency_service::CurrencyService;

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Investment Portfolio Manager API (Rust)"
}

#[launch]
async fn rocket() -> _ {
    let database_url = "sqlite:/home/coder/investment-portfolio-manager/backend/portfolio.db";
    
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to connect to database");

    let currency_service = CurrencyService::new();

    rocket::build()
        .manage(pool)
        .manage(currency_service)
        .mount("/", routes![index])
        .mount("/api/portfolios", routes![
            routes::portfolios::create_portfolio,
            routes::portfolios::list_portfolios,
            routes::portfolios::get_portfolio,
            routes::portfolios::delete_portfolio
        ])
        .mount("/api", routes![
            routes::transactions::create_asset,
            routes::transactions::delete_asset,
            routes::transactions::create_transaction,
            routes::transactions::list_portfolio_transactions,
            routes::transactions::delete_transaction
        ])
        .mount("/api/portfolios", routes![
            routes::analytics::get_portfolio_tax_summary,
            routes::analytics::get_portfolio_performance
        ])
}

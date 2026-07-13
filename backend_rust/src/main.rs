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
use rocket_cors::{CorsOptions, AllowedOrigins, AllowedHeaders};

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

    // Configure CORS
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["http://localhost:5173", "http://127.0.0.1:5173"]),
        allowed_methods: vec![
            rocket::http::Method::Get,
            rocket::http::Method::Post,
            rocket::http::Method::Put,
            rocket::http::Method::Patch,
            rocket::http::Method::Delete,
            rocket::http::Method::Options,
        ]
            .into_iter()
            .map(|m| m.into())
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Content-Type", "Authorization"]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors().expect("Failed to configure CORS");

    // Auto-create tables on startup if they don't exist
    sqlx::query("CREATE TABLE IF NOT EXISTS portfolios (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        description TEXT,
        currency TEXT NOT NULL DEFAULT 'USD',
        base_currency TEXT NOT NULL DEFAULT 'USD'
    )").execute(&pool).await.expect("Failed to create portfolios table");

    sqlx::query("CREATE TABLE IF NOT EXISTS assets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        portfolio_id INTEGER NOT NULL,
        symbol TEXT NOT NULL,
        name TEXT NOT NULL,
        asset_type TEXT NOT NULL,
        sector TEXT,
        currency TEXT NOT NULL DEFAULT 'USD',
        FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
    )").execute(&pool).await.expect("Failed to create assets table");

    sqlx::query("CREATE TABLE IF NOT EXISTS transactions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        asset_id INTEGER NOT NULL,
        type TEXT NOT NULL,
        quantity REAL NOT NULL,
        price REAL NOT NULL,
        fee REAL NOT NULL,
        date TEXT NOT NULL,
        FOREIGN KEY (asset_id) REFERENCES assets(id)
    )").execute(&pool).await.expect("Failed to create transactions table");

    sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (
        symbol TEXT NOT NULL,
        date DATE NOT NULL,
        close_price REAL NOT NULL
    )").execute(&pool).await.expect("Failed to create historical_prices table");

    rocket::build()
        .attach(cors)
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

#[macro_use] extern crate rocket;

pub mod models;
pub mod schemas;
pub mod services;
pub mod engines;
pub mod openapi;
pub mod api_routes {
    pub mod portfolios;
    pub mod transactions;
    pub mod analytics;
}

use rocket::{Rocket, Build};
use rocket::serde::json::Json;
use sqlx::SqlitePool;
use crate::services::currency_service::CurrencyService;
use crate::openapi::ApiDoc;
use utoipa::OpenApi;
use rocket_cors::{CorsOptions, AllowedOrigins, AllowedHeaders, Cors};

#[get("/")]
pub fn index() -> &'static str {
    "Welcome to the Investment Portfolio Manager API (Rust)"
}

#[get("/openapi.json")]
pub fn openapi_json() -> Json<serde_json::Value> {
    Json(serde_json::to_value(ApiDoc::openapi()).unwrap())
}

pub fn get_cors_options(origins: Vec<String>) -> Cors {
    let origins_static: Vec<&'static str> = origins.into_iter()
        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
        .collect();

    CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&origins_static),
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
    }.to_cors().expect("Failed to configure CORS")
}

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS portfolios (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        description TEXT,
        currency TEXT NOT NULL DEFAULT 'USD',
        base_currency TEXT NOT NULL DEFAULT 'USD'
    )").execute(pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS assets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        portfolio_id INTEGER NOT NULL,
        symbol TEXT NOT NULL,
        name TEXT NOT NULL,
        asset_type TEXT NOT NULL,
        sector TEXT,
        currency TEXT NOT NULL DEFAULT 'USD',
        FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
    )").execute(pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS transactions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        asset_id INTEGER NOT NULL,
        type TEXT NOT NULL,
        quantity REAL NOT NULL,
        price REAL NOT NULL,
        fee REAL NOT NULL,
        date TEXT NOT NULL,
        FOREIGN KEY (asset_id) REFERENCES assets(id)
    )").execute(pool).await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (
        symbol TEXT NOT NULL,
        date DATE NOT NULL,
        close_price REAL NOT NULL
    )").execute(pool).await?;

    Ok(())
}

pub fn build_rocket(pool: SqlitePool, currency_service: CurrencyService, cors: Cors) -> Rocket<Build> {
    rocket::build()
        .attach(cors)
        .manage(pool)
        .manage(currency_service)
        .mount("/", routes![index, openapi_json])
        .mount("/api/portfolios", routes![
            api_routes::portfolios::create_portfolio,
            api_routes::portfolios::list_portfolios,
            api_routes::portfolios::get_portfolio,
            api_routes::portfolios::delete_portfolio,
            api_routes::portfolios::update_portfolio,
            api_routes::analytics::get_portfolio_tax_summary,
            api_routes::analytics::get_portfolio_performance,
        ])
        .mount("/api", routes![
            api_routes::transactions::create_asset,
            api_routes::transactions::update_asset,
            api_routes::transactions::delete_asset,
            api_routes::transactions::create_transaction,
            api_routes::transactions::list_portfolio_transactions,
            api_routes::transactions::delete_transaction,
        ])
        .mount("/api-docs", routes![openapi_json])
}

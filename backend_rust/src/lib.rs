#[macro_use] extern crate rocket;

#[get("/")]
pub fn index() -> &'static str {
    "Welcome to the Investment Portfolio Manager API (Rust)"
}

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

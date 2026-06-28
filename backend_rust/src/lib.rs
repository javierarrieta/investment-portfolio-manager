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

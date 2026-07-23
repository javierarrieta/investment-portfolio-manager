#[macro_use] extern crate rocket;

use sqlx::sqlite::SqlitePool;
use backend_rust::services::currency_service::CurrencyService;
use backend_rust::get_cors_options;
use backend_rust::build_rocket;
use backend_rust::init_db;

#[launch]
async fn rocket() -> _ {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:../portfolio.db?mode=rwc".to_string());
    
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    let currency_service = CurrencyService::new();

    let origins: Vec<String> = std::env::var("ALLOWED_ORIGINS")
        .map(|val| {
            eprintln!("ALLOWED_ORIGINS raw value: '{}'", val);
            val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        })
        .unwrap_or_else(|_| {
            eprintln!("ALLOWED_ORIGINS env var not set, using defaults");
            vec!["http://localhost:5173".to_string(), "http://127.0.0.1:5173".to_string()]
        });

    eprintln!("ALLOWED_ORIGINS parsed: {:?}", origins);

    let cors = get_cors_options(origins);

    init_db(&pool).await.expect("Failed to initialize database");

    build_rocket(pool, currency_service, cors)
}

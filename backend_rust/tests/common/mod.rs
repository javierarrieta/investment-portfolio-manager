use rocket::Rocket;
use sqlx::sqlite::SqlitePool;
use std::future::Future;

pub fn run_test<F, Fut, R>(f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f())
}
use chrono::{DateTime, Utc};

pub fn build_rocket(pool: SqlitePool) -> Rocket<rocket::Build> {
    use backend_rust::api_routes;
    use backend_rust::services::currency_service::CurrencyService;

    rocket::build()
        .manage(pool)
        .manage(CurrencyService::new())
        .mount("/", rocket::routes![backend_rust::index])
        .mount("/api/portfolios", rocket::routes![
            api_routes::portfolios::create_portfolio,
            api_routes::portfolios::list_portfolios,
            api_routes::portfolios::get_portfolio,
            api_routes::portfolios::delete_portfolio,
        ])
        .mount("/api", rocket::routes![
            api_routes::transactions::create_asset,
            api_routes::transactions::delete_asset,
            api_routes::transactions::create_transaction,
            api_routes::transactions::list_portfolio_transactions,
            api_routes::transactions::delete_transaction,
        ])
        .mount("/api/portfolios", rocket::routes![
            api_routes::analytics::get_portfolio_tax_summary,
            api_routes::analytics::get_portfolio_performance,
        ])
}

pub async fn setup_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    
    sqlx::query("CREATE TABLE IF NOT EXISTS portfolios (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        description TEXT,
        currency TEXT NOT NULL DEFAULT 'USD',
        base_currency TEXT NOT NULL DEFAULT 'USD'
    )").execute(&pool).await.unwrap();

    sqlx::query("CREATE TABLE IF NOT EXISTS assets (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        portfolio_id INTEGER NOT NULL,
        symbol TEXT NOT NULL,
        name TEXT NOT NULL,
        asset_type TEXT NOT NULL,
        sector TEXT,
        currency TEXT NOT NULL DEFAULT 'USD',
        FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
    )").execute(&pool).await.unwrap();

    sqlx::query("CREATE TABLE IF NOT EXISTS transactions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        asset_id INTEGER NOT NULL,
        type TEXT NOT NULL,
        quantity REAL NOT NULL,
        price REAL NOT NULL,
        fee REAL NOT NULL,
        date TEXT NOT NULL,
        FOREIGN KEY (asset_id) REFERENCES assets(id)
    )").execute(&pool).await.unwrap();

    sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (
        symbol TEXT NOT NULL,
        date DATE NOT NULL,
        close_price REAL NOT NULL
    )").execute(&pool).await.unwrap();

    pool
}

pub async fn seed_portfolio(pool: &SqlitePool, name: &str, currency: &str) -> i32 {
    let res = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO portfolios (name, description, currency, base_currency) VALUES (?, ?, ?, ?) RETURNING id"
    )
    .bind(name)
    .bind(None::<String>)
    .bind(currency)
    .bind(currency)
    .fetch_one(pool)
    .await
    .unwrap();
    res.0
}

pub async fn seed_asset(pool: &SqlitePool, portfolio_id: i32, symbol: &str, name: &str) -> i32 {
    let res = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO assets (portfolio_id, symbol, name, asset_type, currency) VALUES (?, ?, ?, ?, 'USD') RETURNING id"
    )
    .bind(portfolio_id)
    .bind(symbol)
    .bind(name)
    .bind("STOCK")
    .fetch_one(pool)
    .await
    .unwrap();
    res.0
}

pub async fn seed_transaction(
    pool: &SqlitePool,
    asset_id: i32,
    tx_type: &str,
    qty: f64,
    price: f64,
    fee: f64,
) -> i32 {
    let date = DateTime::parse_from_rfc3339("2024-06-15T00:00:00Z").unwrap().with_timezone(&Utc);
    let res = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO transactions (asset_id, type, quantity, price, fee, date) VALUES (?, ?, ?, ?, ?, ?) RETURNING id"
    )
    .bind(asset_id)
    .bind(tx_type)
    .bind(qty)
    .bind(price)
    .bind(fee)
    .bind(date)
    .fetch_one(pool)
    .await
    .unwrap();
    res.0
}

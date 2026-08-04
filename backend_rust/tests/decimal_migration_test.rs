use rust_decimal::Decimal;
use std::str::FromStr;
use sqlx::sqlite::SqlitePool;
use backend_rust::db_types::str_to_decimal;

async fn create_legacy_schema(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            fee REAL NOT NULL,
            date TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE historical_prices (
            symbol TEXT NOT NULL,
            date DATE NOT NULL,
            close_price REAL NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO transactions (asset_id, type, quantity, price, fee, date) VALUES (1, 'BUY', 100.0, 9.99, 0.0, '2024-01-01T00:00:00Z')")
        .execute(pool).await.unwrap();
    sqlx::query("INSERT INTO historical_prices (symbol, date, close_price) VALUES ('AAPL', '2024-01-01', 9.99)")
        .execute(pool).await.unwrap();
}

async fn column_affinity(pool: &SqlitePool, table: &str, column: &str) -> String {
    // PRAGMA table_info returns 6 columns: cid, name, type, notnull, dflt_value, pk
    let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as(&format!("PRAGMA table_info({table})")).fetch_all(pool).await.unwrap();
    rows.into_iter().find(|r| r.1 == column).unwrap().2
}

#[tokio::test]
async fn migration_converts_real_to_text_and_preserves_values() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();

    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "fee").await, "TEXT");
    assert_eq!(column_affinity(&pool, "historical_prices", "close_price").await, "TEXT");

    // values survive and parse as decimals ("100.0" came from REAL 100.0 via CAST AS TEXT)
    let (qty,): (String,) = sqlx::query_as("SELECT quantity FROM transactions WHERE id = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&qty), rust_decimal::Decimal::from_str("100.0").unwrap());
}

#[tokio::test]
async fn migration_is_idempotent() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();
    assert_eq!(column_affinity(&pool, "transactions", "price").await, "TEXT");
}

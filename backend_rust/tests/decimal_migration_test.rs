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

async fn create_partially_migrated_schema(pool: &SqlitePool) {
    // Start from clean legacy and simulate a crashed prior migration:
    // quantity & price were RENAMEd to _old but their TEXT replacements were never added.
    sqlx::query(
        "CREATE TABLE transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            quantity_old REAL NOT NULL,
            price_old REAL NOT NULL,
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
    sqlx::query("INSERT INTO transactions (asset_id, type, quantity_old, price_old, fee, date) VALUES (1, 'BUY', 100.0, 9.99, 0.0, '2024-01-01T00:00:00Z')")
        .execute(pool).await.unwrap();
    sqlx::query("INSERT INTO historical_prices (symbol, date, close_price) VALUES ('AAPL', '2024-01-01', 9.99)")
        .execute(pool).await.unwrap();
}

#[tokio::test]
async fn migration_recovers_partial_rename_state() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_partially_migrated_schema(&pool).await;
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();

    // quantity & price recreated as TEXT with preserved values, no _old leftovers.
    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "price").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "fee").await, "TEXT");
    assert_eq!(column_affinity(&pool, "historical_prices", "close_price").await, "TEXT");

    let (qty, price): (String, String) =
        sqlx::query_as("SELECT quantity, price FROM transactions WHERE id = 1")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&qty), rust_decimal::Decimal::from_str("100.0").unwrap());
    assert_eq!(str_to_decimal(&price), rust_decimal::Decimal::from_str("9.99").unwrap());

    let info: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(transactions)").fetch_all(&pool).await.unwrap();
    assert!(!info.iter().any(|r| r.1.ends_with("_old")), "no _old columns should remain");
}

#[tokio::test]
async fn migration_works_on_file_backed_database() {
    // The motivating bug (duplicate column name after ALTER) only manifests on
    // file-backed SQLite databases because of a stale per-connection schema cache.
    // `:memory:` connections hide it, so guard the file-backed code path explicitly.
    let tmp = std::env::temp_dir().join(format!(
        "portfolio_migration_test_{}.db",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&tmp);

    let url = format!("sqlite:{}?mode=rwc", tmp.display());
    let pool = SqlitePool::connect(&url).await.unwrap();
    create_legacy_schema(&pool).await;

    backend_rust::migrate_decimal_columns(&pool).await.unwrap();

    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "price").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "fee").await, "TEXT");
    assert_eq!(column_affinity(&pool, "historical_prices", "close_price").await, "TEXT");

    let (qty, price, fee): (String, String, String) =
        sqlx::query_as("SELECT quantity, price, fee FROM transactions WHERE id = 1")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&qty), rust_decimal::Decimal::from_str("100.0").unwrap());
    assert_eq!(str_to_decimal(&price), rust_decimal::Decimal::from_str("9.99").unwrap());
    assert_eq!(str_to_decimal(&fee), rust_decimal::Decimal::from_str("0.0").unwrap());

    let (close,): (String,) = sqlx::query_as(
        "SELECT close_price FROM historical_prices WHERE symbol = 'AAPL'",
    )
    .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&close), rust_decimal::Decimal::from_str("9.99").unwrap());

    let info: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(transactions)").fetch_all(&pool).await.unwrap();
    assert!(!info.iter().any(|r| r.1.ends_with("_old")), "no _old columns should remain");

    pool.close().await;
    let _ = std::fs::remove_file(&tmp);
}

#[tokio::test]
async fn migration_recovers_text_plus_leftover_old() {
    // simulate crash between ADD and DROP: column is TEXT, _old still present.
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;   // existing helper (clean legacy)
    sqlx::query("ALTER TABLE transactions RENAME COLUMN quantity TO quantity_old")
        .execute(&pool).await.unwrap();
    sqlx::query("ALTER TABLE transactions ADD COLUMN quantity TEXT NOT NULL DEFAULT '0'")
        .execute(&pool).await.unwrap();
    sqlx::query("UPDATE transactions SET quantity = CAST(quantity_old AS TEXT) WHERE id = 1")
        .execute(&pool).await.unwrap();
    // note: quantity_old still present -> simulates crash before DROP.

    backend_rust::migrate_decimal_columns(&pool).await.unwrap();
    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");
    let info: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(transactions)").fetch_all(&pool).await.unwrap();
    assert!(!info.iter().any(|r| r.1.ends_with("_old")), "no _old columns should remain");
}

#[tokio::test]
async fn migration_recovers_text_plus_old_that_was_never_copied() {
    // simulate crash between ADD and UPDATE: column is TEXT with '0' defaults,
    // `_old` still holds the real data. Dropping `_old` without re-copying would
    // lose the user's values.
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;
    sqlx::query("ALTER TABLE transactions RENAME COLUMN quantity TO quantity_old")
        .execute(&pool).await.unwrap();
    sqlx::query("ALTER TABLE transactions ADD COLUMN quantity TEXT NOT NULL DEFAULT '0'")
        .execute(&pool).await.unwrap();
    // note: no UPDATE ran, so `quantity` is '0' and `quantity_old` has the data.

    backend_rust::migrate_decimal_columns(&pool).await.unwrap();
    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");

    let (qty,): (String,) = sqlx::query_as("SELECT quantity FROM transactions WHERE id = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&qty), rust_decimal::Decimal::from_str("100.0").unwrap());

    let info: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(transactions)").fetch_all(&pool).await.unwrap();
    assert!(!info.iter().any(|r| r.1.ends_with("_old")), "no _old columns should remain");
}

#[tokio::test]
async fn migration_recovers_non_text_plus_stray_old() {
    // simulate a crash that left both the original non-TEXT column and a stray
    // `_old` leftover (the `(Some(_), true)` branch of migrate_one_column):
    // the leftover must be dropped, then the column migrated to TEXT.
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;
    sqlx::query("ALTER TABLE transactions RENAME COLUMN quantity TO quantity_old")
        .execute(&pool).await.unwrap();
    sqlx::query("ALTER TABLE transactions ADD COLUMN quantity REAL")
        .execute(&pool).await.unwrap();
    sqlx::query("UPDATE transactions SET quantity = quantity_old WHERE id = 1")
        .execute(&pool).await.unwrap();

    backend_rust::migrate_decimal_columns(&pool).await.unwrap();

    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "price").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "fee").await, "TEXT");

    let (qty,): (String,) = sqlx::query_as("SELECT quantity FROM transactions WHERE id = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&qty), rust_decimal::Decimal::from_str("100.0").unwrap());

    let info: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as("PRAGMA table_info(transactions)").fetch_all(&pool).await.unwrap();
    assert!(!info.iter().any(|r| r.1.ends_with("_old")), "no _old columns should remain");
}

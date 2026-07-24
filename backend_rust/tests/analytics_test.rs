mod common;

use common::{build_rocket, setup_db, seed_portfolio, seed_asset, seed_transaction};
use rocket::http::Status;
use rocket::local::asynchronous::Client;

#[tokio::test]
async fn test_tax_summary_empty_portfolio() {
    let pool = setup_db().await;
    let port_id = seed_portfolio(&pool, "Empty Tax", "USD").await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get(format!("/api/portfolios/{}/tax-summary?strategy=FIFO&threshold_days=30", port_id)).dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body_str = resp.into_string().await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(parsed["assets"], serde_json::json!([]));
}

#[tokio::test]
async fn test_tax_summary_with_assets_fifo() {
    let pool = setup_db().await;
    let port_id = seed_portfolio(&pool, "FIFO Tax", "USD").await;
    let asset_id = seed_asset(&pool, port_id, "AAPL", "Apple").await;
    seed_transaction(&pool, asset_id, "BUY", 100.0, 150.0, 0.0).await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get(format!("/api/portfolios/{}/tax-summary?strategy=FIFO&threshold_days=30", port_id)).dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body_str = resp.into_string().await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(parsed["strategy"], "FIFO");
    assert_eq!(parsed["assets"].as_array().unwrap().len(), 1);
    // Verify individual asset summary fields
    let asset = &parsed["assets"][0];
    assert_eq!(asset["symbol"], "AAPL");
    assert!(asset["current_shares"].is_number());
    assert!(asset["market_value"].is_number());
    assert!(asset["unrealized_pnl"].is_number());
    assert!(asset["realized_pnl"].is_number());
    assert!(asset["tax_lots"].is_array());
    // Verify tax lot data is present
    let lots = asset["tax_lots"].as_array().unwrap();
    assert_eq!(lots.len(), 1);
    let lot = &lots[0];
    assert!(lot["buy_date"].is_string());
    assert!(lot["buy_price"].is_number());
    assert!(lot["original_qty"].is_number());
    assert!(lot["remaining_qty"].is_number());
}

#[tokio::test]
async fn test_tax_summary_with_assets_lifo() {
    let pool = setup_db().await;
    let port_id = seed_portfolio(&pool, "LIFO Tax", "USD").await;
    let asset_id = seed_asset(&pool, port_id, "GOOGL", "Google").await;
    seed_transaction(&pool, asset_id, "BUY", 50.0, 200.0, 0.0).await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get(format!("/api/portfolios/{}/tax-summary?strategy=LIFO&threshold_days=30", port_id)).dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body_str = resp.into_string().await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(parsed["strategy"], "LIFO");
    assert_eq!(parsed["assets"].as_array().unwrap().len(), 1);
    let asset = &parsed["assets"][0];
    assert_eq!(asset["symbol"], "GOOGL");
}

#[tokio::test]
async fn test_tax_summary_404() {
    let pool = setup_db().await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get("/api/portfolios/9999/tax-summary?strategy=FIFO&threshold_days=30").dispatch().await;
    assert_eq!(resp.status(), Status::NotFound);
}

#[tokio::test]
async fn test_performance_404() {
    let pool = setup_db().await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get("/api/portfolios/9999/performance").dispatch().await;
    assert_eq!(resp.status(), Status::NotFound);
}

#[tokio::test]
async fn test_performance_endpoint_fields() {
    let pool = setup_db().await;
    let port_id = seed_portfolio(&pool, "Perf Fields", "USD").await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get(format!("/api/portfolios/{}/performance", port_id)).dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body_str = resp.into_string().await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    // Verify metrics fields exist
    let metrics = &parsed["metrics"];
    assert!(metrics.get("volatility").is_some());
    assert!(metrics.get("sharpe_ratio").is_some());
    assert!(metrics.get("beta").is_some());
    assert!(metrics.get("portfolio_value").is_some());
    // Verify history is an array
    assert!(parsed["history"].is_array());
    // Verify correlation_matrix exists
    assert!(parsed.get("correlation_matrix").is_some());
}

#[tokio::test]
async fn test_performance_endpoint() {
    let pool = setup_db().await;
    let port_id = seed_portfolio(&pool, "Perf Test", "USD").await;
    let rocket = build_rocket(pool);
    let client = Client::tracked(rocket).await.unwrap();

    let resp = client.get(format!("/api/portfolios/{}/performance", port_id)).dispatch().await;
    assert_eq!(resp.status(), Status::Ok);
    let body_str = resp.into_string().await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert!(parsed.get("metrics").is_some());
    assert!(parsed.get("history").is_some());
}

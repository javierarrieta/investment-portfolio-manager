mod common;

use common::{build_rocket, setup_db, seed_portfolio, seed_asset, seed_transaction};
use rocket::http::Status;
use rocket::local::asynchronous::Client;

#[test]
fn test_tax_summary_empty_portfolio() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "Empty Tax", "USD").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.get(format!("/api/portfolios/{}/tax-summary?strategy=FIFO&threshold_days=30", port_id)).dispatch().await;
        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["assets"], serde_json::json!([]));
    });
}

#[test]
fn test_tax_summary_with_assets_fifo() {
    common::run_test(|| async {
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
    });
}

#[test]
fn test_tax_summary_with_assets_lifo() {
    common::run_test(|| async {
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
    });
}

#[test]
fn test_performance_endpoint() {
    common::run_test(|| async {
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
    });
}

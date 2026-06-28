mod common;

use common::{build_rocket, setup_db, seed_portfolio, seed_asset, seed_transaction};
use rocket::http::Status;
use rocket::local::asynchronous::Client;
use rocket::serde::json::Json;

#[test]
fn test_create_asset() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "Tx Test", "USD").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let body = Json(serde_json::json!({
            "symbol": "AAPL",
            "name": "Apple Inc",
            "asset_type": "STOCK",
            "sector": "Technology"
        }));

        let resp = client.post(format!("/api/portfolios/{}/assets", port_id))
            .header(rocket::http::ContentType::JSON)
            .body(body.to_string())
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["symbol"], "AAPL");
        assert_eq!(parsed["portfolio_id"], port_id);
    });
}

#[test]
fn test_create_duplicate_asset_returns_400() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "Dup Test", "USD").await;
        let asset_id = seed_asset(&pool, port_id, "AAPL", "Apple").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let body = Json(serde_json::json!({
            "symbol": "AAPL",
            "name": "Apple Inc",
            "asset_type": "STOCK",
            "sector": null
        }));

        let resp = client.post(format!("/api/portfolios/{}/assets", port_id))
            .header(rocket::http::ContentType::JSON)
            .body(body.to_string())
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::BadRequest);
    });
}

#[test]
fn test_create_transaction() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "Create Tx", "USD").await;
        let asset_id = seed_asset(&pool, port_id, "GOOGL", "Google").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let body = Json(serde_json::json!({
            "type": "BUY",
            "quantity": 10.0,
            "price": 150.0,
            "fee": 5.0,
            "date": "2024-06-15T00:00:00Z"
        }));

        let resp = client.post(format!("/api/portfolios/{}/assets/{}/transactions", port_id, asset_id))
            .header(rocket::http::ContentType::JSON)
            .body(body.to_string())
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["type"], "BUY");
    });
}

#[test]
fn test_create_transaction_nonexistent_asset_returns_404() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "404 Tx", "USD").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let body = Json(serde_json::json!({
            "type": "BUY",
            "quantity": 10.0,
            "price": 150.0,
            "fee": 5.0,
            "date": "2024-06-15T00:00:00Z"
        }));

        let resp = client.post(format!("/api/portfolios/{}/assets/9999/transactions", port_id))
            .header(rocket::http::ContentType::JSON)
            .body(body.to_string())
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::NotFound);
    });
}

#[test]
fn test_list_portfolio_transactions() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "List Tx", "USD").await;
        let asset_id = seed_asset(&pool, port_id, "MSFT", "Microsoft").await;
        seed_transaction(&pool, asset_id, "BUY", 50.0, 300.0, 10.0).await;
        seed_transaction(&pool, asset_id, "BUY", 25.0, 320.0, 8.0).await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.get(format!("/api/portfolios/{}/transactions", port_id)).dispatch().await;
        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed.len(), 2);
    });
}

#[test]
fn test_delete_asset() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "Del Asset", "USD").await;
        let asset_id = seed_asset(&pool, port_id, "TSLA", "Tesla").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.delete(format!("/api/assets/{}", asset_id)).dispatch().await;
        assert_eq!(resp.status(), Status::NoContent);
    });
}

#[test]
fn test_delete_asset_not_found() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.delete("/api/assets/9999").dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
    });
}

#[test]
fn test_delete_transaction() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let port_id = seed_portfolio(&pool, "Del Tx", "USD").await;
        let asset_id = seed_asset(&pool, port_id, "NVDA", "Nvidia").await;
        let tx_id = seed_transaction(&pool, asset_id, "BUY", 10.0, 500.0, 5.0).await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.delete(format!("/api/transactions/{}", tx_id)).dispatch().await;
        assert_eq!(resp.status(), Status::NoContent);
    });
}

#[test]
fn test_delete_transaction_not_found() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.delete("/api/transactions/9999").dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
    });
}

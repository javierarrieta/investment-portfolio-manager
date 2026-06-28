mod common;

use common::{build_rocket, setup_db, seed_portfolio};
use rocket::http::Status;
use rocket::local::asynchronous::Client;
use rocket::serde::json::Json;

#[test]
fn test_create_portfolio() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let body = Json(serde_json::json!({
            "name": "Test Portfolio",
            "description": null,
            "currency": "USD"
        }));

        let resp = client.post("/api/portfolios/")
            .header(rocket::http::ContentType::JSON)
            .body(body.to_string())
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["name"], "Test Portfolio");
        assert_eq!(parsed["currency"], "USD");
    });
}

#[test]
fn test_list_portfolios() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let id = seed_portfolio(&pool, "List Test", "USD").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.get("/api/portfolios/").dispatch().await;
        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["id"], id);
    });
}

#[test]
fn test_get_portfolio_by_id() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let id = seed_portfolio(&pool, "Get Test", "EUR").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.get(format!("/api/portfolios/{}", id)).dispatch().await;
        assert_eq!(resp.status(), Status::Ok);
        let body_str = resp.into_string().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(parsed["name"], "Get Test");
        assert_eq!(parsed["currency"], "EUR");
    });
}

#[test]
fn test_get_portfolio_not_found() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.get("/api/portfolios/9999").dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
    });
}

#[test]
fn test_delete_portfolio() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let id = seed_portfolio(&pool, "Delete Test", "USD").await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.delete(format!("/api/portfolios/{}", id)).dispatch().await;
        assert_eq!(resp.status(), Status::NoContent);

        // Verify it's gone
        let resp = client.get(format!("/api/portfolios/{}", id)).dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
    });
}

#[test]
fn test_delete_portfolio_not_found() {
    common::run_test(|| async {
        let pool = setup_db().await;
        let rocket = build_rocket(pool);
        let client = Client::tracked(rocket).await.unwrap();

        let resp = client.delete("/api/portfolios/9999").dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
    });
}

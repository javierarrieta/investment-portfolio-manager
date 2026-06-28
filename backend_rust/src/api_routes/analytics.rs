use rocket::{State, serde::json::Json, http::Status};
use sqlx::SqlitePool;
use crate::models::{Portfolio, Asset, Transaction};
use crate::services::currency_service::CurrencyService;
use crate::engines::tax_engine::TaxLotEngine;
use crate::engines::stats_engine::StatsEngine;
use anyhow::Result;

#[utoipa::path(
    get,
    path = "/api/portfolios/<id>/tax-summary",
    responses(
        (status = 200, description = "Tax summary retrieved", body = serde_json::Value),
        (status = 404, description = "Portfolio not found")
    )
)]
#[get("/<id>/tax-summary?<strategy>&<threshold_days>")]
pub async fn get_portfolio_tax_summary(
    id: i32,
    pool: &State<SqlitePool>,
    currency_service: &State<CurrencyService>,
    strategy: Option<String>,
    threshold_days: Option<i64>,
) -> Result<Json<serde_json::Value>, Status> {
    let portfolio = sqlx::query_as::<_, Portfolio>("SELECT * FROM portfolios WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let assets = sqlx::query_as::<_, Asset>("SELECT * FROM assets WHERE portfolio_id = ?")
        .bind(id)
        .fetch_all(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if assets.is_empty() {
        return Ok(Json(serde_json::json!({
            "assets": [],
            "total_portfolio_value": 0.0,
            "total_realized_pnl": 0.0,
            "total_unrealized_pnl": 0.0
        })));
    }

    let mut asset_summaries = Vec::new();
    let mut total_value = 0.0;
    let mut total_realized = 0.0;
    let mut total_unrealized = 0.0;

    let strategy_val = strategy.unwrap_or_else(|| "FIFO".to_string());
    let threshold_val = threshold_days.unwrap_or(30);

    for asset in assets {
        let transactions = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE asset_id = ?")
            .bind(asset.id)
            .fetch_all(pool.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;

        // In a real app, we'd fetch the actual current price from a service
        let current_price = 100.0; 

        let summary = TaxLotEngine::calculate_lots(
            &asset.symbol,
            &asset.asset_type,
            &transactions,
            current_price,
            &asset.currency,
            &portfolio.base_currency,
            currency_service,
            &strategy_val,
            threshold_val,
        ).await.map_err(|_| Status::InternalServerError)?;

        total_value += summary.market_value;
        total_realized += summary.realized_pnl;
        total_unrealized += summary.unrealized_pnl;
        asset_summaries.push(summary);
    }

    Ok(Json(serde_json::json!({
        "assets": asset_summaries,
        "total_portfolio_value": total_value,
        "total_realized_pnl": total_realized,
        "total_unrealized_pnl": total_unrealized,
        "strategy": strategy_val,
        "threshold_days": threshold_val,
    })))
}

#[utoipa::path(
    get,
    path = "/api/portfolios/<id>/performance",
    responses(
        (status = 200, description = "Performance data retrieved", body = serde_json::Value),
        (status = 404, description = "Portfolio not found")
    )
)]
#[get("/<id>/performance")]
pub async fn get_portfolio_performance(
    id: i32,
    pool: &State<SqlitePool>,
    currency_service: &State<CurrencyService>,
) -> Result<Json<serde_json::Value>, Status> {
    let portfolio = sqlx::query_as::<_, Portfolio>("SELECT * FROM portfolios WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let assets = sqlx::query_as::<_, Asset>("SELECT * FROM assets WHERE portfolio_id = ?")
        .bind(id)
        .fetch_all(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE asset_id IN (SELECT id FROM assets WHERE portfolio_id = ?)"
    )
    .bind(id)
    .fetch_all(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    let perf = StatsEngine::calculate_portfolio_performance(
        pool.inner(),
        &assets,
        &transactions,
        &portfolio.base_currency,
        currency_service
    ).await.map_err(|_| Status::InternalServerError)?;

    Ok(Json(perf))
}

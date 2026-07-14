use rocket::{State, serde::json::Json, http::Status};
use sqlx::SqlitePool;
use crate::models::{Asset, Transaction};
use crate::schemas::{AssetCreate, AssetOut, AssetUpdate, TransactionCreate, TransactionOut};
use crate::services::currency_service::CurrencyService;

#[utoipa::path(
    post,
    path = "/api/portfolios/<portfolio_id>/assets",
    responses(
        (status = 201, description = "Asset created", body = AssetOut),
        (status = 400, description = "Asset already exists"),
        (status = 404, description = "Portfolio not found")
    )
)]
#[post("/portfolios/<portfolio_id>/assets", data = "<asset>")]
pub async fn create_asset(
    portfolio_id: i32,
    pool: &State<SqlitePool>,
    _currency_service: &State<CurrencyService>,
    asset: Json<AssetCreate>
) -> Result<Json<AssetOut>, Status> {
    let existing = sqlx::query_as::<_, Asset>(
        "SELECT * FROM assets WHERE portfolio_id = ? AND symbol = ?"
    )
    .bind(portfolio_id)
    .bind(asset.symbol.to_uppercase())
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    if existing.is_some() {
        return Err(Status::BadRequest);
    }

    let resolved_currency = if asset.currency.is_empty() {
        CurrencyService::detect_currency(&asset.symbol)
    } else {
        asset.currency.clone()
    };

    let res = sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (portfolio_id, symbol, name, asset_type, sector, currency) 
         VALUES (?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(portfolio_id)
    .bind(asset.symbol.to_uppercase())
    .bind(&asset.name)
    .bind(asset.asset_type.to_uppercase())
    .bind(&asset.sector)
    .bind(&resolved_currency)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AssetOut {
        id: res.id,
        portfolio_id: res.portfolio_id,
        symbol: res.symbol,
        name: res.name,
        asset_type: res.asset_type,
        sector: res.sector,
        currency: res.currency,
        transactions: vec![],
    }))
}

#[utoipa::path(
    patch,
    path = "/api/portfolios/<portfolio_id>/assets/<asset_id>",
    responses(
        (status = 200, description = "Asset updated", body = AssetOut),
        (status = 404, description = "Asset not found")
    )
)]
#[patch("/portfolios/<portfolio_id>/assets/<asset_id>", data = "<update>")]
pub async fn update_asset(
    portfolio_id: i32,
    asset_id: i32,
    pool: &State<SqlitePool>,
    update: Json<AssetUpdate>
) -> Result<Json<AssetOut>, Status> {
    sqlx::query("UPDATE assets SET currency = ? WHERE id = ? AND portfolio_id = ?")
        .bind(&update.currency)
        .bind(asset_id)
        .bind(portfolio_id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let asset = sqlx::query_as::<_, Asset>(
        "SELECT * FROM assets WHERE id = ? AND portfolio_id = ?"
    )
    .bind(asset_id)
    .bind(portfolio_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    match asset {
        Some(a) => Ok(Json(AssetOut {
            id: a.id,
            portfolio_id: a.portfolio_id,
            symbol: a.symbol,
            name: a.name,
            asset_type: a.asset_type,
            sector: a.sector,
            currency: a.currency,
            transactions: vec![],
        })),
        None => Err(Status::NotFound),
    }
}

#[utoipa::path(
    delete,
    path = "/api/assets/<id>",
    responses(
        (status = 204, description = "Asset deleted"),
        (status = 404, description = "Asset not found")
    )
)]
#[delete("/assets/<id>")]
pub async fn delete_asset(id: i32, pool: &State<SqlitePool>) -> Result<Status, Status> {
    let res = sqlx::query("DELETE FROM assets WHERE id = ?")
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if res.rows_affected() == 0 {
        return Err(Status::NotFound);
    }

    Ok(Status::NoContent)
}

#[utoipa::path(
    post,
    path = "/api/portfolios/<portfolio_id>/assets/<asset_id>/transactions",
    responses(
        (status = 201, description = "Transaction created", body = TransactionOut),
        (status = 404, description = "Asset not found")
    )
)]
#[post("/portfolios/<portfolio_id>/assets/<asset_id>/transactions", data = "<tx>")]
pub async fn create_transaction(
    portfolio_id: i32,
    asset_id: i32,
    pool: &State<SqlitePool>,
    tx: Json<TransactionCreate>
) -> Result<Json<TransactionOut>, Status> {
    let asset = sqlx::query_as::<_, Asset>("SELECT * FROM assets WHERE id = ? AND portfolio_id = ?")
        .bind(asset_id)
        .bind(portfolio_id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if asset.is_none() {
        return Err(Status::NotFound);
    }

    let res = sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (asset_id, type, quantity, price, fee, date) 
         VALUES (?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(asset_id)
    .bind(tx.r#type.to_uppercase())
    .bind(tx.quantity)
    .bind(tx.price)
    .bind(tx.fee)
    .bind(tx.date)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(TransactionOut {
        id: res.id,
        asset_id: res.asset_id,
        r#type: res.r#type,
        quantity: res.quantity,
        price: res.price,
        fee: res.fee,
        date: res.date,
    }))
}

#[utoipa::path(
    get,
    path = "/api/portfolios/<portfolio_id>/transactions",
    responses(
        (status = 200, description = "List of transactions", body = [TransactionOut]),
        (status = 404, description = "Portfolio not found")
    )
)]
#[get("/portfolios/<portfolio_id>/transactions")]
pub async fn list_portfolio_transactions(portfolio_id: i32, pool: &State<SqlitePool>) -> Result<Json<Vec<TransactionOut>>, Status> {
    let assets = sqlx::query_as::<_, (i32,)>("SELECT id FROM assets WHERE portfolio_id = ?")
        .bind(portfolio_id)
        .fetch_all(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let asset_ids: Vec<i32> = assets.into_iter().map(|a| a.0).collect();

    
    if asset_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    if asset_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let mut all_txs = Vec::new();
    for id in asset_ids {
        let txs = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE asset_id = ?")
            .bind(id)
            .fetch_all(pool.inner())
            .await
            .map_err(|_| Status::InternalServerError)?;
        all_txs.extend(txs);
    }
    all_txs.sort_by(|a, b| b.date.cmp(&a.date));

    let out = all_txs.into_iter().map(|t| TransactionOut {

        id: t.id,
        asset_id: t.asset_id,
        r#type: t.r#type,
        quantity: t.quantity,
        price: t.price,
        fee: t.fee,
        date: t.date,
    }).collect();

    Ok(Json(out))
}

#[utoipa::path(
    delete,
    path = "/api/transactions/<id>",
    responses(
        (status = 204, description = "Transaction deleted"),
        (status = 404, description = "Transaction not found")
    )
)]
#[delete("/transactions/<id>")]
pub async fn delete_transaction(id: i32, pool: &State<SqlitePool>) -> Result<Status, Status> {
    let res = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if res.rows_affected() == 0 {
        return Err(Status::NotFound);
    }

    Ok(Status::NoContent)
}

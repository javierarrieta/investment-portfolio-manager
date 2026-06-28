use rocket::{State, serde::json::Json, http::Status};
use sqlx::SqlitePool;
use crate::models::Portfolio as DbPortfolio;
use crate::schemas::{PortfolioCreate, PortfolioOut};

#[post("/", data = "<portfolio>")]
pub async fn create_portfolio(
    pool: &State<SqlitePool>,
    portfolio: Json<PortfolioCreate>
) -> Result<Json<PortfolioOut>, Status> {
    let res = sqlx::query_as::<_, DbPortfolio>(
        "INSERT INTO portfolios (name, description, currency, base_currency) 
         VALUES (?, ?, ?, ?) RETURNING *"
    )
    .bind(&portfolio.name)
    .bind(&portfolio.description)
    .bind(&portfolio.currency)
    .bind(&portfolio.currency) // Using currency as base_currency default
    .fetch_one(pool.inner())
    .await;

    match res {
        Ok(p) => Ok(Json(PortfolioOut {
            id: p.id,
            name: p.name,
            description: p.description,
            currency: p.currency,
            assets: vec![],
        })),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/")]
pub async fn list_portfolios(pool: &State<SqlitePool>) -> Result<Json<Vec<PortfolioOut>>, Status> {
    let portfolios = sqlx::query_as::<_, DbPortfolio>("SELECT * FROM portfolios")
        .fetch_all(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let out = portfolios.into_iter().map(|p| PortfolioOut {
        id: p.id,
        name: p.name,
        description: p.description,
        currency: p.currency,
        assets: vec![],
    }).collect();

    Ok(Json(out))
}

#[get("/<id>")]
pub async fn get_portfolio(id: i32, pool: &State<SqlitePool>) -> Result<Json<PortfolioOut>, Status> {
    let p = sqlx::query_as::<_, DbPortfolio>("SELECT * FROM portfolios WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    match p {
        Some(p) => Ok(Json(PortfolioOut {
            id: p.id,
            name: p.name,
            description: p.description,
            currency: p.currency,
            assets: vec![],
        })),
        None => Err(Status::NotFound),
    }
}

#[delete("/<id>")]
pub async fn delete_portfolio(id: i32, pool: &State<SqlitePool>) -> Result<Status, Status> {
    let res = sqlx::query("DELETE FROM portfolios WHERE id = ?")
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    if res.rows_affected() == 0 {
        return Err(Status::NotFound);
    }

    Ok(Status::NoContent)
}

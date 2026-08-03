use rocket::{serde::json::Json, http::Status};
use serde::Serialize;
use utoipa::ToSchema;
use crate::services::currency_service::CurrencyService;

#[derive(Serialize, ToSchema)]
pub struct AssetLookupResult {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub currency: String,
}

#[utoipa::path(
    get,
    path = "/api/assets/lookup",
    params(
        ("isin" = String, Query, description = "ISIN to look up (12 alphanumeric characters)")
    ),
    responses(
        (status = 200, description = "Asset metadata resolved from ISIN", body = AssetLookupResult),
        (status = 400, description = "Invalid ISIN format"),
        (status = 404, description = "ISIN not found")
    )
)]
#[get("/assets/lookup?<isin>")]
pub async fn lookup_isin(
    isin: String,
) -> Result<Json<AssetLookupResult>, Status> {
    if !is_valid_isin(&isin) {
        return Err(Status::BadRequest);
    }

    let url = format!("https://query1.finance.yahoo.com/v1/finance/search?q={}", isin);
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !response.status().is_success() {
        return Err(Status::InternalServerError);
    }

    let body: serde_json::Value = response.json().await
        .map_err(|_| Status::InternalServerError)?;

    let quotes = body.get("quotes")
        .and_then(|q| q.as_array())
        .and_then(|arr| arr.first())
        .ok_or(Status::NotFound)?;

    let symbol = quotes.get("symbol")
        .and_then(|s| s.as_str())
        .ok_or(Status::NotFound)?
        .to_string();

    let name = quotes.get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(&symbol)
        .to_string();

    let exchange = quotes.get("exchange")
        .and_then(|e| e.as_str())
        .unwrap_or("");

    let asset_type = if exchange.contains("CB") || exchange.contains("CM") {
        "CRYPTO".to_string()
    } else if exchange.contains("INDEX") {
        "ETF".to_string()
    } else {
        "STOCK".to_string()
    };

    let currency = CurrencyService::detect_currency(&symbol);

    Ok(Json(AssetLookupResult {
        symbol,
        name,
        asset_type,
        currency,
    }))
}

pub fn is_valid_isin(isin: &str) -> bool {
    let isin = isin.to_uppercase();
    if isin.len() != 12 {
        return false;
    }
    let mut chars = isin.chars();
    let country_code = chars.next().unwrap();
    let second = chars.next().unwrap();
    if !country_code.is_ascii_alphabetic() || !second.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_isin() {
        assert!(is_valid_isin("US0378331005"));
        assert!(is_valid_isin("DE000BAY0017"));
    }

    #[test]
    fn test_invalid_isin_too_short() {
        assert!(!is_valid_isin("US037833100"));
    }

    #[test]
    fn test_invalid_isin_too_long() {
        assert!(!is_valid_isin("US03783310050"));
    }

    #[test]
    fn test_invalid_isin_non_alphanumeric() {
        assert!(!is_valid_isin("US037833100!"));
    }

    #[test]
    fn test_invalid_isin_no_country_code() {
        assert!(!is_valid_isin("0378331005"));
    }

    #[test]
    fn test_invalid_isin_lowercase_country_code() {
        assert!(is_valid_isin("us0378331005"));
    }
}

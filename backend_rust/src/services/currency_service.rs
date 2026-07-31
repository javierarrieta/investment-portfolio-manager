use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc, NaiveDate};
use reqwest::Client;
use serde::Deserialize;
use anyhow::{Result, anyhow};
use crate::models::HistoricalPrice;
use sqlx::sqlite::SqlitePool;

const RATE_LIMIT_DELAY: Duration = Duration::from_millis(200);
const MAX_RETRIES: usize = 3;

#[derive(Deserialize)]
struct YahooChartResponse {
    chart: Chart,
}

#[derive(Deserialize)]
struct Chart {
    result: Vec<ChartResult>,
}

#[derive(Deserialize)]
struct ChartResult {
    indicators: Indicators,
    #[allow(dead_code)]
    timestamp: Vec<i64>,
}

#[derive(Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
}

#[derive(Deserialize)]
struct Quote {
    close: Vec<Option<f64>>,
}

pub struct CurrencyService {
    client: Client,
    timeout: std::time::Duration,
    cache: Arc<RwLock<HashMap<(String, String, NaiveDate), f64>>>,
    last_request: Mutex<Option<Instant>>,
}

impl CurrencyService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("investment-portfolio-manager/1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
            timeout: std::time::Duration::from_secs(10),
            cache: Arc::new(RwLock::new(HashMap::new())),
            last_request: Mutex::new(None),
        }
    }

    async fn fetch_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        let mut retry_delay = Duration::from_secs(1);

        for attempt in 0..MAX_RETRIES {
            {
                let last = self.last_request.lock().await;
                if let Some(instant) = *last {
                    let elapsed = instant.elapsed();
                    if elapsed < RATE_LIMIT_DELAY {
                        let wait = RATE_LIMIT_DELAY - elapsed;
                        drop(last);
                        tokio::time::sleep(wait).await;
                    }
                }
            }

            let response = self.client.get(url).timeout(self.timeout).send().await;

            {
                let mut last = self.last_request.lock().await;
                *last = Some(Instant::now());
            }

            match response {
                Ok(resp) if resp.status() == 429 => {
                    eprintln!(
                        "WARN: Yahoo Finance returned 429 (attempt {}/{}), retrying in {:?}",
                        attempt + 1,
                        MAX_RETRIES,
                        retry_delay
                    );
                    tokio::time::sleep(retry_delay).await;
                    retry_delay *= 2;
                    continue;
                }
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    if attempt < MAX_RETRIES - 1 {
                        eprintln!(
                            "WARN: Request to {} failed (attempt {}/{}): {}, retrying in {:?}",
                            url,
                            attempt + 1,
                            MAX_RETRIES,
                            e,
                            retry_delay
                        );
                        tokio::time::sleep(retry_delay).await;
                        retry_delay *= 2;
                        continue;
                    }
                    return Err(anyhow!(
                        "Request to {} failed after {} retries: {}",
                        url,
                        MAX_RETRIES,
                        e
                    ));
                }
            }
        }

        Err(anyhow!("Max retries exceeded for {}", url))
    }

    pub async fn get_rate(&self, from_curr: &str, to_curr: &str, date: DateTime<Utc>) -> Result<f64> {
        let date_naive = date.date_naive();
        let cache_key = (from_curr.to_string(), to_curr.to_string(), date_naive);

        {
            let cache = self.cache.read().await;
            if let Some(&rate) = cache.get(&cache_key) {
                return Ok(rate);
            }
        }

        if from_curr == to_curr {
            return Ok(1.0);
        }

        let symbol = format!("{}={}", from_curr, to_curr);
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);

        let response = self.fetch_with_retry(&url).await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Yahoo Finance returned status {} for symbol {}",
                response.status(),
                symbol
            ));
        }

        let yahoo_resp: YahooChartResponse = response.json().await?;
        
        let result = yahoo_resp.chart.result.first()
            .ok_or_else(|| anyhow!("No result found for symbol {}", symbol))?;
        
        let close_prices = &result.indicators.quote.first()
            .ok_or_else(|| anyhow!("No quote data found for symbol {}", symbol))?
            .close;
        
        // For simplicity, we take the last available close price if we can't find the exact date.
        // In a more robust implementation, we'd match the timestamp.
        let rate = close_prices.iter().rev()
            .find_map(|x| *x)
            .ok_or_else(|| anyhow!("No close price found for symbol {}", symbol))?;

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, rate);

        Ok(rate)
    }

    /// Fetches the latest close price for a ticker symbol from Yahoo Finance.
    /// Caches the result in the historical_prices table for future lookups.
    /// Returns 0.0 if the price cannot be fetched (logged warning).
    pub async fn get_price(&self, symbol: &str, pool: &SqlitePool) -> f64 {
        // 1. Check if we have a cached price for today
        let today = chrono::Utc::now().date_naive();
        let existing = sqlx::query_as::<_, HistoricalPrice>(
            "SELECT * FROM historical_prices WHERE symbol = ? AND date = ?"
        )
        .bind(symbol)
        .bind(today)
        .fetch_optional(pool)
        .await;

        if let Ok(Some(hp)) = existing {
            return hp.close_price;
        }

        // 2. Fetch from Yahoo Finance
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
        let response = match self.fetch_with_retry(&url).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("WARN: Failed to fetch price for {}: {}", symbol, e);
                return 0.0;
            }
        };

        let status = response.status();
        let content_type = response.headers().get("content-type").map(|v| v.to_str().unwrap_or("unknown")).unwrap_or("unknown").to_string();

        if !status.is_success() {
            let body_preview = response.text().await.unwrap_or_default();
            eprintln!(
                "WARN: Yahoo Finance returned status {} for symbol {}. Body: {}",
                status,
                symbol,
                &body_preview[..body_preview.len().min(500)]
            );
            return 0.0;
        }

        let body_bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("WARN: Failed to read response body for {}: {}", symbol, e);
                return 0.0;
            }
        };

        let body_preview = String::from_utf8_lossy(&body_bytes).to_string();
        let yahoo_resp: YahooChartResponse = match serde_json::from_slice(&body_bytes) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "WARN: Failed to parse Yahoo response for {}: {}. Status: {}, Content-Type: {}, Body: {}",
                    symbol,
                    e,
                    status,
                    content_type,
                    &body_preview[..body_preview.len().min(500)]
                );
                return 0.0;
            }
        };

        let result = yahoo_resp.chart.result.into_iter().next();
        let close_prices = match result {
            Some(r) => match r.indicators.quote.first() {
                Some(q) => q.close.clone(),
                None => {
                    eprintln!("WARN: No quote data for symbol: {}", symbol);
                    return 0.0;
                }
            },
            None => {
                eprintln!("WARN: No price data for symbol: {}", symbol);
                return 0.0;
            }
        };

        let price = close_prices.iter().rev().find_map(|x| *x).unwrap_or(0.0);
        if price == 0.0 {
            eprintln!("WARN: No valid close price for symbol: {}", symbol);
            return 0.0;
        }

        // 3. Cache in historical_prices table
        let today = chrono::Utc::now().date_naive();
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO historical_prices (symbol, date, close_price) VALUES (?, ?, ?)"
        )
        .bind(symbol)
        .bind(today)
        .bind(price)
        .execute(pool)
        .await;

        price
    }

    /// Fetches historical daily close prices for a symbol from Yahoo Finance.
    /// Calls the /chart endpoint with explicit period1/period2 and interval=1d.
    /// Batch upserts all daily prices into historical_prices (INSERT OR REPLACE).
    /// If Yahoo fetch fails or returns no data, falls back to whatever exists in the DB.
    pub async fn get_historical_prices(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        pool: &SqlitePool,
    ) -> Result<Vec<(NaiveDate, f64)>> {
        let start_ts = start_date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let end_ts = end_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d&includePrePost=false",
            symbol, start_ts, end_ts
        );

        let response = match self.fetch_with_retry(&url).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("WARN: Failed to fetch historical prices for {}: {}", symbol, e);
                return self.get_historical_prices_from_db(symbol, start_date, end_date, pool).await;
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body_preview = response.text().await.unwrap_or_default();
            eprintln!(
                "WARN: Yahoo Finance returned status {} for symbol {}. Body: {}",
                status,
                symbol,
                &body_preview[..body_preview.len().min(500)]
            );
            return self.get_historical_prices_from_db(symbol, start_date, end_date, pool).await;
        }

        let yahoo_resp: YahooChartResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("WARN: Failed to parse historical Yahoo response for {}: {}", symbol, e);
                return self.get_historical_prices_from_db(symbol, start_date, end_date, pool).await;
            }
        };

        let result = yahoo_resp.chart.result.into_iter().next();
        let (timestamps, close_prices) = match result {
            Some(r) => {
                let ts = r.timestamp;
                let cp = r
                    .indicators
                    .quote
                    .first()
                    .map(|q| q.close.clone())
                    .unwrap_or_default();
                (ts, cp)
            }
            None => {
                eprintln!("WARN: No chart result for symbol: {}", symbol);
                return self.get_historical_prices_from_db(symbol, start_date, end_date, pool).await;
            }
        };

        let mut daily_prices = Vec::new();
        for (i, ts) in timestamps.iter().enumerate() {
            if i < close_prices.len() {
                if let Some(close) = close_prices[i] {
                    if close > 0.0 {
                        let date = DateTime::from_timestamp(*ts, 0)
                            .map(|dt| dt.date_naive())
                            .unwrap_or_default();
                        daily_prices.push((date, close));
                    }
                }
            }
        }

        if !daily_prices.is_empty() {
            let mut tx = pool.begin().await?;
            for (date, price) in &daily_prices {
                let _ = sqlx::query(
                    "INSERT OR REPLACE INTO historical_prices (symbol, date, close_price) VALUES (?, ?, ?)"
                )
                .bind(symbol)
                .bind(*date)
                .bind(*price)
                .execute(&mut *tx)
                .await;
            }
            let _ = tx.commit().await;
        }

        if daily_prices.is_empty() {
            return self.get_historical_prices_from_db(symbol, start_date, end_date, pool).await;
        }

        // Merge with DB prices for any dates not covered by Yahoo Finance
        // This ensures we have prices for today even when Yahoo hasn't closed the market yet
        let db_prices = self.get_historical_prices_from_db(symbol, start_date, end_date, pool).await?;
        let yahoo_dates: std::collections::HashSet<NaiveDate> = daily_prices.iter().map(|(d, _)| *d).collect();
        
        let mut merged: Vec<(NaiveDate, f64)> = daily_prices;
        for (date, price) in db_prices {
            if !yahoo_dates.contains(&date) {
                merged.push((date, price));
            }
        }
        merged.sort_by_key(|(date, _)| *date);

        Ok(merged)
    }

    async fn get_historical_prices_from_db(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        pool: &SqlitePool,
    ) -> Result<Vec<(NaiveDate, f64)>> {
        let prices = sqlx::query_as::<_, HistoricalPrice>(
            "SELECT * FROM historical_prices WHERE symbol = ? AND date >= ? AND date <= ?"
        )
        .bind(symbol)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await?;

        Ok(prices.into_iter().map(|p| (p.date, p.close_price)).collect())
    }

    pub fn detect_currency(symbol: &str) -> String {
        let upper = symbol.to_uppercase();
        if upper.ends_with(".DE") || upper.ends_with(".F") || upper.ends_with(".FR") || upper.ends_with(".MC") {
            "EUR".to_string()
        } else if upper.ends_with(".L") {
            "GBP".to_string()
        } else if upper.ends_with(".T") {
            "JPY".to_string()
        } else if upper.ends_with(".HK") {
            "HKD".to_string()
        } else if upper.ends_with(".SX") || upper.ends_with(".SW") {
            "CHF".to_string()
        } else if upper.ends_with(".TO") {
            "CAD".to_string()
        } else if upper.ends_with(".AX") {
            "AUD".to_string()
        } else if upper.ends_with(".K") {
            "KRW".to_string()
        } else if upper.contains("USD") || upper.contains("BTC") || upper.contains("ETH") {
            "USD".to_string()
        } else {
            "USD".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_same_currency_returns_one() {
        let svc = CurrencyService::new();
        let date = DateTime::from_timestamp(1705312200, 0).unwrap();
        let result = svc.get_rate("USD", "USD", date).await;
        assert!(result.is_ok());
        assert!((result.unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_same_currency_returns_one_consistently() {
        let svc = CurrencyService::new();
        let date1 = DateTime::from_timestamp(1700000001, 0).unwrap();
        let date2 = DateTime::from_timestamp(1700000001, 0).unwrap();
        
        let r1 = svc.get_rate("GBP", "GBP", date1).await.unwrap();
        let r2 = svc.get_rate("GBP", "GBP", date2).await.unwrap();
        assert!((r1 - r2).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_price_uses_today_cached_price() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (symbol TEXT, date DATE, close_price REAL)")
            .execute(&pool)
            .await
            .unwrap();

        let today = chrono::Utc::now().date_naive();
        let _ = sqlx::query(
            "INSERT INTO historical_prices (symbol, date, close_price) VALUES (?, ?, ?)"
        )
        .bind("TEST")
        .bind(today)
        .bind(42.0)
        .execute(&pool)
        .await;

        let svc = CurrencyService::new();
        let price = svc.get_price("TEST", &pool).await;
        assert!((price - 42.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_price_ignores_stale_cache() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS historical_prices (symbol TEXT, date DATE, close_price REAL)")
            .execute(&pool)
            .await
            .unwrap();

        let stale_date = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let _ = sqlx::query(
            "INSERT INTO historical_prices (symbol, date, close_price) VALUES (?, ?, ?)"
        )
        .bind("TEST")
        .bind(stale_date)
        .bind(99.0)
        .execute(&pool)
        .await;

        let svc = CurrencyService::new();
        // Should not return stale price (99.0) - falls through to Yahoo fetch
        let price = svc.get_price("TEST", &pool).await;
        assert!((price - 99.0).abs() > f64::EPSILON, "should not return stale cached price");
    }

    #[test]
    fn test_detect_currency_german_stock() {
        assert_eq!(CurrencyService::detect_currency("SAP.DE"), "EUR");
        assert_eq!(CurrencyService::detect_currency("SIE.DE"), "EUR");
    }

    #[test]
    fn test_detect_currency_madrid_stock() {
        assert_eq!(CurrencyService::detect_currency("BBVA.MC"), "EUR");
        assert_eq!(CurrencyService::detect_currency("SAN.MC"), "EUR");
    }

    #[test]
    fn test_detect_currency_uk_stock() {
        assert_eq!(CurrencyService::detect_currency("SHEL.L"), "GBP");
        assert_eq!(CurrencyService::detect_currency("BP.L"), "GBP");
    }

    #[test]
    fn test_detect_currency_japanese_stock() {
        assert_eq!(CurrencyService::detect_currency("7203.T"), "JPY");
    }

    #[test]
    fn test_detect_currency_btc() {
        assert_eq!(CurrencyService::detect_currency("BTC-USD"), "USD");
    }

    #[test]
    fn test_detect_currency_us_stock() {
        assert_eq!(CurrencyService::detect_currency("AAPL"), "USD");
        assert_eq!(CurrencyService::detect_currency("MSFT"), "USD");
    }

    #[test]
    fn test_detect_currency_case_insensitive() {
        assert_eq!(CurrencyService::detect_currency("sap.de"), "EUR");
        assert_eq!(CurrencyService::detect_currency("shel.l"), "GBP");
    }
}

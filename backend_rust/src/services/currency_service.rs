use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, NaiveDate};
use reqwest::Client;
use serde::Deserialize;
use anyhow::{Result, anyhow};

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
    quote: Quote,
}

#[derive(Deserialize)]
struct Quote {
    close: Vec<f64>,
}

pub struct CurrencyService {
    client: Client,
    cache: Arc<RwLock<HashMap<(String, String, NaiveDate), f64>>>,
}

impl CurrencyService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
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

        let response = self.client.get(&url).send().await?.json::<YahooChartResponse>().await?;
        
        let result = response.chart.result.first()
            .ok_or_else(|| anyhow!("No result found for symbol {}", symbol))?;
        
        let close_prices = &result.indicators.quote.close;
        
        // For simplicity, we take the last available close price if we can't find the exact date.
        // In a more robust implementation, we'd match the timestamp.
        let rate = close_prices.last()
            .cloned()
            .ok_or_else(|| anyhow!("No close price found for symbol {}", symbol))?;

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, rate);

        Ok(rate)
    }

    pub fn detect_currency(symbol: &str) -> String {
        let upper = symbol.to_uppercase();
        if upper.ends_with(".DE") || upper.ends_with(".F") || upper.ends_with(".FR") {
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

    #[test]
    fn test_detect_currency_german_stock() {
        assert_eq!(CurrencyService::detect_currency("SAP.DE"), "EUR");
        assert_eq!(CurrencyService::detect_currency("SIE.DE"), "EUR");
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

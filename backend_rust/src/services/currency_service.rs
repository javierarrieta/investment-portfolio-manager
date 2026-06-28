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
}

mod university;
mod government;
mod third_party;
mod selenium;

use crate::types::{Source, Lead};
use anyhow::Result;

pub async fn scrape_source(source: &Source) -> Result<Vec<Lead>> {
    match source.scraper.as_str() {
        "selenium" => {
            // 使用 Selenium 爬蟲
            selenium::scrape_with_selenium(source.url.as_str()).await
        }
        "university" => {
            // 使用 spawn_blocking 包裝同步爬蟲
            let url = source.url.clone();
            tokio::task::spawn_blocking(move || university::scrape(&url))
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)))
        }
        "government" => {
            let url = source.url.clone();
            tokio::task::spawn_blocking(move || government::scrape(&url))
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)))
        }
        "third_party" => {
            let url = source.url.clone();
            tokio::task::spawn_blocking(move || third_party::scrape(&url))
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)))
        }
        _ => {
            eprintln!("Unknown scraper type: {}", source.scraper);
            Ok(vec![])
        }
    }
}

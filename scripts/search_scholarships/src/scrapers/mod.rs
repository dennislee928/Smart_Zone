mod university;
mod government;
mod third_party;
mod selenium;

pub use third_party::enrich_from_official;

use crate::types::{Source, Lead, ScrapeResult, SourceStatus};
use anyhow::Result;

/// Scrape a source and return detailed result for health tracking
pub async fn scrape_source(source: &Source) -> Result<ScrapeResult> {
    match source.scraper.as_str() {
        "selenium" => {
            // 使用 Selenium 爬蟲 - wrap in ScrapeResult
            match selenium::scrape_with_selenium(source.url.as_str()).await {
                Ok(leads) => Ok(ScrapeResult {
                    leads,
                    status: SourceStatus::Ok,
                    http_code: Some(200),
                    error_message: None,
                }),
                Err(e) => Ok(ScrapeResult {
                    leads: vec![],
                    status: SourceStatus::Unknown,
                    http_code: None,
                    error_message: Some(e.to_string()),
                }),
            }
        }
        "university" => {
            let url = source.url.clone();
            tokio::task::spawn_blocking(move || university::scrape(&url))
                .await
                .unwrap_or_else(|e| Ok(ScrapeResult {
                    leads: vec![],
                    status: SourceStatus::Unknown,
                    http_code: None,
                    error_message: Some(format!("Task join error: {}", e)),
                }))
        }
        "government" => {
            let url = source.url.clone();
            tokio::task::spawn_blocking(move || government::scrape(&url))
                .await
                .unwrap_or_else(|e| Ok(ScrapeResult {
                    leads: vec![],
                    status: SourceStatus::Unknown,
                    http_code: None,
                    error_message: Some(format!("Task join error: {}", e)),
                }))
        }
        "third_party" => {
            let url = source.url.clone();
            tokio::task::spawn_blocking(move || third_party::scrape(&url))
                .await
                .unwrap_or_else(|e| Ok(ScrapeResult {
                    leads: vec![],
                    status: SourceStatus::Unknown,
                    http_code: None,
                    error_message: Some(format!("Task join error: {}", e)),
                }))
        }
        _ => {
            eprintln!("Unknown scraper type: {}", source.scraper);
            Ok(ScrapeResult {
                leads: vec![],
                status: SourceStatus::Unknown,
                http_code: None,
                error_message: Some(format!("Unknown scraper type: {}", source.scraper)),
            })
        }
    }
}

/// Legacy function - returns only leads (for backward compatibility)
#[allow(dead_code)]
pub async fn scrape_source_leads_only(source: &Source) -> Result<Vec<Lead>> {
    let result = scrape_source(source).await?;
    Ok(result.leads)
}

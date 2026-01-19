//! Link Health Check Module
//! 
//! Checks URL validity and generates deadlinks.md report

use crate::types::{Lead, LinkHealthResult, LinkHealthStatus};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

/// Check health of all URLs in leads
pub async fn check_links(leads: &mut [Lead], max_concurrent: usize) -> Vec<LinkHealthResult> {
    let mut results: Vec<LinkHealthResult> = Vec::new();
    let mut url_cache: HashMap<String, LinkHealthResult> = HashMap::new();
    
    // Build client with reasonable timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (compatible; ScholarshipBot/1.0; +https://github.com)")
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    
    // Process in batches to avoid overwhelming servers
    for chunk in leads.chunks_mut(max_concurrent) {
        let mut futures = Vec::new();
        
        for lead in chunk.iter() {
            let url = lead.url.clone();
            
            // Skip if already checked
            if url_cache.contains_key(&url) {
                continue;
            }
            
            let client = client.clone();
            futures.push(async move {
                check_single_url(&client, &url).await
            });
        }
        
        // Execute batch
        let batch_results = futures::future::join_all(futures).await;
        
        // Update cache and leads
        for result in batch_results {
            url_cache.insert(result.url.clone(), result.clone());
            results.push(result);
        }
        
        // Small delay between batches to be polite
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Update leads with HTTP status
    for lead in leads.iter_mut() {
        if let Some(result) = url_cache.get(&lead.url) {
            lead.http_status = result.http_code;
        }
    }
    
    results
}

/// Check a single URL
async fn check_single_url(client: &reqwest::Client, url: &str) -> LinkHealthResult {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // Try HEAD request first (lighter)
    let response = client.head(url).send().await;
    
    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let final_url = resp.url().to_string();
            
            let status = match status_code {
                200..=299 => LinkHealthStatus::Ok,
                301 | 302 | 303 | 307 | 308 => LinkHealthStatus::Redirect,
                403 => LinkHealthStatus::Forbidden,
                404 | 410 => LinkHealthStatus::NotFound,
                429 => LinkHealthStatus::RateLimited,
                500..=599 => LinkHealthStatus::ServerError,
                _ => LinkHealthStatus::Unknown,
            };
            
            LinkHealthResult {
                url: url.to_string(),
                status,
                http_code: Some(status_code),
                final_url: if final_url != url { Some(final_url) } else { None },
                checked_at: timestamp,
                error_message: None,
            }
        }
        Err(e) => {
            let (status, error_msg) = if e.is_timeout() {
                (LinkHealthStatus::Timeout, "Request timed out")
            } else if e.is_connect() {
                (LinkHealthStatus::Unknown, "Connection failed")
            } else {
                (LinkHealthStatus::Unknown, "Request failed")
            };
            
            LinkHealthResult {
                url: url.to_string(),
                status,
                http_code: None,
                final_url: None,
                checked_at: timestamp,
                error_message: Some(format!("{}: {}", error_msg, e)),
            }
        }
    }
}

/// Generate deadlinks.md report
pub fn generate_deadlinks_report(results: &[LinkHealthResult]) -> String {
    let mut report = String::from("# Dead Links Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M UTC")));
    
    // Filter for problematic links
    let dead_links: Vec<_> = results.iter()
        .filter(|r| matches!(r.status, 
            LinkHealthStatus::NotFound | 
            LinkHealthStatus::Forbidden |
            LinkHealthStatus::ServerError |
            LinkHealthStatus::Timeout |
            LinkHealthStatus::Unknown
        ))
        .collect();
    
    let rate_limited: Vec<_> = results.iter()
        .filter(|r| matches!(r.status, LinkHealthStatus::RateLimited))
        .collect();
    
    let redirects: Vec<_> = results.iter()
        .filter(|r| matches!(r.status, LinkHealthStatus::Redirect))
        .collect();
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Total URLs checked: {}\n", results.len()));
    report.push_str(&format!("- Dead/Error links: {}\n", dead_links.len()));
    report.push_str(&format!("- Rate limited: {}\n", rate_limited.len()));
    report.push_str(&format!("- Redirects: {}\n", redirects.len()));
    report.push_str("\n");
    
    // Dead links table
    if !dead_links.is_empty() {
        report.push_str("## Dead/Error Links\n\n");
        report.push_str("| URL | Status | HTTP Code | Error | Checked At |\n");
        report.push_str("|-----|--------|-----------|-------|------------|\n");
        
        for link in &dead_links {
            let http_code = link.http_code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
            let error = link.error_message.as_deref().unwrap_or("-");
            // Truncate URL for readability
            let url_display = if link.url.len() > 60 {
                format!("{}...", &link.url[..57])
            } else {
                link.url.clone()
            };
            
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                url_display, link.status, http_code, error, link.checked_at
            ));
        }
        report.push_str("\n");
    }
    
    // Rate limited
    if !rate_limited.is_empty() {
        report.push_str("## Rate Limited (Needs Retry)\n\n");
        report.push_str("| URL | Checked At |\n");
        report.push_str("|-----|------------|\n");
        
        for link in &rate_limited {
            report.push_str(&format!("| {} | {} |\n", link.url, link.checked_at));
        }
        report.push_str("\n");
    }
    
    // Redirects
    if !redirects.is_empty() {
        report.push_str("## Redirects\n\n");
        report.push_str("| Original URL | Final URL |\n");
        report.push_str("|--------------|----------|\n");
        
        for link in &redirects {
            let final_url = link.final_url.as_deref().unwrap_or("-");
            report.push_str(&format!("| {} | {} |\n", link.url, final_url));
        }
        report.push_str("\n");
    }
    
    report
}

/// Quick check without full validation (for filtering)
pub fn is_likely_dead(lead: &Lead) -> bool {
    if let Some(status) = lead.http_status {
        matches!(status, 404 | 410 | 500..=599)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_report_empty() {
        let results: Vec<LinkHealthResult> = vec![];
        let report = generate_deadlinks_report(&results);
        assert!(report.contains("Total URLs checked: 0"));
    }
}

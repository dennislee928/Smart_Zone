//! Link Health Check Module
//! 
//! Checks URL validity and generates deadlinks.md report
//! 
//! Features:
//! - HEAD request with GET fallback for 403/405/429
//! - Exponential backoff with retry
//! - Proper classification: Dead (404/410), Transient (403/429/5xx), Redirect (301/302)

use crate::types::{Lead, LinkHealthResult, LinkHealthStatus};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

/// Maximum number of retries for transient errors
const MAX_RETRIES: u32 = 2;

/// Base delay for exponential backoff (milliseconds)
const BASE_BACKOFF_MS: u64 = 1000;

/// Check health of all URLs in leads
pub async fn check_links(leads: &mut [Lead], max_concurrent: usize) -> Vec<LinkHealthResult> {
    let mut results: Vec<LinkHealthResult> = Vec::new();
    let mut url_cache: HashMap<String, LinkHealthResult> = HashMap::new();
    
    // Build client with reasonable timeout and follow redirects
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(5))
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
                check_single_url_with_retry(&client, &url).await
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

/// Check a single URL with retry logic and HEAD->GET fallback
async fn check_single_url_with_retry(client: &reqwest::Client, url: &str) -> LinkHealthResult {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // Try HEAD request first (lighter)
    let head_result = client.head(url).send().await;
    
    match head_result {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let final_url = resp.url().to_string();
            
            // If HEAD returns 403/405/429/5xx, try GET fallback
            if should_fallback_to_get(status_code) {
                return check_with_get_fallback(client, url, &timestamp).await;
            }
            
            let status = classify_http_status(status_code);
            
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
            // Network error on HEAD - try GET as some servers don't support HEAD
            if e.is_connect() || e.is_timeout() {
                return check_with_get_fallback(client, url, &timestamp).await;
            }
            
            create_error_result(url, &timestamp, &e)
        }
    }
}

/// Check if we should fallback to GET request
fn should_fallback_to_get(status_code: u16) -> bool {
    matches!(status_code, 403 | 405 | 429 | 500..=599)
}

/// Fallback to GET request with optional retry
async fn check_with_get_fallback(client: &reqwest::Client, url: &str, timestamp: &str) -> LinkHealthResult {
    let mut last_result = None;
    
    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            // Exponential backoff: 1s, 2s, 4s...
            let delay = BASE_BACKOFF_MS * (1 << (attempt - 1));
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
        
        // Use GET with range header to minimize data transfer
        let response = client
            .get(url)
            .header("Range", "bytes=0-1023")  // Request only first 1KB
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let final_url = resp.url().to_string();
                
                // Check for Retry-After header on 429/503
                if status_code == 429 || status_code == 503 {
                    if let Some(retry_after) = resp.headers().get("retry-after") {
                        if let Ok(seconds) = retry_after.to_str().unwrap_or("0").parse::<u64>() {
                            if seconds <= 60 && attempt < MAX_RETRIES {
                                // Wait and retry if Retry-After is reasonable
                                tokio::time::sleep(Duration::from_secs(seconds.min(30))).await;
                                continue;
                            }
                        }
                    }
                }
                
                // For transient errors, continue retrying
                if is_transient_error(status_code) && attempt < MAX_RETRIES {
                    last_result = Some(LinkHealthResult {
                        url: url.to_string(),
                        status: classify_http_status(status_code),
                        http_code: Some(status_code),
                        final_url: if final_url != url { Some(final_url) } else { None },
                        checked_at: timestamp.to_string(),
                        error_message: Some(format!("Transient error, attempt {}/{}", attempt + 1, MAX_RETRIES + 1)),
                    });
                    continue;
                }
                
                // Final result
                return LinkHealthResult {
                    url: url.to_string(),
                    status: classify_http_status(status_code),
                    http_code: Some(status_code),
                    final_url: if final_url != url { Some(final_url) } else { None },
                    checked_at: timestamp.to_string(),
                    error_message: None,
                };
            }
            Err(e) => {
                if attempt < MAX_RETRIES && (e.is_timeout() || e.is_connect()) {
                    last_result = Some(create_error_result(url, timestamp, &e));
                    continue;
                }
                return create_error_result(url, timestamp, &e);
            }
        }
    }
    
    // Return last result if all retries exhausted
    last_result.unwrap_or_else(|| LinkHealthResult {
        url: url.to_string(),
        status: LinkHealthStatus::Unknown,
        http_code: None,
        final_url: None,
        checked_at: timestamp.to_string(),
        error_message: Some("Max retries exhausted".to_string()),
    })
}

/// Check if status code indicates a transient error worth retrying
fn is_transient_error(status_code: u16) -> bool {
    matches!(status_code, 429 | 500 | 502 | 503 | 504)
}

/// Classify HTTP status code into LinkHealthStatus
/// 
/// Classification:
/// - Dead: 404/410 (confirmed not found after GET verification)
/// - Transient: 403/429/5xx (should retry with backoff)
/// - Redirect: 301/302/303/307/308 (follow to canonical URL)
/// - Ok: 200-299 (healthy)
fn classify_http_status(status_code: u16) -> LinkHealthStatus {
    match status_code {
        200..=299 | 206 => LinkHealthStatus::Ok,  // 206 = Partial Content (from Range request)
        301 | 302 | 303 | 307 | 308 => LinkHealthStatus::Redirect,
        403 => LinkHealthStatus::Forbidden,  // Transient - may be blocking bots
        404 | 410 => LinkHealthStatus::NotFound,  // True dead link
        429 => LinkHealthStatus::RateLimited,  // Transient - retry with backoff
        500..=599 => LinkHealthStatus::ServerError,  // Transient - server issue
        _ => LinkHealthStatus::Unknown,
    }
}

/// Create an error result from a reqwest error
fn create_error_result(url: &str, timestamp: &str, e: &reqwest::Error) -> LinkHealthResult {
    let (status, error_msg) = if e.is_timeout() {
        (LinkHealthStatus::Timeout, "Request timed out")
    } else if e.is_connect() {
        (LinkHealthStatus::Unknown, "Connection failed")
    } else if e.is_redirect() {
        (LinkHealthStatus::Redirect, "Too many redirects")
    } else {
        (LinkHealthStatus::Unknown, "Request failed")
    };
    
    LinkHealthResult {
        url: url.to_string(),
        status,
        http_code: None,
        final_url: None,
        checked_at: timestamp.to_string(),
        error_message: Some(format!("{}: {}", error_msg, e)),
    }
}

/// Check if a link health result represents a true dead link
/// Only 404/410 after GET verification are considered dead
pub fn is_true_dead_link(result: &LinkHealthResult) -> bool {
    matches!(result.status, LinkHealthStatus::NotFound)
}

/// Check if a link health result represents a transient issue (should retry later)
pub fn is_transient_issue(result: &LinkHealthResult) -> bool {
    matches!(result.status, 
        LinkHealthStatus::Forbidden | 
        LinkHealthStatus::RateLimited | 
        LinkHealthStatus::ServerError |
        LinkHealthStatus::Timeout
    )
}

/// Generate deadlinks.md report
/// 
/// Report structure:
/// - True Dead Links: Only 404/410 after GET verification
/// - Transient Issues: 403/429/5xx that may recover with retry
/// - Redirects: 301/302 with final URLs
pub fn generate_deadlinks_report(results: &[LinkHealthResult]) -> String {
    let mut report = String::from("# Dead Links Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M UTC")));
    
    // Categorize links properly
    let true_dead_links: Vec<_> = results.iter()
        .filter(|r| is_true_dead_link(r))
        .collect();
    
    let transient_issues: Vec<_> = results.iter()
        .filter(|r| is_transient_issue(r))
        .collect();
    
    let rate_limited: Vec<_> = results.iter()
        .filter(|r| matches!(r.status, LinkHealthStatus::RateLimited))
        .collect();
    
    let redirects: Vec<_> = results.iter()
        .filter(|r| matches!(r.status, LinkHealthStatus::Redirect))
        .collect();
    
    let healthy: Vec<_> = results.iter()
        .filter(|r| matches!(r.status, LinkHealthStatus::Ok))
        .collect();
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Total URLs checked: {}\n", results.len()));
    report.push_str(&format!("- Healthy (200-299): {}\n", healthy.len()));
    report.push_str(&format!("- **True Dead (404/410)**: {}\n", true_dead_links.len()));
    report.push_str(&format!("- Transient (403/5xx/Timeout): {}\n", transient_issues.len()));
    report.push_str(&format!("- Rate limited (429): {}\n", rate_limited.len()));
    report.push_str(&format!("- Redirects (301/302): {}\n", redirects.len()));
    report.push_str("\n");
    
    // True Dead Links (404/410 only)
    if !true_dead_links.is_empty() {
        report.push_str("## True Dead Links (Confirmed 404/410)\n\n");
        report.push_str("These URLs are confirmed dead after GET verification:\n\n");
        report.push_str("| URL | HTTP Code | Checked At |\n");
        report.push_str("|-----|-----------|------------|\n");
        
        for link in &true_dead_links {
            let http_code = link.http_code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
            let url_display = truncate_url(&link.url, 60);
            
            report.push_str(&format!(
                "| {} | {} | {} |\n",
                url_display, http_code, link.checked_at
            ));
        }
        report.push_str("\n");
    }
    
    // Transient Issues (may recover)
    if !transient_issues.is_empty() {
        report.push_str("## Transient Issues (May Recover)\n\n");
        report.push_str("These URLs had temporary issues - retry with backoff:\n\n");
        report.push_str("| URL | Status | HTTP Code | Error | Checked At |\n");
        report.push_str("|-----|--------|-----------|-------|------------|\n");
        
        for link in &transient_issues {
            let http_code = link.http_code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
            let error = link.error_message.as_deref().unwrap_or("-");
            let url_display = truncate_url(&link.url, 50);
            
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                url_display, link.status, http_code, error, link.checked_at
            ));
        }
        report.push_str("\n");
    }
    
    // Rate limited (separate category for visibility)
    if !rate_limited.is_empty() {
        report.push_str("## Rate Limited (429 - Needs Backoff)\n\n");
        report.push_str("These URLs returned 429 - implement exponential backoff:\n\n");
        report.push_str("| URL | Checked At |\n");
        report.push_str("|-----|------------|\n");
        
        for link in &rate_limited {
            report.push_str(&format!("| {} | {} |\n", link.url, link.checked_at));
        }
        report.push_str("\n");
    }
    
    // Redirects
    if !redirects.is_empty() {
        report.push_str("## Redirects (Update Canonical URLs)\n\n");
        report.push_str("| Original URL | Final URL |\n");
        report.push_str("|--------------|----------|\n");
        
        for link in &redirects {
            let final_url = link.final_url.as_deref().unwrap_or("-");
            let orig_display = truncate_url(&link.url, 40);
            let final_display = truncate_url(final_url, 40);
            report.push_str(&format!("| {} | {} |\n", orig_display, final_display));
        }
        report.push_str("\n");
    }
    
    report
}

/// Truncate URL for display purposes
fn truncate_url(url: &str, max_len: usize) -> String {
    if url.chars().count() > max_len {
        format!("{}...", url.chars().take(max_len - 3).collect::<String>())
    } else {
        url.to_string()
    }
}

/// Quick check without full validation (for filtering)
/// Only considers 404/410 as truly dead - not transient errors
pub fn is_likely_dead(lead: &Lead) -> bool {
    if let Some(status) = lead.http_status {
        // Only 404/410 are true dead links
        matches!(status, 404 | 410)
    } else {
        false
    }
}

/// Check if a lead has a transient HTTP issue (may recover)
pub fn has_transient_issue(lead: &Lead) -> bool {
    if let Some(status) = lead.http_status {
        matches!(status, 403 | 429 | 500..=599)
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
    
    #[test]
    fn test_classify_http_status() {
        assert!(matches!(classify_http_status(200), LinkHealthStatus::Ok));
        assert!(matches!(classify_http_status(206), LinkHealthStatus::Ok));
        assert!(matches!(classify_http_status(301), LinkHealthStatus::Redirect));
        assert!(matches!(classify_http_status(404), LinkHealthStatus::NotFound));
        assert!(matches!(classify_http_status(410), LinkHealthStatus::NotFound));
        assert!(matches!(classify_http_status(403), LinkHealthStatus::Forbidden));
        assert!(matches!(classify_http_status(429), LinkHealthStatus::RateLimited));
        assert!(matches!(classify_http_status(500), LinkHealthStatus::ServerError));
    }
    
    #[test]
    fn test_is_true_dead_link() {
        let dead_result = LinkHealthResult {
            url: "https://example.com/404".to_string(),
            status: LinkHealthStatus::NotFound,
            http_code: Some(404),
            final_url: None,
            checked_at: "2026-01-21".to_string(),
            error_message: None,
        };
        assert!(is_true_dead_link(&dead_result));
        
        let transient_result = LinkHealthResult {
            url: "https://example.com/503".to_string(),
            status: LinkHealthStatus::ServerError,
            http_code: Some(503),
            final_url: None,
            checked_at: "2026-01-21".to_string(),
            error_message: None,
        };
        assert!(!is_true_dead_link(&transient_result));
    }
    
    #[test]
    fn test_is_transient_issue() {
        let rate_limited = LinkHealthResult {
            url: "https://example.com".to_string(),
            status: LinkHealthStatus::RateLimited,
            http_code: Some(429),
            final_url: None,
            checked_at: "2026-01-21".to_string(),
            error_message: None,
        };
        assert!(is_transient_issue(&rate_limited));
        
        let forbidden = LinkHealthResult {
            url: "https://example.com".to_string(),
            status: LinkHealthStatus::Forbidden,
            http_code: Some(403),
            final_url: None,
            checked_at: "2026-01-21".to_string(),
            error_message: None,
        };
        assert!(is_transient_issue(&forbidden));
    }
    
    #[test]
    fn test_should_fallback_to_get() {
        assert!(should_fallback_to_get(403));
        assert!(should_fallback_to_get(405));
        assert!(should_fallback_to_get(429));
        assert!(should_fallback_to_get(500));
        assert!(should_fallback_to_get(503));
        assert!(!should_fallback_to_get(200));
        assert!(!should_fallback_to_get(404));
    }
    
    #[test]
    fn test_truncate_url() {
        assert_eq!(truncate_url("https://example.com", 50), "https://example.com");
        assert_eq!(truncate_url("https://example.com/very/long/path/that/exceeds/limit", 30), "https://example.com/very/lon...");
    }
}

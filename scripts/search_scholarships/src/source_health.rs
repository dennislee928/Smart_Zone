//! Source Health Tracking Module
//! 
//! Tracks the health status of each source URL across runs.
//! Auto-disables sources that fail consecutively with cooldown mechanism.
//! Per-domain politeness controls and error taxonomy.

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, Context};
use chrono::{Utc, DateTime, Duration};

use crate::types::{SourceHealth, SourceHealthFile, SourceStatus, Source, SourceFilterConfig, ScrapeResult};

const HEALTH_FILE: &str = "tracking/source_health.json";
const COOLDOWN_HOURS: i64 = 24; // 24-hour cooldown after auto-disable

/// Error taxonomy for source health tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Blocked,          // 403 Forbidden
    RateLimited,      // 429 Too Many Requests
    Timeout,          // Request timeout
    ParseError,       // HTML parsing failed
    RobotsDisallow,   // robots.txt disallows
    ServerError,      // 5xx errors
    NotFound,         // 404 Not Found
    NetworkError,     // Network/connection errors
    Unknown,          // Unknown error
}

impl ErrorCategory {
    pub fn from_status(status: SourceStatus) -> Self {
        match status {
            SourceStatus::Forbidden => ErrorCategory::Blocked,
            SourceStatus::RateLimited => ErrorCategory::RateLimited,
            SourceStatus::Timeout => ErrorCategory::Timeout,
            SourceStatus::ServerError => ErrorCategory::ServerError,
            SourceStatus::NotFound => ErrorCategory::NotFound,
            SourceStatus::NetworkError => ErrorCategory::NetworkError,
            SourceStatus::Unknown => ErrorCategory::Unknown,
            SourceStatus::Ok => ErrorCategory::Unknown, // Should not happen
            SourceStatus::SslError => ErrorCategory::NetworkError,
            SourceStatus::TooManyRedirects => ErrorCategory::NetworkError,
        }
    }
}

/// Per-domain politeness configuration
#[derive(Debug, Clone)]
pub struct DomainPoliteness {
    pub domain: String,
    pub min_delay_ms: u64,
    pub max_concurrency: usize,
    pub retry_backoff_ms: u64,
    pub max_retries: u32,
}

impl Default for DomainPoliteness {
    fn default() -> Self {
        Self {
            domain: String::new(),
            min_delay_ms: 1000,      // 1 second default delay
            max_concurrency: 2,      // Max 2 concurrent requests per domain
            retry_backoff_ms: 2000,  // 2 seconds backoff
            max_retries: 3,
        }
    }
}

/// Source health statistics for dashboard
#[derive(Debug, Clone, Default)]
pub struct SourceHealthStats {
    pub unique_found: usize,
    pub dup_rate: f32,
    pub missing_deadline_rate: f32,
    pub blocked_rate: f32,
    pub total_attempts: u32,
    pub total_successes: u32,
    pub error_counts: HashMap<ErrorCategory, u32>,
}

/// Load source health data from file
pub fn load_health(root: &str) -> Result<SourceHealthFile> {
    let path = PathBuf::from(root).join(HEALTH_FILE);
    
    if !path.exists() {
        return Ok(SourceHealthFile {
            last_updated: Utc::now().to_rfc3339(),
            sources: vec![],
        });
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read source health from {:?}", path))?;
    
    let health: SourceHealthFile = serde_json::from_str(&content)
        .unwrap_or_else(|_| SourceHealthFile {
            last_updated: Utc::now().to_rfc3339(),
            sources: vec![],
        });
    
    Ok(health)
}

/// Save source health data to file
pub fn save_health(root: &str, health: &SourceHealthFile) -> Result<()> {
    let path = PathBuf::from(root).join(HEALTH_FILE);
    let json = serde_json::to_string_pretty(health)?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write source health to {:?}", path))?;
    Ok(())
}

/// Update health record for a source after scraping with cooldown support
pub fn update_health(
    health_file: &mut SourceHealthFile,
    source: &Source,
    result: &ScrapeResult,
    max_failures: u32,
) {
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    
    // Find or create health record
    let health = health_file.sources.iter_mut()
        .find(|h| h.url == source.url);
    
    if let Some(h) = health {
        // Check if source is in cooldown period
        if h.auto_disabled {
            if let Some(disabled_time_str) = &h.disabled_reason {
                // Try to extract timestamp from disabled_reason or use last_checked
                if let Ok(disabled_time) = DateTime::parse_from_rfc3339(&h.last_checked) {
                    let disabled_time_utc = disabled_time.with_timezone(&Utc);
                    let cooldown_end = disabled_time_utc + Duration::hours(COOLDOWN_HOURS);
                    
                    if now < cooldown_end {
                        // Still in cooldown, skip update
                        return;
                    } else {
                        // Cooldown expired, re-enable
                        h.auto_disabled = false;
                        h.disabled_reason = None;
                        h.consecutive_failures = 0;
                    }
                }
            }
        }
        
        // Update existing record
        h.total_attempts += 1;
        h.last_checked = now_str.clone();
        h.last_status = result.status;
        h.last_http_code = result.http_code;
        h.last_error = result.error_message.clone();
        
        if result.status == SourceStatus::Ok {
            h.consecutive_failures = 0;
            h.total_successes += 1;
            h.auto_disabled = false;
            h.disabled_reason = None;
            h.fallback_strategies.clear(); // Clear fallback strategies on success
        } else {
            // Don't immediately increase consecutive_failures if we have fallback strategies
            if result.status == SourceStatus::Forbidden {
                // 403 - try fallback strategies instead of immediately disabling
                if h.fallback_strategies.is_empty() {
                    h.fallback_strategies = vec!["sitemap".to_string(), "rss".to_string()];
                }
                // Only increase consecutive_failures if all fallback strategies have been tried
                // For now, we'll still track failures but won't auto-disable until fallbacks fail
                h.consecutive_failures += 1;
            } else if result.status == SourceStatus::Timeout {
                // Timeout - try fallback strategies
                if h.fallback_strategies.is_empty() {
                    h.fallback_strategies = vec!["sitemap".to_string(), "rss".to_string(), "head_request".to_string()];
                }
                h.consecutive_failures += 1;
            } else {
                // Other errors - increase failures normally
                h.consecutive_failures += 1;
            }
            
            // Auto-disable only if too many consecutive failures AND no active fallback strategies
            if h.consecutive_failures >= max_failures && !h.auto_disabled && h.fallback_strategies.is_empty() {
                h.auto_disabled = true;
                let error_cat = ErrorCategory::from_status(result.status);
                h.disabled_reason = Some(format!(
                    "Auto-disabled after {} consecutive failures. Error: {:?}. Cooldown: {} hours",
                    h.consecutive_failures,
                    error_cat,
                    COOLDOWN_HOURS
                ));
            }
        }
    } else {
        // Create new record
        let is_success = result.status == SourceStatus::Ok;
        let consecutive_failures = if is_success { 0 } else { 1 };
        let auto_disabled = consecutive_failures >= max_failures;
        
        health_file.sources.push(SourceHealth {
            url: source.url.clone(),
            name: source.name.clone(),
            source_type: source.source_type.clone(),
            consecutive_failures,
            total_attempts: 1,
            total_successes: if is_success { 1 } else { 0 },
            last_status: result.status,
            last_http_code: result.http_code,
            last_error: result.error_message.clone(),
            last_checked: now_str,
            auto_disabled,
            disabled_reason: if auto_disabled {
                let error_cat = ErrorCategory::from_status(result.status);
                Some(format!("Auto-disabled on first failure: {:?}. Cooldown: {} hours", error_cat, COOLDOWN_HOURS))
            } else {
                None
            },
            fallback_strategies: if !is_success {
                match result.status {
                    SourceStatus::Forbidden => vec!["sitemap".to_string(), "rss".to_string()],
                    SourceStatus::Timeout => vec!["sitemap".to_string(), "rss".to_string(), "head_request".to_string()],
                    _ => vec![],
                }
            } else {
                vec![]
            },
        });
    }
    
    health_file.last_updated = Utc::now().to_rfc3339();
}

/// Get per-domain politeness configuration
pub fn get_domain_politeness(domain: &str) -> DomainPoliteness {
    // Default politeness settings per domain
    match domain {
        "gla.ac.uk" | "glasgow.ac.uk" => DomainPoliteness {
            domain: domain.to_string(),
            min_delay_ms: 500,      // Faster for trusted sources
            max_concurrency: 3,
            retry_backoff_ms: 1000,
            max_retries: 5,
        },
        "gov.uk" | "ukri.org" => DomainPoliteness {
            domain: domain.to_string(),
            min_delay_ms: 2000,     // More conservative for government sites
            max_concurrency: 1,
            retry_backoff_ms: 5000,
            max_retries: 3,
        },
        _ => DomainPoliteness {
            domain: domain.to_string(),
            min_delay_ms: 1000,     // Default: 1 second
            max_concurrency: 2,
            retry_backoff_ms: 2000,
            max_retries: 3,
        },
    }
}

/// Calculate source health statistics for dashboard
pub fn calculate_source_stats(
    health_file: &SourceHealthFile,
    source_url: &str,
    leads_found: usize,
    duplicates: usize,
    missing_deadline: usize,
) -> SourceHealthStats {
    let mut stats = SourceHealthStats::default();
    
    if let Some(health) = health_file.sources.iter().find(|h| h.url == source_url) {
        stats.total_attempts = health.total_attempts;
        stats.total_successes = health.total_successes;
        
        // Calculate rates
        if leads_found > 0 {
            stats.dup_rate = (duplicates as f32) / (leads_found as f32) * 100.0;
            stats.missing_deadline_rate = (missing_deadline as f32) / (leads_found as f32) * 100.0;
        }
        
        stats.unique_found = leads_found - duplicates;
        
        // Calculate blocked rate
        if health.total_attempts > 0 {
            let blocked_count = if health.last_status == SourceStatus::Forbidden || 
                                  health.last_status == SourceStatus::RateLimited {
                1
            } else {
                0
            };
            stats.blocked_rate = (blocked_count as f32) / (health.total_attempts as f32) * 100.0;
        }
        
        // Error counts by category
        let error_cat = ErrorCategory::from_status(health.last_status);
        *stats.error_counts.entry(error_cat).or_insert(0) += 1;
    }
    
    stats
}

/// Check if a source should be skipped based on health and filter config
pub fn should_skip_source(
    source: &Source,
    health_file: &SourceHealthFile,
    filter: &SourceFilterConfig,
) -> Option<String> {
    // Check type filters
    if !filter.include_types.is_empty() && !filter.include_types.contains(&source.source_type) {
        return Some(format!("Type '{}' not in include list", source.source_type));
    }
    
    if filter.exclude_types.contains(&source.source_type) {
        return Some(format!("Type '{}' is excluded", source.source_type));
    }
    
    // Check auto-disabled status
    if filter.skip_auto_disabled {
        if let Some(health) = health_file.sources.iter().find(|h| h.url == source.url) {
            if health.auto_disabled {
                return Some(format!(
                    "Auto-disabled: {}",
                    health.disabled_reason.as_deref().unwrap_or("unknown reason")
                ));
            }
        }
    }
    
    None
}

/// Generate enhanced health report with dashboard summary
pub fn generate_health_report(health_file: &SourceHealthFile) -> String {
    let mut report = String::from("# Source Health Report\n\n");
    report.push_str(&format!("**Last Updated:** {}\n\n", health_file.last_updated));
    
    // Summary stats
    let total = health_file.sources.len();
    let healthy = health_file.sources.iter().filter(|h| h.last_status == SourceStatus::Ok).count();
    let disabled = health_file.sources.iter().filter(|h| h.auto_disabled).count();
    let failing = health_file.sources.iter().filter(|h| h.consecutive_failures > 0 && !h.auto_disabled).count();
    
    // Error taxonomy counts
    let mut error_counts: HashMap<ErrorCategory, usize> = HashMap::new();
    for h in &health_file.sources {
        if h.last_status != SourceStatus::Ok {
            let cat = ErrorCategory::from_status(h.last_status);
            *error_counts.entry(cat).or_insert(0) += 1;
        }
    }
    
    report.push_str("## Summary\n\n");
    report.push_str(&format!("| Status | Count |\n"));
    report.push_str(&format!("|--------|-------|\n"));
    report.push_str(&format!("| Total Sources | {} |\n", total));
    report.push_str(&format!("| Healthy | {} |\n", healthy));
    report.push_str(&format!("| Failing | {} |\n", failing));
    report.push_str(&format!("| Auto-Disabled | {} |\n", disabled));
    report.push_str("\n");
    
    // Error taxonomy
    if !error_counts.is_empty() {
        report.push_str("## Error Taxonomy\n\n");
        report.push_str("| Error Type | Count |\n");
        report.push_str("|------------|-------|\n");
        for (cat, count) in error_counts.iter() {
            report.push_str(&format!("| {:?} | {} |\n", cat, count));
        }
        report.push_str("\n");
    }
    
    // Group by status
    let mut by_status: HashMap<SourceStatus, Vec<&SourceHealth>> = HashMap::new();
    for h in &health_file.sources {
        by_status.entry(h.last_status).or_default().push(h);
    }
    
    // Auto-disabled sources
    if disabled > 0 {
        report.push_str("## Auto-Disabled Sources\n\n");
        report.push_str("| Source | Type | Failures | Last Error |\n");
        report.push_str("|--------|------|----------|------------|\n");
        
        for h in health_file.sources.iter().filter(|h| h.auto_disabled) {
            let name = if h.name.chars().count() > 30 { 
                format!("{}...", h.name.chars().take(27).collect::<String>()) 
            } else { 
                h.name.clone() 
            };
            let error = h.last_error.as_deref().unwrap_or("-");
            let error_short = if error.chars().count() > 40 { 
                format!("{}...", error.chars().take(37).collect::<String>()) 
            } else { 
                error.to_string() 
            };
            report.push_str(&format!("| {} | {} | {} | {} |\n", 
                name, h.source_type, h.consecutive_failures, error_short));
        }
        report.push_str("\n");
    }
    
    // Failing sources (not yet disabled)
    if failing > 0 {
        report.push_str("## Failing Sources (Not Yet Disabled)\n\n");
        report.push_str("| Source | Type | Failures | Last Status |\n");
        report.push_str("|--------|------|----------|-------------|\n");
        
        for h in health_file.sources.iter().filter(|h| h.consecutive_failures > 0 && !h.auto_disabled) {
            let name = if h.name.chars().count() > 30 { 
                format!("{}...", h.name.chars().take(27).collect::<String>()) 
            } else { 
                h.name.clone() 
            };
            report.push_str(&format!("| {} | {} | {} | {} |\n", 
                name, h.source_type, h.consecutive_failures, h.last_status));
        }
        report.push_str("\n");
    }
    
    // By source type
    report.push_str("## By Source Type\n\n");
    let mut type_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
    for h in &health_file.sources {
        let entry = type_stats.entry(h.source_type.clone()).or_insert((0, 0, 0));
        entry.0 += 1; // total
        if h.last_status == SourceStatus::Ok { entry.1 += 1; } // healthy
        if h.auto_disabled { entry.2 += 1; } // disabled
    }
    
    report.push_str("| Type | Total | Healthy | Disabled |\n");
    report.push_str("|------|-------|---------|----------|\n");
    for (source_type, (total, healthy, disabled)) in type_stats.iter() {
        report.push_str(&format!("| {} | {} | {} | {} |\n", source_type, total, healthy, disabled));
    }
    
    report
}

/// Re-enable a source that was auto-disabled
#[allow(dead_code)]
pub fn reenable_source(health_file: &mut SourceHealthFile, url: &str) -> bool {
    if let Some(h) = health_file.sources.iter_mut().find(|h| h.url == url) {
        h.auto_disabled = false;
        h.disabled_reason = None;
        h.consecutive_failures = 0;
        true
    } else {
        false
    }
}

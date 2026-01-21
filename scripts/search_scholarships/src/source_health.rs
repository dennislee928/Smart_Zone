//! Source Health Tracking Module
//! 
//! Tracks the health status of each source URL across runs.
//! Auto-disables sources that fail consecutively.

use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, Context};
use chrono::Utc;

use crate::types::{SourceHealth, SourceHealthFile, SourceStatus, Source, SourceFilterConfig, ScrapeResult};

const HEALTH_FILE: &str = "tracking/source_health.json";

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

/// Update health record for a source after scraping
pub fn update_health(
    health_file: &mut SourceHealthFile,
    source: &Source,
    result: &ScrapeResult,
    max_failures: u32,
) {
    let now = Utc::now().to_rfc3339();
    
    // Find or create health record
    let health = health_file.sources.iter_mut()
        .find(|h| h.url == source.url);
    
    if let Some(h) = health {
        // Update existing record
        h.total_attempts += 1;
        h.last_checked = now.clone();
        h.last_status = result.status;
        h.last_http_code = result.http_code;
        h.last_error = result.error_message.clone();
        
        if result.status == SourceStatus::Ok {
            h.consecutive_failures = 0;
            h.total_successes += 1;
            h.auto_disabled = false;
            h.disabled_reason = None;
        } else {
            h.consecutive_failures += 1;
            
            // Auto-disable if too many consecutive failures
            if h.consecutive_failures >= max_failures && !h.auto_disabled {
                h.auto_disabled = true;
                h.disabled_reason = Some(format!(
                    "Auto-disabled after {} consecutive failures. Last error: {}",
                    h.consecutive_failures,
                    result.status
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
            last_checked: now,
            auto_disabled,
            disabled_reason: if auto_disabled {
                Some(format!("Auto-disabled on first failure: {}", result.status))
            } else {
                None
            },
        });
    }
    
    health_file.last_updated = Utc::now().to_rfc3339();
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

/// Generate health report markdown
pub fn generate_health_report(health_file: &SourceHealthFile) -> String {
    let mut report = String::from("# Source Health Report\n\n");
    report.push_str(&format!("**Last Updated:** {}\n\n", health_file.last_updated));
    
    // Summary stats
    let total = health_file.sources.len();
    let healthy = health_file.sources.iter().filter(|h| h.last_status == SourceStatus::Ok).count();
    let disabled = health_file.sources.iter().filter(|h| h.auto_disabled).count();
    let failing = health_file.sources.iter().filter(|h| h.consecutive_failures > 0 && !h.auto_disabled).count();
    
    report.push_str("## Summary\n\n");
    report.push_str(&format!("| Status | Count |\n"));
    report.push_str(&format!("|--------|-------|\n"));
    report.push_str(&format!("| Total Sources | {} |\n", total));
    report.push_str(&format!("| Healthy | {} |\n", healthy));
    report.push_str(&format!("| Failing | {} |\n", failing));
    report.push_str(&format!("| Auto-Disabled | {} |\n", disabled));
    report.push_str("\n");
    
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

//! URL Normalization and Deduplication Module
//!
//! Provides functions to:
//! - Normalize URLs (https, lowercase host, remove tracking params)
//! - Deduplicate leads based on canonical URL or content signature
//! - Follow redirects to get final canonical URLs

use crate::types::Lead;
use std::collections::{HashMap, HashSet};

/// Tracking parameters to remove from URLs
const TRACKING_PARAMS: &[&str] = &[
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "utm_id", "utm_source_platform", "utm_creative_format",
    "gclid", "gclsrc",           // Google Ads
    "fbclid",                     // Facebook
    "msclkid",                    // Microsoft/Bing
    "dclid",                      // DoubleClick
    "mc_cid", "mc_eid",          // Mailchimp
    "ref", "referrer",           // Generic referrer
    "source", "src",             // Generic source
    "sessionid", "session_id",   // Session tracking
    "sid", "PHPSESSID",          // PHP sessions
    "_ga", "_gl",                // Google Analytics
    "affiliate", "aff_id",       // Affiliate tracking
];

/// Normalize a URL for canonical comparison
/// 
/// Normalization rules:
/// 1. Force HTTPS
/// 2. Lowercase hostname
/// 3. Remove tracking parameters
/// 4. Normalize trailing slash (remove for non-root paths)
/// 5. Sort remaining query parameters for consistency
pub fn normalize_url(url: &str) -> String {
    let url = url.trim();
    
    // Handle empty or invalid URLs
    if url.is_empty() || !url.contains("://") {
        return url.to_string();
    }
    
    // Parse URL manually (avoid external URL parsing crate dependency)
    let (scheme, rest) = if let Some(pos) = url.find("://") {
        (&url[..pos], &url[pos + 3..])
    } else {
        return url.to_string();
    };
    
    // Force HTTPS
    let scheme = if scheme.to_lowercase() == "http" { "https" } else { scheme };
    
    // Split host and path
    let (host_port, path_query) = if let Some(pos) = rest.find('/') {
        (&rest[..pos], &rest[pos..])
    } else {
        (rest, "/")
    };
    
    // Lowercase hostname (preserve port)
    let host_port_lower = host_port.to_lowercase();
    
    // Split path and query
    let (path, query) = if let Some(pos) = path_query.find('?') {
        (&path_query[..pos], Some(&path_query[pos + 1..]))
    } else {
        (path_query, None)
    };
    
    // Remove fragment
    let path = if let Some(pos) = path.find('#') {
        &path[..pos]
    } else {
        path
    };
    
    // Normalize path: remove trailing slash for non-root paths
    let path = if path.len() > 1 && path.ends_with('/') {
        &path[..path.len() - 1]
    } else if path.is_empty() {
        "/"
    } else {
        path
    };
    
    // Filter and sort query parameters
    let normalized_query = if let Some(q) = query {
        let filtered: Vec<(String, String)> = q
            .split('&')
            .filter_map(|param| {
                let parts: Vec<&str> = param.splitn(2, '=').collect();
                if parts.is_empty() || parts[0].is_empty() {
                    return None;
                }
                let key = parts[0].to_lowercase();
                // Skip tracking parameters
                if TRACKING_PARAMS.iter().any(|&tp| key == tp.to_lowercase()) {
                    return None;
                }
                let value = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
                Some((key, value))
            })
            .collect();
        
        if filtered.is_empty() {
            None
        } else {
            // Sort for consistency
            let mut sorted = filtered;
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            Some(sorted.iter()
                .map(|(k, v)| if v.is_empty() { k.clone() } else { format!("{}={}", k, v) })
                .collect::<Vec<_>>()
                .join("&"))
        }
    } else {
        None
    };
    
    // Rebuild URL
    if let Some(q) = normalized_query {
        format!("{}://{}{}?{}", scheme, host_port_lower, path, q)
    } else {
        format!("{}://{}{}", scheme, host_port_lower, path)
    }
}

/// Generate a deduplication key for a lead
/// 
/// Key is based on:
/// 1. Primary: canonical_url (if available)
/// 2. Secondary: normalized url
/// 3. Fallback: (title, sponsor, country, deadline_date) signature
pub fn generate_dedup_key(lead: &Lead) -> String {
    // Use canonical URL if available
    if let Some(ref canonical) = lead.canonical_url {
        return normalize_url(canonical);
    }
    
    // Use normalized URL
    let normalized = normalize_url(&lead.url);
    if !normalized.is_empty() && normalized != lead.url {
        return normalized;
    }
    
    // Fallback: content-based signature
    let name_normalized = normalize_text(&lead.name);
    let amount_normalized = normalize_text(&lead.amount);
    let deadline = lead.deadline_date.as_deref().unwrap_or(&lead.deadline);
    
    format!("{}|{}|{}", name_normalized, amount_normalized, deadline)
}

/// Normalize text for comparison (lowercase, remove extra whitespace)
fn normalize_text(text: &str) -> String {
    // Simple whitespace normalization without regex for performance
    text.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Deduplicate leads and return unique leads with best quality
/// 
/// When duplicates are found:
/// - Keep the one with highest trust_tier
/// - Keep the one with most complete data (deadline_date, eligibility)
/// - Update canonical_url for all kept leads
pub fn deduplicate_leads(leads: Vec<Lead>) -> Vec<Lead> {
    let mut dedup_map: HashMap<String, Lead> = HashMap::new();
    let mut seen_signatures: HashSet<String> = HashSet::new();
    
    for mut lead in leads {
        let key = generate_dedup_key(&lead);
        
        // Also check content signature for near-duplicates
        let content_sig = format!("{}|{}", 
            normalize_text(&lead.name),
            lead.deadline_date.as_deref().unwrap_or(&lead.deadline)
        );
        
        if seen_signatures.contains(&content_sig) && !dedup_map.contains_key(&key) {
            // Near-duplicate by content but different URL - skip
            continue;
        }
        
        // Set canonical URL
        if lead.canonical_url.is_none() {
            lead.canonical_url = Some(normalize_url(&lead.url));
        }
        
        if let Some(existing) = dedup_map.get(&key) {
            // Keep the better quality lead
            if is_better_quality(&lead, existing) {
                dedup_map.insert(key.clone(), lead);
            }
        } else {
            seen_signatures.insert(content_sig);
            dedup_map.insert(key, lead);
        }
    }
    
    dedup_map.into_values().collect()
}

/// Compare two leads and determine if the new one is better quality
fn is_better_quality(new_lead: &Lead, existing: &Lead) -> bool {
    // Score based on data completeness
    let new_score = quality_score(new_lead);
    let existing_score = quality_score(existing);
    
    if new_score != existing_score {
        return new_score > existing_score;
    }
    
    // Prefer higher trust tier
    let new_tier = tier_rank(new_lead.trust_tier.as_deref());
    let existing_tier = tier_rank(existing.trust_tier.as_deref());
    
    new_tier > existing_tier
}

/// Calculate quality score for a lead based on data completeness
fn quality_score(lead: &Lead) -> i32 {
    let mut score = 0;
    
    if lead.deadline_date.is_some() { score += 3; }
    if !lead.eligibility.is_empty() { score += 2; }
    if lead.is_taiwan_eligible.is_some() { score += 2; }
    if !lead.amount.is_empty() && lead.amount != "See website" { score += 1; }
    if lead.http_status == Some(200) { score += 2; }
    if lead.official_source_url.is_some() { score += 1; }
    
    score
}

/// Convert trust tier to numeric rank for comparison
fn tier_rank(tier: Option<&str>) -> i32 {
    match tier {
        Some("S") => 4,
        Some("A") => 3,
        Some("B") => 2,
        Some("C") => 1,
        _ => 0,
    }
}

/// Extract domain from URL for grouping
pub fn extract_domain(url: &str) -> String {
    let normalized = normalize_url(url);
    if let Some(start) = normalized.find("://") {
        let rest = &normalized[start + 3..];
        if let Some(end) = rest.find('/') {
            return rest[..end].to_string();
        }
        return rest.to_string();
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_url_https() {
        assert_eq!(
            normalize_url("http://example.com/path"),
            "https://example.com/path"
        );
    }
    
    #[test]
    fn test_normalize_url_lowercase_host() {
        assert_eq!(
            normalize_url("https://EXAMPLE.COM/Path"),
            "https://example.com/Path"
        );
    }
    
    #[test]
    fn test_normalize_url_remove_tracking() {
        assert_eq!(
            normalize_url("https://example.com/page?id=123&utm_source=google&utm_campaign=test"),
            "https://example.com/page?id=123"
        );
    }
    
    #[test]
    fn test_normalize_url_trailing_slash() {
        assert_eq!(
            normalize_url("https://example.com/path/"),
            "https://example.com/path"
        );
        // Root path keeps slash
        assert_eq!(
            normalize_url("https://example.com/"),
            "https://example.com/"
        );
    }
    
    #[test]
    fn test_normalize_url_sort_params() {
        let url1 = normalize_url("https://example.com?b=2&a=1");
        let url2 = normalize_url("https://example.com?a=1&b=2");
        assert_eq!(url1, url2);
    }
    
    #[test]
    fn test_normalize_url_remove_fragment() {
        assert_eq!(
            normalize_url("https://example.com/page#section"),
            "https://example.com/page"
        );
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.gla.ac.uk/scholarships/"), "www.gla.ac.uk");
        assert_eq!(extract_domain("http://example.com:8080/path"), "example.com:8080");
    }
    
    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("  Hello   World  "), "hello world");
    }
}

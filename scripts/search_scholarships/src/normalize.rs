//! URL Normalization and Deduplication Module
//!
//! Provides functions to:
//! - Normalize URLs (https, lowercase host, remove tracking params)
//! - Resolve canonical URLs from HTML <link rel="canonical">
//! - Deduplicate leads based on canonical URL or content signature
//! - Follow redirects to get final canonical URLs

use crate::types::Lead;
use std::collections::{HashMap, HashSet};
use reqwest::Client;
use scraper::{Html, Selector};

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

/// Resolve canonical URL from HTML page
/// 
/// Attempts to extract canonical URL from:
/// 1. <link rel="canonical" href="...">
/// 2. HTTP redirects (301/302)
/// 3. Falls back to normalized URL if not found
pub async fn resolve_canonical_url(client: &Client, url: &str) -> String {
    // First normalize the URL
    let normalized = normalize_url(url);
    
    // Try to fetch HTML and extract canonical link
    if let Ok(response) = client.get(&normalized).send().await {
        let final_url = response.url().clone();
        
        if let Ok(html_text) = response.text().await {
            let html = Html::parse_document(&html_text);
            // Look for <link rel="canonical" href="...">
            if let Ok(selector) = Selector::parse("link[rel='canonical']") {
                for element in html.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        // Resolve relative URLs
                        if let Ok(resolved) = resolve_relative_url(&normalized, href) {
                            return normalize_url(&resolved);
                        }
                    }
                }
            }
        }
        
        // Check if URL was redirected
        if final_url.as_str() != normalized {
            return normalize_url(final_url.as_str());
        }
    }
    
    // Fallback to normalized URL
    normalized
}

/// Resolve relative URL against base URL
fn resolve_relative_url(base: &str, relative: &str) -> Result<String, ()> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Ok(relative.to_string());
    }
    
    // Simple relative URL resolution
    if let Some(pos) = base.rfind('/') {
        if relative.starts_with('/') {
            // Absolute path relative to domain
            if let Some(domain_end) = base.find("://") {
                if let Some(path_start) = base[domain_end + 3..].find('/') {
                    let domain = &base[..domain_end + 3 + path_start];
                    return Ok(format!("{}{}", domain, relative));
                }
            }
        } else {
            // Relative path
            let base_path = &base[..pos + 1];
            return Ok(format!("{}{}", base_path, relative));
        }
    }
    
    Err(())
}

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

/// Generate entity-level deduplication key
/// 
/// Uses: provider + title + deadline + award + level
/// This is more robust than URL-based deduplication for cases where
/// the same scholarship appears on multiple pages or domains
/// 
/// Enhanced version: Also includes hash of normalized name + canonical_url for stronger uniqueness
pub fn generate_entity_dedup_key(lead: &Lead) -> String {
    // Provider: source domain or source name
    let provider = lead.source_domain.as_ref()
        .map(|d| d.to_lowercase())
        .unwrap_or_else(|| normalize_text(&lead.source));
    
    // Title: normalized scholarship name
    let title = normalize_text(&lead.name);
    
    // Deadline: prefer structured date, fallback to deadline string
    let deadline = lead.deadline_date.as_deref()
        .unwrap_or_else(|| {
            // Normalize deadline string (remove "TBD", "Check website", etc.)
            let deadline_str = &lead.deadline;
            if deadline_str.to_lowercase().contains("tbd") || 
               deadline_str.to_lowercase().contains("check") ||
               deadline_str.to_lowercase().contains("see website") {
                "unknown"
            } else {
                deadline_str
            }
        });
    
    // Award: normalized amount
    let award = normalize_text(&lead.amount);
    
    // Level: extract from eligibility or notes (postgraduate, undergraduate, etc.)
    let level = extract_level(lead);
    
    // Create base key
    let base_key = format!("{}|{}|{}|{}|{}", provider, title, deadline, award, level);
    
    // Add hash of normalized name + canonical_url for stronger uniqueness (as per plan)
    let canonical = lead.canonical_url.as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&lead.url);
    let hash_input = format!("{}|{}", normalize_text(&lead.name), normalize_url(canonical));
    let hash = crate::url_state::UrlStateStorage::calculate_content_hash(hash_input.as_bytes());
    
    // Combine base key with hash for stronger deduplication
    format!("{}|{}", base_key, &hash[..16])  // Use first 16 chars of hash
}

/// Extract programme level from lead
fn extract_level(lead: &Lead) -> String {
    let text = format!("{} {} {}", 
        lead.name, 
        lead.notes, 
        lead.eligibility.join(" ")
    ).to_lowercase();
    
    if text.contains("postgraduate") || text.contains("master") || text.contains("masters") || text.contains("m.sc") || text.contains("m.a") || text.contains("m.eng") {
        "postgraduate".to_string()
    } else if text.contains("undergraduate") || text.contains("bachelor") || text.contains("b.sc") || text.contains("b.a") {
        "undergraduate".to_string()
    } else if text.contains("phd") || text.contains("doctoral") || text.contains("d.phil") {
        "phd".to_string()
    } else {
        "unknown".to_string()
    }
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

/// Deduplication result with statistics
#[derive(Debug, Default)]
pub struct DeduplicationStats {
    pub total_input: usize,
    pub unique_output: usize,
    pub duplicates_removed: usize,
    pub dup_count_by_key: HashMap<String, usize>,
}

/// Deduplicate leads and return unique leads with best quality
/// 
/// When duplicates are found:
/// - Keep the one with highest trust_tier
/// - Keep the one with most complete data (deadline_date, eligibility)
/// - Update canonical_url for all kept leads
/// 
/// Uses entity-level deduplication key: provider + title + deadline + award + level
pub fn deduplicate_leads(leads: Vec<Lead>) -> Vec<Lead> {
    deduplicate_leads_with_stats(leads).0
}

/// Deduplicate leads and return statistics
pub fn deduplicate_leads_with_stats(leads: Vec<Lead>) -> (Vec<Lead>, DeduplicationStats) {
    let mut dedup_map: HashMap<String, Lead> = HashMap::new();
    let mut dup_count: HashMap<String, usize> = HashMap::new();
    let mut seen_signatures: HashSet<String> = HashSet::new();
    let mut content_hash_map: HashMap<String, String> = HashMap::new(); // content_hash -> entity_key
    
    for mut lead in leads {
        // Use entity-level deduplication key
        let key = generate_entity_dedup_key(&lead);
        
        // Calculate content hash for second-level deduplication
        let content_hash = calculate_lead_content_hash(&lead);
        
        // Check if content-hash already exists (different entity_key but same content)
        if let Some(existing_key) = content_hash_map.get(&content_hash) {
            if existing_key != &key {
                // Content same but entity_key different - treat as duplicate, skip
                *dup_count.entry(key.clone()).or_insert(0) += 1;
                continue;
            }
        }
        
        // Track duplicate count
        *dup_count.entry(key.clone()).or_insert(0) += 1;
        
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
            // Entity key matches - check if content-hash differs (different version)
            let existing_hash = calculate_lead_content_hash(existing);
            if content_hash != existing_hash {
                // Same entity but different content - update if new is better quality
                if is_better_quality(&lead, existing) {
                    dedup_map.insert(key.clone(), lead);
                    content_hash_map.insert(content_hash, key.clone());
                }
            } else {
                // Same entity and same content - keep existing (already better quality)
            }
        } else {
            // New entity key
            seen_signatures.insert(content_sig);
            dedup_map.insert(key.clone(), lead);
            content_hash_map.insert(content_hash, key.clone());
        }
    }
    
    let unique_output = dedup_map.len();
    let total_input = dup_count.values().sum::<usize>();
    let duplicates_removed = total_input - unique_output;
    
    let stats = DeduplicationStats {
        total_input,
        unique_output,
        duplicates_removed,
        dup_count_by_key: dup_count.into_iter()
            .filter(|(_, count)| *count > 1)
            .collect(),
    };
    
    (dedup_map.into_values().collect(), stats)
}

/// Calculate content hash for a lead
fn calculate_lead_content_hash(lead: &Lead) -> String {
    let content = format!("{}|{}|{}|{}", 
        normalize_text(&lead.name),
        normalize_text(&lead.amount),
        normalize_text(&lead.deadline),
        lead.eligibility.iter().map(|e| normalize_text(e)).collect::<Vec<_>>().join(",")
    );
    crate::url_state::UrlStateStorage::calculate_content_hash(content.as_bytes())
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

/// Canonicalize candidate URL for deduplication
/// 
/// Specifically designed for candidate URLs from discovery:
/// - Removes UTM parameters and other tracking params
/// - Removes fragment (#section)
/// - Normalizes trailing slash
/// - Lowercases domain
/// 
/// This is a wrapper around normalize_url with candidate-specific semantics
pub fn canonicalize_candidate_url(url: &str) -> String {
    normalize_url(url)
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

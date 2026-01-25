//! Discovery Engine Module
//!
//! Provides URL discovery strategies:
//! - robots.txt -> sitemap URLs
//! - sitemap index traversal (with size limits)
//! - RSS/Atom feeds
//! - site internal search endpoints

use crate::types::Source;
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Candidate URL for crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateUrl {
    pub url: String,
    pub source_seed: String,        // 哪個 seed 來的（Source name）
    pub discovered_from: String,   // 在哪個頁面找到（URL）
    pub confidence: f32,             // 0~1
    pub reason: String,             // e.g. "contains scholarship keyword in url path"
    pub discovered_at: String,     // ISO 8601
    pub tags: Vec<String>,         // e.g. ["gov.uk", "funding"]
    // 保留現有字段以向後相容
    #[serde(default)]
    pub source_id: String,          // 保留（等同 source_seed）
    #[serde(default)]
    pub discovery_source: DiscoverySource,  // 保留舊的 enum 字段
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiscoverySource {
    #[default]
    Manual,
    RobotsTxt,
    Sitemap,
    SitemapIndex,
    RssFeed,
    AtomFeed,
    SearchEndpoint,
    ExternalLink,  // 新增：從外部連結發現
}

/// Discovery configuration for a source
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub allowlist_path_regex: Option<Regex>,
    pub search_endpoints: Vec<String>,
    pub search_keywords: Vec<String>,
    pub max_sitemap_size: usize,  // Max URLs per sitemap
    pub max_total_urls: usize,     // Max total URLs to discover
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            allowlist_path_regex: None,
            search_endpoints: vec![],
            search_keywords: vec!["scholarship".to_string(), "funding".to_string(), "bursary".to_string(), "award".to_string(), "grant".to_string()],
            max_sitemap_size: 50000,
            max_total_urls: 10000,
        }
    }
}

/// Discover URLs from a source using multiple strategies
pub async fn discover_urls(
    client: &Client,
    source: &Source,
    config: &DiscoveryConfig,
) -> Result<Vec<CandidateUrl>> {
    let mut candidates: Vec<CandidateUrl> = Vec::new();
    let mut seen_urls: HashSet<String> = HashSet::new();
    
    // Extract base URL
    let base_url = extract_base_url(&source.url)?;
    
    // Strategy 1: robots.txt -> sitemap URLs
    if let Ok(sitemap_urls) = discover_from_robots_txt(client, &base_url).await {
        for sitemap_url in sitemap_urls {
            if seen_urls.insert(sitemap_url.clone()) {
                candidates.push(CandidateUrl {
                    url: sitemap_url.clone(),
                    source_seed: source.name.clone(),
                    discovered_from: base_url.clone(),
                    confidence: 0.9,
                    reason: "Discovered from robots.txt sitemap".to_string(),
                    discovered_at: chrono::Utc::now().to_rfc3339(),
                    tags: vec!["sitemap".to_string()],
                    source_id: source.name.clone(),
                    discovery_source: DiscoverySource::RobotsTxt,
                });
            }
        }
    }
    
    // Strategy 2: Try common sitemap locations
    let common_sitemaps = vec![
        format!("{}/sitemap.xml", base_url),
        format!("{}/sitemap_index.xml", base_url),
        format!("{}/sitemaps.xml", base_url),
    ];
    
    for sitemap_url in common_sitemaps {
        if seen_urls.insert(sitemap_url.clone()) {
            candidates.push(CandidateUrl {
                url: sitemap_url.clone(),
                source_seed: source.name.clone(),
                discovered_from: base_url.clone(),
                confidence: 0.7,
                reason: "Common sitemap location".to_string(),
                discovered_at: chrono::Utc::now().to_rfc3339(),
                tags: vec!["sitemap".to_string()],
                source_id: source.name.clone(),
                discovery_source: DiscoverySource::Sitemap,
            });
        }
    }
    
    // Strategy 3: RSS/Atom feeds
    let feed_urls = discover_feeds(client, &base_url).await?;
    for feed_url in feed_urls {
        if seen_urls.insert(feed_url.url.clone()) {
            candidates.push(CandidateUrl {
                url: feed_url.url.clone(),
                source_seed: source.name.clone(),
                discovered_from: base_url.clone(),
                confidence: 0.8,
                reason: format!("Discovered from {:?} feed", feed_url.source),
                discovered_at: chrono::Utc::now().to_rfc3339(),
                tags: vec!["feed".to_string()],
                source_id: source.name.clone(),
                discovery_source: feed_url.source.clone(),
            });
        }
    }
    
    // Strategy 4: Site internal search endpoints
    for search_endpoint in &config.search_endpoints {
        for keyword in &config.search_keywords {
            let search_url = format!("{}?q={}", search_endpoint, keyword);
            if seen_urls.insert(search_url.clone()) {
                candidates.push(CandidateUrl {
                    url: search_url.clone(),
                    source_seed: source.name.clone(),
                    discovered_from: base_url.clone(),
                    confidence: 0.6,
                    reason: format!("Search endpoint with keyword: {}", keyword),
                    discovered_at: chrono::Utc::now().to_rfc3339(),
                    tags: vec!["search".to_string(), keyword.clone()],
                    source_id: source.name.clone(),
                    discovery_source: DiscoverySource::SearchEndpoint,
                });
            }
        }
    }
    
    // Apply allowlist path regex filter
    if let Some(ref regex) = config.allowlist_path_regex {
        candidates.retain(|c| {
            regex.is_match(&c.url)
        });
    }
    
    // Limit total URLs
    candidates.truncate(config.max_total_urls);
    
    Ok(candidates)
}

/// Extract base URL (scheme + host) from a URL
fn extract_base_url(url: &str) -> Result<String> {
    if let Some(pos) = url.find("://") {
        let rest = &url[pos + 3..];
        if let Some(path_pos) = rest.find('/') {
            Ok(format!("{}://{}", &url[..pos + 3], &rest[..path_pos]))
        } else {
            Ok(url.to_string())
        }
    } else {
        Err(anyhow::anyhow!("Invalid URL format: {}", url))
    }
}

/// Discover sitemap URLs from robots.txt
async fn discover_from_robots_txt(client: &Client, base_url: &str) -> Result<Vec<String>> {
    let robots_url = format!("{}/robots.txt", base_url);
    let response = client.get(&robots_url).send().await?;
    
    if !response.status().is_success() {
        return Ok(vec![]);
    }
    
    let text = response.text().await?;
    let mut sitemaps = Vec::new();
    
    for line in text.lines() {
        let line = line.trim();
        if line.to_lowercase().starts_with("sitemap:") {
            if let Some(url) = line.splitn(2, ':').nth(1) {
                let url = url.trim();
                if !url.is_empty() {
                    sitemaps.push(url.to_string());
                }
            }
        }
    }
    
    Ok(sitemaps)
}

struct FeedInfo {
    url: String,
    source: DiscoverySource,
}

/// Discover RSS/Atom feeds from HTML (public wrapper)
pub async fn discover_feeds_public(client: &Client, base_url: &str) -> Result<Vec<String>> {
    let feeds = discover_feeds(client, base_url).await?;
    Ok(feeds.into_iter().map(|f| f.url).collect())
}

/// Discover RSS/Atom feeds from HTML
async fn discover_feeds(client: &Client, base_url: &str) -> Result<Vec<FeedInfo>> {
    let response = client.get(base_url).send().await?;
    
    if !response.status().is_success() {
        return Ok(vec![]);
    }
    
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    let mut feeds = Vec::new();
    
    // Look for <link rel="alternate" type="application/rss+xml">
    if let Ok(selector) = Selector::parse("link[rel='alternate']") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Some(type_attr) = element.value().attr("type") {
                    let feed_url = resolve_url(base_url, href);
                    if type_attr.contains("rss") || type_attr.contains("atom") {
                        feeds.push(FeedInfo {
                            url: feed_url,
                            source: if type_attr.contains("atom") {
                                DiscoverySource::AtomFeed
                            } else {
                                DiscoverySource::RssFeed
                            },
                        });
                    }
                }
            }
        }
    }
    
    // Try common feed locations
    let common_feeds = vec![
        format!("{}/feed", base_url),
        format!("{}/rss", base_url),
        format!("{}/atom.xml", base_url),
        format!("{}/feed.xml", base_url),
    ];
    
    for feed_url in common_feeds {
        feeds.push(FeedInfo {
            url: feed_url,
            source: DiscoverySource::RssFeed,
        });
    }
    
    Ok(feeds)
}

/// Resolve relative URL against base
fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return relative.to_string();
    }
    
    if relative.starts_with('/') {
        if let Some(pos) = base.find("://") {
            let rest = &base[pos + 3..];
            if let Some(path_pos) = rest.find('/') {
                return format!("{}://{}{}", &base[..pos + 3], &rest[..path_pos], relative);
            }
        }
    }
    
    format!("{}/{}", base.trim_end_matches('/'), relative.trim_start_matches('/'))
}

/// Parse sitemap XML and extract URLs
pub async fn parse_sitemap(client: &Client, sitemap_url: &str, config: &DiscoveryConfig) -> Result<Vec<CandidateUrl>> {
    parse_sitemap_internal(client, sitemap_url, config, &mut HashSet::new()).await
}

async fn parse_sitemap_internal(
    client: &Client,
    sitemap_url: &str,
    config: &DiscoveryConfig,
    seen_sitemaps: &mut HashSet<String>,
) -> Result<Vec<CandidateUrl>> {
    // Prevent infinite recursion
    if seen_sitemaps.contains(sitemap_url) {
        return Ok(vec![]);
    }
    seen_sitemaps.insert(sitemap_url.to_string());
    
    let response = client.get(sitemap_url).send().await?;
    
    if !response.status().is_success() {
        return Ok(vec![]);
    }
    
    let text = response.text().await?;
    let mut urls = Vec::new();
    
    // Check if it's a sitemap index
    if text.contains("<sitemapindex>") {
        // Parse sitemap index
        let document = Html::parse_document(&text);
        if let Ok(selector) = Selector::parse("sitemap > loc") {
            for element in document.select(&selector) {
                if let Some(loc_text) = element.text().next() {
                    let loc_url = loc_text.trim().to_string();
                    // Recursively parse nested sitemaps using Box::pin
                    let future = parse_sitemap_internal(client, &loc_url, config, seen_sitemaps);
                    if let Ok(nested_urls) = Box::pin(future).await {
                        urls.extend(nested_urls);
                        if urls.len() >= config.max_total_urls {
                            break;
                        }
                    }
                }
            }
        }
    } else {
        // Parse regular sitemap
        let document = Html::parse_document(&text);
        if let Ok(selector) = Selector::parse("url > loc") {
            for (idx, element) in document.select(&selector).enumerate() {
                if idx >= config.max_sitemap_size {
                    break;
                }
                
                if let Some(loc_text) = element.text().next() {
                    let url = loc_text.trim().to_string();
                    urls.push(CandidateUrl {
                        url: url.clone(),
                        source_seed: String::new(), // Will be set by caller
                        discovered_from: sitemap_url.to_string(),
                        confidence: 0.8,
                        reason: "Found in sitemap".to_string(),
                        discovered_at: chrono::Utc::now().to_rfc3339(),
                        tags: vec!["sitemap".to_string()],
                        source_id: String::new(),
                        discovery_source: DiscoverySource::Sitemap,
                    });
                }
            }
        }
    }
    
    Ok(urls)
}

/// Check content type and determine if URL should be crawled
pub fn should_crawl_by_content_type(content_type: &str, url: &str) -> bool {
    let content_type_lower = content_type.to_lowercase();
    
    // Skip large binaries
    if content_type_lower.contains("application/octet-stream") ||
       content_type_lower.contains("application/zip") ||
       content_type_lower.contains("application/x-zip-compressed") {
        return false;
    }
    
    // Handle PDFs explicitly (can be crawled for text extraction)
    if content_type_lower.contains("application/pdf") {
        return true; // PDFs can be processed
    }
    
    // Prefer HTML/text content
    if content_type_lower.contains("text/html") ||
       content_type_lower.contains("text/plain") ||
       content_type_lower.contains("application/xhtml") {
        return true;
    }
    
    // Check URL extension as fallback
    let url_lower = url.to_lowercase();
    if url_lower.ends_with(".pdf") {
        return true; // PDFs can be processed
    }
    
    // Skip other binary formats
    if url_lower.ends_with(".zip") ||
       url_lower.ends_with(".tar") ||
       url_lower.ends_with(".gz") ||
       url_lower.ends_with(".exe") ||
       url_lower.ends_with(".dmg") {
        return false;
    }
    
    // Default: allow crawling
    true
}

/// Extract domain from URL
fn extract_domain_from_url(url: &str) -> Option<String> {
    if let Some(pos) = url.find("://") {
        let rest = &url[pos + 3..];
        if let Some(path_pos) = rest.find('/') {
            Some(rest[..path_pos].to_string())
        } else if let Some(query_pos) = rest.find('?') {
            Some(rest[..query_pos].to_string())
        } else {
            Some(rest.to_string())
        }
    } else {
        None
    }
}

/// Check if domain matches allowlist (supports wildcard patterns like *.ac.uk)
fn matches_domain_allowlist(domain: &str, allowlist: &[String]) -> bool {
    for allowed in allowlist {
        // Exact match
        if domain == allowed {
            return true;
        }
        
        // Wildcard match (e.g., *.ac.uk)
        if allowed.starts_with("*.") {
            let suffix = &allowed[2..];
            if domain.ends_with(suffix) || domain == &suffix[1..] {
                return true;
            }
        }
        
        // Subdomain match (e.g., domain contains allowed)
        if domain.contains(allowed) {
            return true;
        }
    }
    false
}

/// Check if URL path matches deny patterns
fn matches_deny_patterns(url: &str, deny_patterns: &[String]) -> bool {
    let url_lower = url.to_lowercase();
    for pattern in deny_patterns {
        if url_lower.contains(pattern) {
            return true;
        }
    }
    false
}

/// Resolve relative URL against base URL
fn resolve_relative_url(base: &str, relative: &str) -> Option<String> {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return Some(relative.to_string());
    }
    
    if let Some(pos) = base.find("://") {
        let scheme = &base[..pos + 3];
        let rest = &base[pos + 3..];
        
        if relative.starts_with('/') {
            // Absolute path relative to domain
            if let Some(path_start) = rest.find('/') {
                let domain = &rest[..path_start];
                return Some(format!("{}{}{}", scheme, domain, relative));
            } else {
                return Some(format!("{}{}{}", scheme, rest, relative));
            }
        } else {
            // Relative path
            if let Some(path_pos) = rest.rfind('/') {
                let base_path = &rest[..path_pos + 1];
                return Some(format!("{}{}{}", scheme, base_path, relative));
            } else {
                return Some(format!("{}{}/{}", scheme, rest, relative));
            }
        }
    }
    
    None
}

/// Discover URLs from a discovery_seed source
/// 
/// Implements controlled BFS crawling with:
/// - Domain allowlist/denylist filtering
/// - Path/keyword gates with confidence scoring
/// - Depth limiting (max_depth from source config)
pub async fn discover_from_seed(
    client: &Client,
    source: &Source,
) -> Result<Vec<CandidateUrl>> {
    let max_depth = source.max_depth.unwrap_or(1);
    let mut candidates: Vec<CandidateUrl> = Vec::new();
    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u8)> = VecDeque::new(); // (url, depth)
    
    // Extract base domain for exclusion
    let base_domain = extract_domain_from_url(&source.url)
        .ok_or_else(|| anyhow::anyhow!("Invalid source URL: {}", source.url))?;
    
    // Start with seed URL
    queue.push_back((source.url.clone(), 0));
    seen_urls.insert(source.url.clone());
    
    while let Some((current_url, depth)) = queue.pop_front() {
        // Check depth limit
        if depth >= max_depth {
            continue;
        }
        
        // Fetch page
        let response = match client.get(&current_url).send().await {
            Ok(resp) => resp,
            Err(_) => continue, // Skip on error
        };
        
        if !response.status().is_success() {
            continue;
        }
        
        let html = match response.text().await {
            Ok(h) => h,
            Err(_) => continue,
        };
        
        let document = Html::parse_document(&html);
        
        // Extract page title
        let page_title = document
            .select(&Selector::parse("title").unwrap_or_else(|_| unreachable!()))
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.to_string());
        
        // Extract links
        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    // Resolve URL
                    let resolved_url = match resolve_relative_url(&current_url, href) {
                        Some(url) => url,
                        None => continue,
                    };
                    
                    // Skip if already seen
                    let normalized = crate::normalize::normalize_url(&resolved_url);
                    if !seen_urls.insert(normalized.clone()) {
                        continue;
                    }
                    
                    // Extract domain
                    let domain = match extract_domain_from_url(&resolved_url) {
                        Some(d) => d,
                        None => continue,
                    };
                    
                    // Check deny patterns
                    if let Some(ref deny_patterns) = source.deny_patterns {
                        if matches_deny_patterns(&resolved_url, deny_patterns) {
                            continue;
                        }
                    }
                    
                    // Check domain allowlist (if configured)
                    let is_allowed = if let Some(ref allow_domains) = source.allow_domains_outbound {
                        matches_domain_allowlist(&domain, allow_domains)
                    } else {
                        // If no allowlist, only allow external domains (not same as seed)
                        !domain.contains(&base_domain)
                    };
                    
                    if !is_allowed {
                        continue;
                    }
                    
                    // Extract anchor text
                    let anchor_text = element.text().collect::<String>();
                    
                    // Calculate confidence
                    let confidence = calculate_confidence(
                        &resolved_url,
                        if anchor_text.is_empty() { None } else { Some(&anchor_text) },
                        page_title.as_deref(),
                    );
                    
                    // Only add if confidence >= 0.6
                    if confidence >= 0.6 {
                        let mut tags = vec!["discovery_seed".to_string()];
                        if domain.contains(".gov.uk") {
                            tags.push("gov.uk".to_string());
                        }
                        if domain.contains(".ac.uk") {
                            tags.push("ac.uk".to_string());
                        }
                        
                        candidates.push(CandidateUrl {
                            url: resolved_url.clone(),
                            source_seed: source.name.clone(),
                            discovered_from: current_url.clone(),
                            confidence,
                            reason: format!("Discovered from seed (depth={}, confidence={:.2})", depth, confidence),
                            discovered_at: chrono::Utc::now().to_rfc3339(),
                            tags,
                            source_id: source.name.clone(),
                            discovery_source: DiscoverySource::ExternalLink,
                        });
                        
                        // Add to queue for next depth level (if not at max depth)
                        if depth + 1 < max_depth {
                            queue.push_back((resolved_url, depth + 1));
                        }
                    }
                }
            }
        }
    }
    
    Ok(candidates)
}

/// Calculate confidence score for a candidate URL
/// 
/// Scoring rules:
/// - URL path contains funding keywords: +0.5
/// - Anchor text contains funding keywords: +0.3
/// - Page title contains funding keywords: +0.2
/// - Guide/How-to patterns: -0.4
/// 
/// Returns confidence score (0.0 - 1.0)
pub fn calculate_confidence(
    url: &str,
    anchor_text: Option<&str>,
    page_title: Option<&str>,
) -> f32 {
    let mut confidence: f32 = 0.0;
    let url_lower = url.to_lowercase();
    
    // URL path patterns (+0.5)
    let path_patterns = [
        "/scholarship", "/funding", "/bursary", "/studentship", "/fees-funding",
        "/award", "/grant", "/financial-aid", "/financial-support"
    ];
    if path_patterns.iter().any(|pattern| url_lower.contains(pattern)) {
        confidence += 0.5;
    }
    
    // Anchor text patterns (+0.3)
    if let Some(anchor) = anchor_text {
        let anchor_lower = anchor.to_lowercase();
        let anchor_keywords = [
            "scholarship", "funding", "bursary", "grant", "award",
            "financial aid", "financial support", "studentship"
        ];
        if anchor_keywords.iter().any(|kw| anchor_lower.contains(kw)) {
            confidence += 0.3;
        }
        
        // Guide/How-to penalty (-0.4)
        let guide_patterns = [
            "how to", "guide", "tips", "finding", "types of",
            "what is", "explaining", "overview"
        ];
        if guide_patterns.iter().any(|pattern| anchor_lower.contains(pattern)) {
            confidence -= 0.4;
        }
    }
    
    // Page title patterns (+0.2)
    if let Some(title) = page_title {
        let title_lower = title.to_lowercase();
        let title_keywords = [
            "scholarship", "funding", "bursary", "grant", "award"
        ];
        if title_keywords.iter().any(|kw| title_lower.contains(kw)) {
            confidence += 0.2;
        }
    }
    
    // URL path guide patterns (-0.4)
    let url_guide_patterns = [
        "overview", "what-is", "how-to", "guide", "types-of",
        "explaining", "finding"
    ];
    if url_guide_patterns.iter().any(|pattern| url_lower.contains(pattern)) {
        confidence -= 0.4;
    }
    
    // Clamp to [0.0, 1.0]
    confidence.max(0.0).min(1.0)
}

/// Heavy validation for candidate URLs
/// 
/// Performs full HTTP GET + HTML content check:
/// - Validates HTTP status code (must be 200)
/// - Validates Content-Type (must be HTML)
/// - Checks HTML content for funding keywords
/// - Checks for application forms or eligibility criteria
/// - Updates candidate confidence and tags based on validation results
pub async fn validate_candidate_heavy(
    client: &Client,
    candidate: &mut CandidateUrl,
) -> Result<bool> {
    // Send HTTP GET request
    let response = match client
        .get(&candidate.url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            candidate.reason = format!("HTTP request failed: {}", e);
            candidate.tags.push("validation_failed".to_string());
            return Ok(false);
        }
    };
    
    // Check HTTP status code
    let status = response.status();
    if !status.is_success() {
        candidate.reason = format!("HTTP {}: {}", status.as_u16(), status.as_str());
        candidate.tags.push("invalid_status".to_string());
        candidate.confidence = (candidate.confidence * 0.5).max(0.0);
        return Ok(false);
    }
    
    // Check Content-Type
    let content_type = response.headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    
    if !content_type.contains("text/html") && !content_type.contains("application/xhtml") {
        candidate.reason = format!("Invalid Content-Type: {}", content_type);
        candidate.tags.push("invalid_content_type".to_string());
        candidate.confidence = (candidate.confidence * 0.7).max(0.0);
        return Ok(false);
    }
    
    // Parse HTML
    let html = match response.text().await {
        Ok(h) => h,
        Err(e) => {
            candidate.reason = format!("Failed to read HTML: {}", e);
            candidate.tags.push("parse_error".to_string());
            return Ok(false);
        }
    };
    
    let document = Html::parse_document(&html);
    
    // Extract title
    let title = document
        .select(&Selector::parse("title").unwrap_or_else(|_| unreachable!()))
        .next()
        .and_then(|el| el.text().next())
        .map(|s| s.to_string());
    
    // Extract meta description
    let meta_description = document
        .select(&Selector::parse("meta[name='description']").unwrap_or_else(|_| unreachable!()))
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());
    
    // Extract body text
    let body_text = document
        .select(&Selector::parse("body").unwrap_or_else(|_| unreachable!()))
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    // Check for funding keywords
    let funding_keywords = [
        "scholarship", "funding", "bursary", "grant", "award",
        "financial aid", "financial support", "studentship"
    ];
    
    let text_to_check = format!("{} {} {}", 
        title.as_deref().unwrap_or(""),
        meta_description.as_deref().unwrap_or(""),
        body_text
    ).to_lowercase();
    
    let has_funding_keyword = funding_keywords.iter()
        .any(|kw| text_to_check.contains(kw));
    
    if !has_funding_keyword {
        candidate.reason = "No funding keywords found in page content".to_string();
        candidate.tags.push("no_funding_content".to_string());
        candidate.confidence = (candidate.confidence * 0.5).max(0.0);
        return Ok(false);
    }
    
    // Check for application form or eligibility criteria
    let has_application_form = body_text.to_lowercase().contains("<form") ||
        body_text.to_lowercase().contains("apply now") ||
        body_text.to_lowercase().contains("application form");
    
    let has_eligibility = body_text.to_lowercase().contains("eligibility") ||
        body_text.to_lowercase().contains("requirements") ||
        body_text.to_lowercase().contains("criteria");
    
    // Check for guide/overview patterns
    let url_lower = candidate.url.to_lowercase();
    let is_guide_page = url_lower.contains("overview") ||
        url_lower.contains("what-is") ||
        url_lower.contains("how-to") ||
        url_lower.contains("guide") ||
        url_lower.contains("types-of");
    
    // Update confidence based on validation results
    if has_application_form {
        candidate.confidence = (candidate.confidence + 0.2).min(1.0);
        candidate.tags.push("has_application_form".to_string());
    }
    
    if has_eligibility {
        candidate.confidence = (candidate.confidence + 0.1).min(1.0);
        candidate.tags.push("has_eligibility_info".to_string());
    }
    
    if is_guide_page && !has_application_form {
        candidate.confidence = (candidate.confidence - 0.4).max(0.0);
        candidate.tags.push("guide_page".to_string());
        if candidate.confidence < 0.6 {
            candidate.reason = "Guide/overview page without application form".to_string();
            return Ok(false);
        }
    }
    
    candidate.tags.push("validated".to_string());
    candidate.reason = format!("Validated: funding keywords found, has_form={}, has_eligibility={}", 
        has_application_form, has_eligibility);
    
    Ok(true)
}

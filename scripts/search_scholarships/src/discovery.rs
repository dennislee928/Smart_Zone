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
use std::collections::HashSet;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoverySource {
    RobotsTxt,
    Sitemap,
    SitemapIndex,
    RssFeed,
    AtomFeed,
    SearchEndpoint,
    Manual,
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

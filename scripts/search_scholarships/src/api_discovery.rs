//! API Discovery Module
//!
//! Discovers API endpoints from browser worker results and enables
//! direct API calls to avoid browser overhead

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use reqwest::Client;

/// API endpoint mapping (domain -> endpoint config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpointConfig {
    pub base_url: String,
    pub endpoint_pattern: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub discovered_at: String,
    pub last_used: Option<String>,
    pub success_count: u32,
    pub failure_count: u32,
}

/// API endpoints cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpointsCache {
    pub endpoints: HashMap<String, ApiEndpointConfig>,
}

impl Default for ApiEndpointsCache {
    fn default() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }
}

/// Discover API endpoints from browser worker network log
pub fn discover_api_from_browser_result(
    browser_result: &crate::browser_queue::BrowserResultEntry,
) -> Vec<String> {
    let mut endpoints = Vec::new();
    
    for api_endpoint in &browser_result.detected_api_endpoints {
        endpoints.push(api_endpoint.url.clone());
    }
    
    endpoints
}

/// Load API endpoints cache from file
pub fn load_api_endpoints_cache(root: &str) -> Result<ApiEndpointsCache> {
    let cache_path = PathBuf::from(root).join("tracking").join("api_endpoints.json");
    
    if !cache_path.exists() {
        return Ok(ApiEndpointsCache::default());
    }
    
    let file = File::open(&cache_path)
        .context("Failed to open API endpoints cache")?;
    
    let cache: ApiEndpointsCache = serde_json::from_reader(file)
        .context("Failed to parse API endpoints cache")?;
    
    Ok(cache)
}

/// Save API endpoints cache to file
pub fn save_api_endpoints_cache(root: &str, cache: &ApiEndpointsCache) -> Result<()> {
    let cache_path = PathBuf::from(root).join("tracking").join("api_endpoints.json");
    
    // Create tracking directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create tracking directory")?;
    }
    
    let file = File::create(&cache_path)
        .context("Failed to create API endpoints cache file")?;
    
    serde_json::to_writer_pretty(file, cache)
        .context("Failed to write API endpoints cache")?;
    
    Ok(())
}

/// Register a new API endpoint from browser worker result
pub fn register_api_endpoint(
    root: &str,
    domain: &str,
    endpoint_url: &str,
) -> Result<()> {
    let mut cache = load_api_endpoints_cache(root)?;
    
    // Extract base URL from endpoint
    let base_url = extract_base_url(endpoint_url)?;
    
    let config = ApiEndpointConfig {
        base_url: base_url.clone(),
        endpoint_pattern: endpoint_url.to_string(),
        method: "GET".to_string(),
        headers: HashMap::new(),
        params: HashMap::new(),
        discovered_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
        success_count: 0,
        failure_count: 0,
    };
    
    cache.endpoints.insert(domain.to_string(), config);
    
    save_api_endpoints_cache(root, &cache)?;
    
    Ok(())
}

/// Extract base URL from endpoint URL
fn extract_base_url(endpoint_url: &str) -> Result<String> {
    if let Some(pos) = endpoint_url.find("://") {
        let rest = &endpoint_url[pos + 3..];
        if let Some(path_pos) = rest.find('/') {
            return Ok(format!("{}://{}", &endpoint_url[..pos + 3], &rest[..path_pos]));
        }
        return Ok(endpoint_url.to_string());
    }
    
    anyhow::bail!("Invalid endpoint URL: {}", endpoint_url)
}

/// Check if we have a cached API endpoint for a domain
pub fn has_api_endpoint(root: &str, domain: &str) -> Result<bool> {
    let cache = load_api_endpoints_cache(root)?;
    Ok(cache.endpoints.contains_key(domain))
}

/// Call API directly (skip browser)
pub async fn call_api_directly(
    client: &Client,
    domain: &str,
    url: &str,
) -> Result<serde_json::Value> {
    // Try to get cached endpoint config
    // For now, just make a direct request
    // In the future, we can use cached config for headers/params
    
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .context("Failed to call API")?;
    
    if !response.status().is_success() {
        anyhow::bail!("API call failed with status: {}", response.status());
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse API response as JSON")?;
    
    Ok(json)
}

/// Extract scholarship data from API response
pub fn extract_scholarships_from_api_response(
    json: &serde_json::Value,
    url: &str,
) -> Vec<crate::types::Lead> {
    let mut leads = Vec::new();
    
    // Try common API response structures
    let scholarships = if let Some(arr) = json.get("scholarships").and_then(|v| v.as_array()) {
        arr
    } else if let Some(arr) = json.get("data").and_then(|v| v.as_array()) {
        arr
    } else if let Some(arr) = json.get("results").and_then(|v| v.as_array()) {
        arr
    } else if json.is_array() {
        json.as_array().unwrap()
    } else {
        return leads;
    };
    
    for item in scholarships {
        if let Some(obj) = item.as_object() {
            let name = obj.get("name")
                .or_else(|| obj.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let amount = obj.get("amount")
                .or_else(|| obj.get("value"))
                .or_else(|| obj.get("award"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let deadline = obj.get("deadline")
                .or_else(|| obj.get("applicationDeadline"))
                .or_else(|| obj.get("dueDate"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let eligibility = obj.get("eligibility")
                .or_else(|| obj.get("requirements"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            
            if !name.is_empty() {
                let lead = crate::types::Lead {
                    name,
                    amount,
                    deadline,
                    source: "api_extracted".to_string(),
                    source_type: "api".to_string(),
                    status: "new".to_string(),
                    eligibility,
                    notes: String::new(),
                    added_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    url: url.to_string(),
                    match_score: 0,
                    match_reasons: vec![],
                    hard_fail_reasons: vec![],
                    soft_flags: vec![],
                    bucket: None,
                    http_status: Some(200),
                    effort_score: None,
                    trust_tier: Some("A".to_string()),
                    risk_flags: vec![],
                    matched_rule_ids: vec![],
                    eligible_countries: vec![],
                    is_taiwan_eligible: None,
                    taiwan_eligibility_confidence: None,
                    deadline_date: None,
                    deadline_label: None,
                    intake_year: None,
                    study_start: None,
                    deadline_confidence: None,
                    canonical_url: None,
                    is_directory_page: false,
                    official_source_url: None,
                    source_domain: None,
                    confidence: Some(0.9),
                    eligibility_confidence: None,
                    tags: vec!["api_extracted".to_string()],
                    is_index_only: false,
                    first_seen_at: None,
                    last_checked_at: Some(chrono::Utc::now().to_rfc3339()),
                    next_check_at: None,
                    persistence_status: None,
                    source_seed: None,
                    check_count: Some(1),
                    extraction_evidence: vec![crate::types::ExtractionEvidence {
                        attribute: "api_response".to_string(),
                        snippet: serde_json::to_string(item).unwrap_or_default(),
                        selector: None,
                        url: url.to_string(),
                        method: "api_direct".to_string(),
                    }],
                };
                
                leads.push(lead);
            }
        }
    }
    
    leads
}

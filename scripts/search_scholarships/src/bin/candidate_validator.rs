//! Candidate Validator Binary
//!
//! Validates candidate URLs from candidate_urls.jsonl:
//! - Performs HTTP GET requests
//! - Checks HTTP status codes (filters 404/403/500)
//! - Checks Content-Type (ensures HTML)
//! - Checks HTML content for funding keywords
//! - Updates candidate confidence and tags
//! - Writes validated results back to candidate_urls.jsonl

// Note: This binary shares the same crate as main.rs
// We need to declare modules here since binaries don't automatically get access to lib modules
// For now, we'll use a simpler approach: read/write JSONL directly

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::fs::OpenOptions;

// Copy CandidateUrl struct definition (must match discovery.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateUrl {
    url: String,
    source_seed: String,
    discovered_from: String,
    confidence: f32,
    reason: String,
    discovered_at: String,
    tags: Vec<String>,
    #[serde(default)]
    source_id: String,
    #[serde(default)]
    discovery_source: String,  // Serialized as string for JSON
}

async fn validate_candidate(client: &Client, candidate: &mut CandidateUrl) -> Result<bool> {
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
    
    // Check for funding keywords
    let funding_keywords = [
        "scholarship", "funding", "bursary", "grant", "award",
        "financial aid", "financial support", "studentship"
    ];
    
    let text_lower = html.to_lowercase();
    let has_funding_keyword = funding_keywords.iter()
        .any(|kw| text_lower.contains(kw));
    
    if !has_funding_keyword {
        candidate.reason = "No funding keywords found in page content".to_string();
        candidate.tags.push("no_funding_content".to_string());
        candidate.confidence = (candidate.confidence * 0.5).max(0.0);
        return Ok(false);
    }
    
    candidate.tags.push("validated".to_string());
    candidate.reason = "Validated: funding keywords found".to_string();
    
    Ok(true)
}

fn load_candidates(root: &str) -> Result<Vec<CandidateUrl>> {
    let path = PathBuf::from(root).join("tracking/candidate_urls.jsonl");
    
    if !path.exists() {
        return Ok(vec![]);
    }
    
    let file = File::open(&path)
        .context("Failed to open candidate URLs file")?;
    
    let reader = BufReader::new(file);
    let mut candidates = Vec::new();
    
    for line in reader.lines() {
        let line = line.context("Failed to read line")?;
        if line.trim().is_empty() {
            continue;
        }
        
        let candidate: CandidateUrl = serde_json::from_str(&line)
            .context("Failed to parse candidate URL JSON")?;
        candidates.push(candidate);
    }
    
    Ok(candidates)
}

fn save_candidates(root: &str, candidates: &[CandidateUrl]) -> Result<()> {
    let path = PathBuf::from(root).join("tracking/candidate_urls.jsonl");
    
    // Create tracking directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create tracking directory")?;
    }
    
    // Write candidates to JSONL file (overwrite)
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .context("Failed to open candidate URLs file")?;
    
    let mut writer = BufWriter::new(file);
    
    for candidate in candidates {
        let json_line = serde_json::to_string(candidate)
            .context("Failed to serialize candidate URL")?;
        writeln!(writer, "{}", json_line)
            .context("Failed to write candidate URL")?;
    }
    
    writer.flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let root = std::env::var("ROOT").unwrap_or_else(|_| ".".to_string());
    
    println!("=== Candidate URL Validator ===");
    println!("Reading candidates from candidate_urls.jsonl...");
    
    // Load candidates
    let candidates = load_candidates(&root)
        .context("Failed to load candidates")?;
    
    println!("Loaded {} candidates", candidates.len());
    
    if candidates.is_empty() {
        println!("No candidates to validate. Exiting.");
        return Ok(());
    }
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;
    
    // Validate each candidate
    let candidates_count = candidates.len();
    let mut validated_candidates: Vec<CandidateUrl> = Vec::new();
    let mut validated_count = 0;
    let mut rejected_count = 0;
    let mut error_count = 0;
    
    for mut candidate in candidates {
        match validate_candidate(&client, &mut candidate).await {
            Ok(is_valid) => {
                if is_valid {
                    validated_candidates.push(candidate);
                    validated_count += 1;
                } else {
                    rejected_count += 1;
                    println!("  Rejected: {} (reason: {})", candidate.url, candidate.reason);
                }
            }
            Err(e) => {
                error_count += 1;
                println!("  Error validating {}: {}", candidate.url, e);
                // Still add to validated list with error tag
                candidate.tags.push("validation_error".to_string());
                validated_candidates.push(candidate);
            }
        }
    }
    
    // Save validated candidates
    if !validated_candidates.is_empty() {
        save_candidates(&root, &validated_candidates)
            .context("Failed to save validated candidates")?;
        println!("Saved {} validated candidates", validated_candidates.len());
    }
    
    println!();
    println!("Validation Summary:");
    println!("  Validated: {}", validated_count);
    println!("  Rejected: {}", rejected_count);
    println!("  Errors: {}", error_count);
    println!("  Total: {}", candidates_count);
    
    Ok(())
}

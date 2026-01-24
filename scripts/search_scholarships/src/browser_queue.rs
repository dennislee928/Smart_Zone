//! Browser Queue Management Module
//!
//! Manages browser_queue.jsonl (URLs that need browser rendering)
//! and browser_results.jsonl (results from Python worker)

use crate::types::Lead;
use crate::js_detector::BrowserDetectionResult;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::collections::HashSet;

/// Browser queue entry (written to browser_queue.jsonl)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserQueueEntry {
    pub url: String,
    pub source_id: String,
    pub source_name: String,
    pub discovered_at: String,
    pub detection_reason: String,
    pub detected_api_endpoints: Vec<String>,
    pub priority: u8,
    pub retry_count: u32,
}

/// Browser result entry (read from browser_results.jsonl)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserResultEntry {
    pub url: String,
    pub source_id: String,
    pub status: String,
    pub leads: Vec<BrowserLead>,
    pub detected_api_endpoints: Vec<ApiEndpoint>,
    pub error: Option<String>,
    pub processed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserLead {
    pub name: String,
    pub amount: String,
    pub deadline: String,
    pub eligibility: Vec<String>,
    pub extraction_evidence: Vec<ExtractionEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEvidence {
    pub attribute: String,
    pub snippet: String,
    pub selector: Option<String>,
    pub xpath: Option<String>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub url: String,
    pub method: String,
    pub response_type: String,
    pub sample_response: Option<serde_json::Value>,
}

/// Write URL to browser queue
pub fn write_to_browser_queue(
    root: &str,
    lead: &Lead,
    detection: &BrowserDetectionResult,
) -> Result<()> {
    let queue_path = PathBuf::from(root).join("tracking").join("browser_queue.jsonl");
    
    // Create tracking directory if it doesn't exist
    if let Some(parent) = queue_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create tracking directory")?;
    }
    
    // Check for duplicates
    let existing_urls = read_queue_urls(&queue_path)?;
    if existing_urls.contains(&lead.url) {
        // URL already in queue, skip
        return Ok(());
    }
    
    // Append to queue file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&queue_path)
        .context("Failed to open browser queue file")?;
    
    let mut writer = BufWriter::new(file);
    
    let entry = BrowserQueueEntry {
        url: lead.url.clone(),
        source_id: lead.source.clone(),
        source_name: lead.source.clone(),
        discovered_at: chrono::Utc::now().to_rfc3339(),
        detection_reason: format!("{:?}", detection.reason),
        detected_api_endpoints: detection.detected_api_endpoints.clone(),
        priority: 1, // Default priority
        retry_count: 0,
    };
    
    let json_line = serde_json::to_string(&entry)
        .context("Failed to serialize browser queue entry")?;
    
    writeln!(writer, "{}", json_line)
        .context("Failed to write to browser queue")?;
    
    writer.flush()?;
    
    Ok(())
}

/// Read all URLs from browser queue (for deduplication)
fn read_queue_urls(queue_path: &PathBuf) -> Result<HashSet<String>> {
    let mut urls = HashSet::new();
    
    if !queue_path.exists() {
        return Ok(urls);
    }
    
    let file = File::open(queue_path)
        .context("Failed to open browser queue file")?;
    
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(entry) = serde_json::from_str::<BrowserQueueEntry>(&line) {
            urls.insert(entry.url);
        }
    }
    
    Ok(urls)
}

/// Read browser results from browser_results.jsonl
pub fn read_browser_results(root: &str) -> Result<Vec<BrowserResultEntry>> {
    let results_path = PathBuf::from(root).join("tracking").join("browser_results.jsonl");
    
    if !results_path.exists() {
        return Ok(vec![]);
    }
    
    let file = File::open(&results_path)
        .context("Failed to open browser results file")?;
    
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(entry) = serde_json::from_str::<BrowserResultEntry>(&line) {
            results.push(entry);
        }
    }
    
    Ok(results)
}

/// Merge browser result into lead
pub fn merge_browser_result(leads: &mut Vec<Lead>, result: BrowserResultEntry) {
    for browser_lead in result.leads {
        // Find matching lead by URL
        if let Some(existing_lead) = leads.iter_mut().find(|l| l.url == result.url) {
            // Merge: prefer browser result if it has more complete data
            let browser_has_amount = !browser_lead.amount.is_empty() && 
                !browser_lead.amount.to_lowercase().contains("see website");
            let browser_has_deadline = !browser_lead.deadline.is_empty() && 
                !browser_lead.deadline.to_lowercase().contains("check website");
            
            let existing_has_amount = !existing_lead.amount.is_empty() && 
                !existing_lead.amount.to_lowercase().contains("see website");
            let existing_has_deadline = !existing_lead.deadline.is_empty() && 
                !existing_lead.deadline.to_lowercase().contains("check website");
            
            // Update if browser result is better
            if browser_has_amount && !existing_has_amount {
                existing_lead.amount = browser_lead.amount;
            }
            
            if browser_has_deadline && !existing_has_deadline {
                existing_lead.deadline = browser_lead.deadline;
            }
            
            if !browser_lead.name.is_empty() && existing_lead.name.is_empty() {
                existing_lead.name = browser_lead.name;
            }
            
            // Merge eligibility
            for elig in browser_lead.eligibility {
                if !existing_lead.eligibility.contains(&elig) {
                    existing_lead.eligibility.push(elig);
                }
            }
            
            // Merge extraction evidence
            for evidence in browser_lead.extraction_evidence {
                // Convert to crate::types::ExtractionEvidence
                let rust_evidence = crate::types::ExtractionEvidence {
                    attribute: evidence.attribute,
                    snippet: evidence.snippet,
                    selector: evidence.selector,
                    url: result.url.clone(),
                    method: evidence.method,
                };
                existing_lead.extraction_evidence.push(rust_evidence);
            }
            
            // Remove pending_browser tag
            existing_lead.tags.retain(|t| t != "pending_browser");
            
            // Update confidence
            if existing_lead.confidence.is_none() || existing_lead.confidence.unwrap() < 0.8 {
                existing_lead.confidence = Some(0.8);
            }
        } else {
            // New lead from browser result
            let mut new_lead = Lead {
                name: browser_lead.name,
                amount: browser_lead.amount,
                deadline: browser_lead.deadline,
                source: result.source_id.clone(),
                source_type: "browser_extracted".to_string(),
                status: "new".to_string(),
                eligibility: browser_lead.eligibility,
                notes: String::new(),
                added_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                url: result.url.clone(),
                match_score: 0,
                match_reasons: vec![],
                hard_fail_reasons: vec![],
                soft_flags: vec![],
                bucket: None,
                http_status: Some(200),
                effort_score: None,
                trust_tier: Some("B".to_string()),
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
                confidence: Some(0.8),
                eligibility_confidence: None,
                tags: vec!["browser_extracted".to_string()],
                is_index_only: false,
                first_seen_at: None,
                last_checked_at: Some(chrono::Utc::now().to_rfc3339()),
                next_check_at: None,
                persistence_status: None,
                source_seed: None,
                check_count: Some(1),
                extraction_evidence: browser_lead.extraction_evidence.into_iter().map(|e| {
                    crate::types::ExtractionEvidence {
                        attribute: e.attribute,
                        snippet: e.snippet,
                        selector: e.selector,
                        url: result.url.clone(),
                        method: e.method,
                    }
                }).collect(),
            };
            
            leads.push(new_lead);
        }
    }
}

/// Clear processed entries from browser queue
pub fn clear_processed_entries(root: &str, processed_urls: &HashSet<String>) -> Result<()> {
    let queue_path = PathBuf::from(root).join("tracking").join("browser_queue.jsonl");
    
    if !queue_path.exists() {
        return Ok(());
    }
    
    // Read all entries
    let file = File::open(&queue_path)?;
    let reader = BufReader::new(file);
    let mut remaining_entries = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(entry) = serde_json::from_str::<BrowserQueueEntry>(&line) {
            if !processed_urls.contains(&entry.url) {
                remaining_entries.push(entry);
            }
        }
    }
    
    // Write back remaining entries
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&queue_path)?;
    
    let mut writer = BufWriter::new(file);
    
    for entry in remaining_entries {
        let json_line = serde_json::to_string(&entry)?;
        writeln!(writer, "{}", json_line)?;
    }
    
    writer.flush()?;
    
    Ok(())
}

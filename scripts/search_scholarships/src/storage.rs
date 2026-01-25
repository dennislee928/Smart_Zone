use std::fs;
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use anyhow::{Result, Context};

use crate::{LeadsFile, Criteria, Sources};
use crate::discovery::CandidateUrl;

pub fn load_criteria(root: &str) -> Result<Criteria> {
    let path = PathBuf::from(root).join("tracking/criteria.yml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read criteria from {:?}", path))?;
    
    let criteria: Criteria = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse criteria YAML")?;
    
    Ok(criteria)
}

pub fn load_sources(root: &str) -> Result<Sources> {
    let path = PathBuf::from(root).join("tracking/sources.yml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read sources from {:?}", path))?;
    
    let sources: Sources = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse sources YAML")?;
    
    Ok(sources)
}

pub fn load_leads(root: &str) -> Result<LeadsFile> {
    let path = PathBuf::from(root).join("tracking/leads.json");
    
    if !path.exists() {
        return Ok(LeadsFile { leads: vec![] });
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read leads from {:?}", path))?;
    
    let leads: LeadsFile = serde_json::from_str(&content)
        .unwrap_or_else(|_| LeadsFile { leads: vec![] });
    
    Ok(leads)
}

pub fn save_leads(root: &str, leads: &LeadsFile) -> Result<()> {
    let path = PathBuf::from(root).join("tracking/leads.json");
    let json = serde_json::to_string_pretty(leads)?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write leads to {:?}", path))?;
    Ok(())
}

/// Save candidate URLs to JSONL file
pub fn save_candidates(root: &str, candidates: &[CandidateUrl]) -> Result<()> {
    let path = PathBuf::from(root).join("tracking/candidate_urls.jsonl");
    
    // Create tracking directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
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

/// Load candidate URLs from JSONL file
pub fn load_candidates(root: &str) -> Result<Vec<CandidateUrl>> {
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

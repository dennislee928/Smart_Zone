use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

use crate::{LeadsFile, Criteria, Sources};

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

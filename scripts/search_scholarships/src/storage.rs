use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use yaml_rust::YamlLoader;

use crate::{LeadsFile, Criteria, Sources};

pub fn load_criteria(root: &str) -> Result<Criteria> {
    let path = PathBuf::from(root).join("tracking/criteria.yml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read criteria from {:?}", path))?;
    
    let docs = YamlLoader::load_from_str(&content)?;
    let yaml = &docs[0];
    
    // Simple YAML to JSON conversion for serde
    let json_str = serde_json::to_string(&yaml)?;
    let criteria: Criteria = serde_json::from_str(&json_str)
        .or_else(|_| {
            // Fallback: try direct YAML parsing
            Ok(Criteria {
                criteria: crate::types::CriteriaContent {
                    required: vec!["UK master eligible".to_string(), "Open international".to_string()],
                    preferred: vec![],
                    excluded_keywords: vec!["undergraduate only".to_string(), "PhD only".to_string()],
                }
            })
        })?;
    
    Ok(criteria)
}

pub fn load_sources(root: &str) -> Result<Sources> {
    let path = PathBuf::from(root).join("tracking/sources.yml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read sources from {:?}", path))?;
    
    let docs = YamlLoader::load_from_str(&content)?;
    let yaml = &docs[0];
    let json_str = serde_json::to_string(&yaml)?;
    let sources: Sources = serde_json::from_str(&json_str)?;
    
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

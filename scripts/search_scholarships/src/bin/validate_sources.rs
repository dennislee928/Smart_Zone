//! Source Validation Binary
//!
//! Validates sources.yml configuration:
//! - Checks scraper types are supported
//! - Checks required fields for each source type
//! - Reports configuration issues before pipeline execution

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct Sources {
    sources: Vec<Source>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Source {
    name: String,
    #[serde(rename = "type")]
    source_type: String,
    url: String,
    enabled: bool,
    scraper: String,
    #[serde(default)]
    priority: Option<u8>,
    #[serde(default)]
    discovery_mode: Option<String>,
    #[serde(default)]
    allow_domains_outbound: Option<Vec<String>>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    max_depth: Option<u8>,
    #[serde(default)]
    deny_patterns: Option<Vec<String>>,
}

fn load_sources(root: &str) -> Result<Sources> {
    let path = PathBuf::from(root).join("tracking/sources.yml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read sources from {:?}", path))?;
    
    let sources: Sources = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse sources YAML")?;
    
    Ok(sources)
}

fn main() -> Result<()> {
    let root = std::env::var("ROOT").unwrap_or_else(|_| ".".to_string());
    
    println!("=== Source Configuration Validator ===");
    
    // Load sources
    let sources = load_sources(&root)
        .context("Failed to load sources.yml")?;
    
    // Supported scraper types
    let supported_scrapers = vec![
        "selenium",
        "university", 
        "government",
        "third_party",
        "foundation",  // Foundation is mapped to third_party, but still valid in config
    ];
    
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    for source in &sources.sources {
        // Check scraper type
        if !supported_scrapers.contains(&source.scraper.as_str()) {
            errors.push(format!(
                "Source '{}' has unsupported scraper type: '{}'. Supported types: {:?}",
                source.name, source.scraper, supported_scrapers
            ));
        }
        
        // Check required fields
        if source.url.is_empty() {
            errors.push(format!("Source '{}' has empty URL", source.name));
        }
        
        if source.name.is_empty() {
            errors.push(format!("Source '{}' has empty name", source.name));
        }
        
        // Check URL format
        if !source.url.starts_with("http://") && !source.url.starts_with("https://") {
            warnings.push(format!(
                "Source '{}' has URL without http/https scheme: {}",
                source.name, source.url
            ));
        }
        
        // Check discovery_seed mode has required fields
        if source.mode.as_deref() == Some("discovery_seed") {
            if source.max_depth.is_none() {
                warnings.push(format!(
                    "Source '{}' is discovery_seed but has no max_depth (will default to 1)",
                    source.name
                ));
            }
        }
    }
    
    // Report results
    if errors.is_empty() && warnings.is_empty() {
        println!("✓ All {} sources are valid", sources.sources.len());
        return Ok(());
    }
    
    if !errors.is_empty() {
        println!("\n❌ ERRORS (must fix):");
        for error in &errors {
            println!("  - {}", error);
        }
    }
    
    if !warnings.is_empty() {
        println!("\n⚠️  WARNINGS:");
        for warning in &warnings {
            println!("  - {}", warning);
        }
    }
    
    if !errors.is_empty() {
        std::process::exit(1);
    }
    
    Ok(())
}

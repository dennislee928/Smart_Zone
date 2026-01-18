mod scrapers;
mod filter;
mod storage;
mod notify;
mod types;

pub use types::*;

use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    let root = std::env::var("ROOT").unwrap_or_else(|_| ".".to_string());
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    
    // Load criteria
    let criteria = storage::load_criteria(&root)?;
    
    // Load sources
    let sources = storage::load_sources(&root)?;
    let enabled_sources: Vec<_> = sources.sources.iter().filter(|s| s.enabled).collect();
    
    // Load existing leads
    let mut leads_file = storage::load_leads(&root)?;
    let existing_names: std::collections::HashSet<String> = leads_file.leads.iter()
        .map(|l| l.name.clone())
        .collect();
    
    // Scrape scholarships
    let mut new_leads = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    
    for source in &enabled_sources {
        println!("Scraping: {} ({})", source.name, source.url);
        match scrapers::scrape_source(source) {
            Ok(scraped) => {
                for mut lead in scraped {
                    // Skip if already exists
                    if existing_names.contains(&lead.name) {
                        continue;
                    }
                    
                    // Filter by criteria
                    if filter::matches_criteria(&lead, &criteria) {
                        lead.status = "qualified".to_string();
                        lead.source_type = source.source_type.clone();
                        lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                        new_leads.push(lead);
                    } else {
                        println!("Filtered out: {}", lead.name);
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to scrape {}: {}", source.name, e);
                println!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    }
    
    // Build report
    let report = build_report(&now, &new_leads, &errors, leads_file.leads.len());
    
    // Save report to file
    fs::write("report.txt", &report)?;
    println!("Report saved to report.txt");
    
    // Add new leads and save
    if !new_leads.is_empty() {
        let mut new_leads_clone = new_leads.clone();
        leads_file.leads.append(&mut new_leads_clone);
        storage::save_leads(&root, &leads_file)?;
    }
    
    // Send notification with report
    notify::send_notifications(&report)?;
    
    Ok(())
}

fn build_report(timestamp: &str, new_leads: &[Lead], errors: &[String], total_leads: usize) -> String {
    let mut report = format!("🔍 **ScholarshipOps Search Report**\n📅 {}\n\n", timestamp);
    
    if new_leads.is_empty() {
        report.push_str("No new scholarships found.\n");
    } else {
        report.push_str(&format!("✅ Found **{}** new qualified scholarships:\n\n", new_leads.len()));
        for lead in new_leads.iter().take(10) {
            report.push_str(&format!(
                "• **{}**\n  💰 {} | ⏰ {}\n  🔗 {}\n\n",
                lead.name, lead.amount, lead.deadline, lead.url
            ));
        }
        if new_leads.len() > 10 {
            report.push_str(&format!("... and {} more\n\n", new_leads.len() - 10));
        }
    }
    
    if !errors.is_empty() {
        report.push_str(&format!("⚠️ {} source(s) failed:\n", errors.len()));
        for err in errors {
            report.push_str(&format!("• {}\n", err));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("📊 Total leads in database: {}", total_leads + new_leads.len()));
    
    report
}

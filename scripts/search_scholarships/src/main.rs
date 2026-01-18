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
    let mut new_leads: Vec<Lead> = Vec::new();
    let mut filtered_out: Vec<(String, Vec<String>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    
    for source in &enabled_sources {
        println!("Scraping: {} ({})", source.name, source.url);
        match scrapers::scrape_source(source) {
            Ok(scraped) => {
                for mut lead in scraped {
                    // Skip if already exists
                    if existing_names.contains(&lead.name) {
                        println!("Skipping duplicate: {}", lead.name);
                        continue;
                    }
                    
                    // Step 1: Basic keyword filtering
                    if !filter::matches_criteria(&lead, &criteria) {
                        println!("Filtered out (keywords): {}", lead.name);
                        filtered_out.push((lead.name.clone(), vec!["Keyword mismatch".to_string()]));
                        continue;
                    }
                    
                    // Step 2: Profile-based qualification filtering
                    if let Some(ref profile) = criteria.profile {
                        if filter::filter_by_profile(&mut lead, profile) {
                            lead.status = "qualified".to_string();
                            lead.source_type = source.source_type.clone();
                            lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                            println!("✅ Qualified: {} (score: {})", lead.name, lead.match_score);
                            new_leads.push(lead);
                        } else {
                            println!("❌ Disqualified: {} - {:?}", lead.name, lead.match_reasons);
                            filtered_out.push((lead.name.clone(), lead.match_reasons.clone()));
                        }
                    } else {
                        // No profile, just use basic filtering
                        lead.status = "qualified".to_string();
                        lead.source_type = source.source_type.clone();
                        lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                        new_leads.push(lead);
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
    
    // Sort by match score (highest first)
    new_leads.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    
    // Build report with profile info
    let report = build_report(&now, &new_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    
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

fn build_report(
    timestamp: &str, 
    new_leads: &[Lead], 
    filtered_out: &[(String, Vec<String>)],
    errors: &[String], 
    total_leads: usize,
    profile: &Option<Profile>,
) -> String {
    let mut report = format!("🔍 **ScholarshipOps Search Report**\n📅 {}\n\n", timestamp);
    
    // Show profile summary
    if let Some(p) = profile {
        report.push_str("👤 **Your Profile:**\n");
        report.push_str(&format!("• 🇹🇼 Nationality: {}\n", p.nationality));
        report.push_str(&format!("• 🎓 Target: {} ({})\n", p.target_university, p.programme_start));
        report.push_str(&format!("• 📚 Level: {}\n", p.programme_level));
        if !p.education.is_empty() {
            let best_gpa = p.education.iter().map(|e| e.gpa).fold(0.0, f64::max);
            report.push_str(&format!("• 📊 Best GPA: {:.2}\n", best_gpa));
        }
        report.push_str("\n");
    }
    
    if new_leads.is_empty() {
        report.push_str("No new qualified scholarships found.\n\n");
    } else {
        report.push_str(&format!("✅ **Found {} qualified scholarships:**\n\n", new_leads.len()));
        
        for (i, lead) in new_leads.iter().take(10).enumerate() {
            report.push_str(&format!(
                "**{}. {}** (Score: {})\n",
                i + 1, lead.name, lead.match_score
            ));
            report.push_str(&format!("💰 {} | ⏰ {}\n", lead.amount, lead.deadline));
            
            // Show match reasons
            if !lead.match_reasons.is_empty() {
                report.push_str(&format!("📋 {}\n", lead.match_reasons.join(" | ")));
            }
            
            report.push_str(&format!("🔗 {}\n\n", lead.url));
        }
        
        if new_leads.len() > 10 {
            report.push_str(&format!("... and {} more scholarships\n\n", new_leads.len() - 10));
        }
    }
    
    // Show filtered out summary
    if !filtered_out.is_empty() {
        report.push_str(&format!("⏭️ **Filtered out:** {} scholarships\n", filtered_out.len()));
        for (name, reasons) in filtered_out.iter().take(5) {
            report.push_str(&format!("• {} - {}\n", name, reasons.join(", ")));
        }
        if filtered_out.len() > 5 {
            report.push_str(&format!("... and {} more\n", filtered_out.len() - 5));
        }
        report.push_str("\n");
    }
    
    if !errors.is_empty() {
        report.push_str(&format!("⚠️ **{} source(s) had issues:**\n", errors.len()));
        for err in errors {
            report.push_str(&format!("• {}\n", err));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("📊 **Total leads in database:** {}", total_leads + new_leads.len()));
    
    report
}

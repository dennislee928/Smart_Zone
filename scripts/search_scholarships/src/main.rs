mod scrapers;
mod filter;
mod storage;
mod notify;
mod types;
mod sorter;

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
    
    // Sort using comprehensive multi-dimensional sorting
    sorter::sort_leads(&mut new_leads);
    
    // Build full report (for file)
    let full_report = build_full_report(&now, &new_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    
    // Build summary report (for Discord, max 2000 chars)
    let summary_report = build_summary_report(&now, &new_leads, &filtered_out, &errors, leads_file.leads.len());
    
    // Save full report to file
    fs::write("report.txt", &full_report)?;
    println!("Report saved to report.txt");
    
    // Save summary for Discord
    fs::write("summary.txt", &summary_report)?;
    println!("Summary saved to summary.txt");
    
    // Add new leads and save
    if !new_leads.is_empty() {
        let mut new_leads_clone = new_leads.clone();
        leads_file.leads.append(&mut new_leads_clone);
        storage::save_leads(&root, &leads_file)?;
    }
    
    // Send summary notification (fits Discord 2000 char limit)
    notify::send_notifications(&summary_report)?;
    
    Ok(())
}

/// Build full detailed report (for file storage)
fn build_full_report(
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
        report.push_str("📊 **Sorting:** Comprehensive score (Match + ROI + Urgency + Source Reliability)\n\n");
        
        // Display ALL scholarships (no limit)
        for (i, lead) in new_leads.iter().enumerate() {
            let comprehensive_score = sorter::calculate_comprehensive_score(lead);
            let roi_score = sorter::calculate_roi_score(lead);
            let urgency_score = sorter::calculate_urgency_score(lead);
            let source_score = sorter::calculate_source_reliability_score(lead);
            let days_until = sorter::days_until_deadline(lead);
            
            report.push_str(&format!(
                "**{}. {}**\n",
                i + 1, lead.name
            ));
            report.push_str(&format!("💰 {} | ⏰ {}\n", lead.amount, lead.deadline));
            
            // Show sorting scores
            report.push_str(&format!("📊 **Scores:** Comprehensive: {:.1} (Match: {} + ROI: £{:.0} + Urgency: {} + Source: {})\n", 
                comprehensive_score, lead.match_score, roi_score, urgency_score, source_score));
            
            if let Some(days) = days_until {
                if days < 0 {
                    report.push_str(&format!("  ⚠️ Deadline passed ({} days ago)\n", -days));
                } else if days <= 30 {
                    report.push_str(&format!("  🚨 URGENT: D-{} days\n", days));
                } else if days <= 60 {
                    report.push_str(&format!("  ⚡ D-{} days (Apply soon)\n", days));
                } else {
                    report.push_str(&format!("  ⏰ D-{} days\n", days));
                }
            }
            
            // Show match reasons
            if !lead.match_reasons.is_empty() {
                report.push_str(&format!("📋 {}\n", lead.match_reasons.join(" | ")));
            }
            
            report.push_str(&format!("🔗 {}\n\n", lead.url));
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

/// Build summary report for Discord (max ~1800 chars to be safe)
fn build_summary_report(
    timestamp: &str,
    new_leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
) -> String {
    let mut report = format!("🔍 **ScholarshipOps Summary**\n📅 {}\n\n", timestamp);
    
    if new_leads.is_empty() {
        report.push_str("No new qualified scholarships found.\n\n");
    } else {
        report.push_str(&format!("✅ **{} qualified scholarships:**\n\n", new_leads.len()));
        
        // Show top 5 scholarships with minimal info
        for (i, lead) in new_leads.iter().take(5).enumerate() {
            let days_until = sorter::days_until_deadline(lead);
            let urgency = match days_until {
                Some(d) if d < 0 => "⚠️PAST".to_string(),
                Some(d) if d <= 30 => format!("🚨D-{}", d),
                Some(d) if d <= 60 => format!("⚡D-{}", d),
                Some(d) => format!("D-{}", d),
                None => "TBD".to_string(),
            };
            
            // Truncate name if too long
            let name = if lead.name.len() > 40 {
                format!("{}...", &lead.name[..37])
            } else {
                lead.name.clone()
            };
            
            report.push_str(&format!(
                "{}. **{}**\n   💰{} | ⏰{} | {}\n\n",
                i + 1, name, lead.amount, lead.deadline, urgency
            ));
        }
        
        if new_leads.len() > 5 {
            report.push_str(&format!("... +{} more (see full report)\n\n", new_leads.len() - 5));
        }
    }
    
    // Brief stats
    report.push_str(&format!("📊 **Stats:** {} qualified | {} filtered | {} errors\n", 
        new_leads.len(), filtered_out.len(), errors.len()));
    report.push_str(&format!("📁 **Total in DB:** {}", total_leads + new_leads.len()));
    
    report
}

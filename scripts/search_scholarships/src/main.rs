mod scrapers;
mod filter;
mod storage;
mod notify;
mod types;
mod sorter;

pub use types::*;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
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
        match scrapers::scrape_source(source).await {
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
    
    // Build HTML and Markdown reports
    let html_report = build_html_report(&now, &new_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    let markdown_report = build_markdown_report(&now, &new_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    
    // Create date-based folder in scripts/productions
    let date_str = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let productions_dir = PathBuf::from(&root).join("scripts").join("productions");
    let report_dir = productions_dir.join(&date_str);
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&report_dir)?;
    println!("Created report directory: {:?}", report_dir);
    
    // Save all three formats
    fs::write(report_dir.join("report.txt"), &full_report)?;
    fs::write(report_dir.join("report.md"), &markdown_report)?;
    fs::write(report_dir.join("report.html"), &html_report)?;
    println!("Saved reports to: {:?}", report_dir);
    
    // Also save summary for Discord (in current directory for workflow)
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

/// Build Markdown format report (full version)
fn build_markdown_report(
    timestamp: &str,
    new_leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
    profile: &Option<Profile>,
) -> String {
    let mut report = format!("# ScholarshipOps Search Report\n\n**Date:** {}\n\n", timestamp);
    
    // Show profile summary
    if let Some(p) = profile {
        report.push_str("## Your Profile\n\n");
        report.push_str(&format!("- **Nationality:** {}\n", p.nationality));
        report.push_str(&format!("- **Target:** {} ({})\n", p.target_university, p.programme_start));
        report.push_str(&format!("- **Level:** {}\n", p.programme_level));
        if !p.education.is_empty() {
            let best_gpa = p.education.iter().map(|e| e.gpa).fold(0.0, f64::max);
            report.push_str(&format!("- **Best GPA:** {:.2}\n", best_gpa));
        }
        report.push_str("\n");
    }
    
    if new_leads.is_empty() {
        report.push_str("## Results\n\nNo new qualified scholarships found.\n\n");
    } else {
        report.push_str(&format!("## Results\n\n**Found {} qualified scholarships**\n\n", new_leads.len()));
        report.push_str("**Sorting:** Comprehensive score (Match + ROI + Urgency + Source Reliability)\n\n");
        
        // Create table header
        report.push_str("| # | Name | Amount | Deadline | Comprehensive Score | Match | ROI | Urgency | Source | Days Until |\n");
        report.push_str("|---|------|--------|---------|-------------------|-------|-----|---------|--------|------------|\n");
        
        // Display ALL scholarships (no limit)
        for (i, lead) in new_leads.iter().enumerate() {
            let comprehensive_score = sorter::calculate_comprehensive_score(lead);
            let roi_score = sorter::calculate_roi_score(lead);
            let urgency_score = sorter::calculate_urgency_score(lead);
            let source_score = sorter::calculate_source_reliability_score(lead);
            let days_until = sorter::days_until_deadline(lead);
            
            let days_str = match days_until {
                Some(d) if d < 0 => format!("⚠️ {} days ago", -d),
                Some(d) if d <= 30 => format!("🚨 D-{}", d),
                Some(d) if d <= 60 => format!("⚡ D-{}", d),
                Some(d) => format!("D-{}", d),
                None => "TBD".to_string(),
            };
            
            // Escape pipe characters in name
            let name_escaped = lead.name.replace("|", "\\|");
            
            report.push_str(&format!(
                "| {} | {} | {} | {} | {:.1} | {} | £{:.0} | {} | {} | {} |\n",
                i + 1, name_escaped, lead.amount, lead.deadline,
                comprehensive_score, lead.match_score, roi_score, urgency_score, source_score, days_str
            ));
        }
        
        report.push_str("\n### Detailed Information\n\n");
        
        // Add detailed information for each scholarship
        for (i, lead) in new_leads.iter().enumerate() {
            let comprehensive_score = sorter::calculate_comprehensive_score(lead);
            let roi_score = sorter::calculate_roi_score(lead);
            let urgency_score = sorter::calculate_urgency_score(lead);
            let source_score = sorter::calculate_source_reliability_score(lead);
            let days_until = sorter::days_until_deadline(lead);
            
            report.push_str(&format!("#### {}. {}\n\n", i + 1, lead.name));
            report.push_str(&format!("- **Amount:** {}\n", lead.amount));
            report.push_str(&format!("- **Deadline:** {}\n", lead.deadline));
            
            if let Some(days) = days_until {
                if days < 0 {
                    report.push_str(&format!("- **Status:** ⚠️ Deadline passed ({} days ago)\n", -days));
                } else if days <= 30 {
                    report.push_str(&format!("- **Status:** 🚨 URGENT: D-{} days\n", days));
                } else if days <= 60 {
                    report.push_str(&format!("- **Status:** ⚡ D-{} days (Apply soon)\n", days));
                } else {
                    report.push_str(&format!("- **Status:** ⏰ D-{} days\n", days));
                }
            }
            
            report.push_str(&format!("- **Scores:** Comprehensive: {:.1} (Match: {} + ROI: £{:.0} + Urgency: {} + Source: {})\n",
                comprehensive_score, lead.match_score, roi_score, urgency_score, source_score));
            
            if !lead.match_reasons.is_empty() {
                report.push_str(&format!("- **Match Reasons:** {}\n", lead.match_reasons.join(" | ")));
            }
            
            report.push_str(&format!("- **URL:** {}\n", lead.url));
            report.push_str("\n");
        }
    }
    
    // Show filtered out summary
    if !filtered_out.is_empty() {
        report.push_str(&format!("## Filtered Out\n\n{} scholarships were filtered out.\n\n", filtered_out.len()));
        for (name, reasons) in filtered_out.iter().take(10) {
            report.push_str(&format!("- **{}:** {}\n", name, reasons.join(", ")));
        }
        if filtered_out.len() > 10 {
            report.push_str(&format!("... and {} more\n", filtered_out.len() - 10));
        }
        report.push_str("\n");
    }
    
    if !errors.is_empty() {
        report.push_str(&format!("## Errors\n\n{} source(s) had issues:\n\n", errors.len()));
        for err in errors {
            report.push_str(&format!("- {}\n", err));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("## Statistics\n\n**Total leads in database:** {}\n", total_leads + new_leads.len()));
    
    report
}

/// Build HTML format report (full version with styling)
fn build_html_report(
    timestamp: &str,
    new_leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
    profile: &Option<Profile>,
) -> String {
    let mut html = String::from("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>ScholarshipOps Search Report</title>\n");
    html.push_str("<style>\n");
    html.push_str(r#"
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; 
               line-height: 1.6; color: #333; background: #f5f5f5; padding: 20px; }
        .container { max-width: 1400px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; margin-bottom: 10px; }
        h2 { color: #34495e; margin-top: 30px; margin-bottom: 15px; border-bottom: 2px solid #3498db; padding-bottom: 5px; }
        .timestamp { color: #7f8c8d; font-size: 0.9em; margin-bottom: 20px; }
        .profile-section, .results-section, .filtered-section, .errors-section, .stats-section { margin-bottom: 30px; }
        ul { margin-left: 20px; }
        li { margin: 5px 0; }
        .table-container { overflow-x: auto; margin: 20px 0; }
        .scholarship-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
        .scholarship-table th { background: #3498db; color: white; padding: 12px; text-align: left; font-weight: 600; position: sticky; top: 0; }
        .scholarship-table td { padding: 10px; border-bottom: 1px solid #ddd; }
        .scholarship-table tr:hover { background: #f8f9fa; }
        .scholarship-table tr.urgent { background: #fff3cd; }
        .scholarship-table tr.soon { background: #d1ecf1; }
        .scholarship-table tr.past { background: #f8d7da; opacity: 0.6; }
        .scholarship-table tr.normal { }
        a { color: #3498db; text-decoration: none; }
        a:hover { text-decoration: underline; }
        @media (max-width: 768px) {
            .container { padding: 15px; }
            .scholarship-table { font-size: 0.8em; }
            .scholarship-table th, .scholarship-table td { padding: 6px; }
        }
    "#);
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    
    html.push_str(&format!("<div class=\"container\">\n"));
    html.push_str(&format!("<h1>🔍 ScholarshipOps Search Report</h1>\n"));
    html.push_str(&format!("<p class=\"timestamp\">📅 {}</p>\n\n", timestamp));
    
    // Show profile summary
    if let Some(p) = profile {
        html.push_str("<div class=\"profile-section\">\n");
        html.push_str("<h2>👤 Your Profile</h2>\n");
        html.push_str("<ul>\n");
        html.push_str(&format!("<li><strong>Nationality:</strong> {}</li>\n", p.nationality));
        html.push_str(&format!("<li><strong>Target:</strong> {} ({})</li>\n", p.target_university, p.programme_start));
        html.push_str(&format!("<li><strong>Level:</strong> {}</li>\n", p.programme_level));
        if !p.education.is_empty() {
            let best_gpa = p.education.iter().map(|e| e.gpa).fold(0.0, f64::max);
            html.push_str(&format!("<li><strong>Best GPA:</strong> {:.2}</li>\n", best_gpa));
        }
        html.push_str("</ul>\n");
        html.push_str("</div>\n\n");
    }
    
    if new_leads.is_empty() {
        html.push_str("<div class=\"results-section\">\n");
        html.push_str("<h2>Results</h2>\n");
        html.push_str("<p>No new qualified scholarships found.</p>\n");
        html.push_str("</div>\n");
    } else {
        html.push_str("<div class=\"results-section\">\n");
        html.push_str(&format!("<h2>✅ Found {} qualified scholarships</h2>\n", new_leads.len()));
        html.push_str("<p><strong>Sorting:</strong> Comprehensive score (Match + ROI + Urgency + Source Reliability)</p>\n");
        
        // Create table
        html.push_str("<div class=\"table-container\">\n");
        html.push_str("<table class=\"scholarship-table\">\n");
        html.push_str("<thead>\n<tr>\n");
        html.push_str("<th>#</th>\n");
        html.push_str("<th>Name</th>\n");
        html.push_str("<th>Amount</th>\n");
        html.push_str("<th>Deadline</th>\n");
        html.push_str("<th>Comprehensive Score</th>\n");
        html.push_str("<th>Match</th>\n");
        html.push_str("<th>ROI</th>\n");
        html.push_str("<th>Urgency</th>\n");
        html.push_str("<th>Source</th>\n");
        html.push_str("<th>Days Until</th>\n");
        html.push_str("<th>URL</th>\n");
        html.push_str("</tr>\n</thead>\n<tbody>\n");
        
        // Display ALL scholarships (no limit)
        for (i, lead) in new_leads.iter().enumerate() {
            let comprehensive_score = sorter::calculate_comprehensive_score(lead);
            let roi_score = sorter::calculate_roi_score(lead);
            let urgency_score = sorter::calculate_urgency_score(lead);
            let source_score = sorter::calculate_source_reliability_score(lead);
            let days_until = sorter::days_until_deadline(lead);
            
            let urgency_class = match days_until {
                Some(d) if d < 0 => "past",
                Some(d) if d <= 30 => "urgent",
                Some(d) if d <= 60 => "soon",
                _ => "normal",
            };
            
            let days_str = match days_until {
                Some(d) if d < 0 => format!("⚠️ {} days ago", -d),
                Some(d) if d <= 30 => format!("🚨 D-{}", d),
                Some(d) if d <= 60 => format!("⚡ D-{}", d),
                Some(d) => format!("D-{}", d),
                None => "TBD".to_string(),
            };
            
            // Escape HTML entities
            let name_escaped = lead.name.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            let amount_escaped = lead.amount.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            
            html.push_str(&format!("<tr class=\"{}\">\n", urgency_class));
            html.push_str(&format!("<td>{}</td>\n", i + 1));
            html.push_str(&format!("<td>{}</td>\n", name_escaped));
            html.push_str(&format!("<td>{}</td>\n", amount_escaped));
            html.push_str(&format!("<td>{}</td>\n", lead.deadline));
            html.push_str(&format!("<td>{:.1}</td>\n", comprehensive_score));
            html.push_str(&format!("<td>{}</td>\n", lead.match_score));
            html.push_str(&format!("<td>£{:.0}</td>\n", roi_score));
            html.push_str(&format!("<td>{}</td>\n", urgency_score));
            html.push_str(&format!("<td>{}</td>\n", source_score));
            html.push_str(&format!("<td>{}</td>\n", days_str));
            html.push_str(&format!("<td><a href=\"{}\" target=\"_blank\">Link</a></td>\n", lead.url));
            html.push_str("</tr>\n");
        }
        
        html.push_str("</tbody>\n</table>\n");
        html.push_str("</div>\n");
        html.push_str("</div>\n");
    }
    
    // Show filtered out summary
    if !filtered_out.is_empty() {
        html.push_str("<div class=\"filtered-section\">\n");
        html.push_str(&format!("<h2>⏭️ Filtered Out</h2>\n"));
        html.push_str(&format!("<p>{} scholarships were filtered out.</p>\n", filtered_out.len()));
        html.push_str("<ul>\n");
        for (name, reasons) in filtered_out.iter().take(10) {
            let name_escaped = name.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            html.push_str(&format!("<li><strong>{}:</strong> {}</li>\n", name_escaped, reasons.join(", ")));
        }
        if filtered_out.len() > 10 {
            html.push_str(&format!("<li>... and {} more</li>\n", filtered_out.len() - 10));
        }
        html.push_str("</ul>\n");
        html.push_str("</div>\n");
    }
    
    if !errors.is_empty() {
        html.push_str("<div class=\"errors-section\">\n");
        html.push_str(&format!("<h2>⚠️ Errors</h2>\n"));
        html.push_str(&format!("<p>{} source(s) had issues:</p>\n", errors.len()));
        html.push_str("<ul>\n");
        for err in errors {
            let err_escaped = err.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            html.push_str(&format!("<li>{}</li>\n", err_escaped));
        }
        html.push_str("</ul>\n");
        html.push_str("</div>\n");
    }
    
    html.push_str("<div class=\"stats-section\">\n");
    html.push_str(&format!("<h2>📊 Statistics</h2>\n"));
    html.push_str(&format!("<p><strong>Total leads in database:</strong> {}</p>\n", total_leads + new_leads.len()));
    html.push_str("</div>\n");
    
    html.push_str("</div>\n</body>\n</html>");
    
    html
}

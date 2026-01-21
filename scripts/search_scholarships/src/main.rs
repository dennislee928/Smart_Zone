//! ScholarshipOps Search & Triage System
//! 
//! Complete pipeline for scholarship discovery and qualification

mod scrapers;
mod filter;
mod storage;
mod notify;
mod types;
mod sorter;
mod rules;
mod link_health;
mod triage;
mod effort;
mod source_health;

pub use types::*;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<()> {
    let root = std::env::var("ROOT").unwrap_or_else(|_| ".".to_string());
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    
    println!("=== ScholarshipOps Search & Triage ===");
    println!("Timestamp: {}", now);
    println!();
    
    // ==========================================
    // Stage 0: Load Configuration
    // ==========================================
    println!("Stage 0: Loading configuration...");
    
    let criteria = storage::load_criteria(&root)?;
    let sources = storage::load_sources(&root)?;
    
    // Load source filter config from environment or use defaults
    let source_filter = load_source_filter_config();
    println!("  Source filter: include={:?}, exclude={:?}, max_failures={}",
        source_filter.include_types,
        source_filter.exclude_types,
        source_filter.max_consecutive_failures
    );
    
    // Load source health tracking
    let mut health_file = source_health::load_health(&root)?;
    println!("  Source health records: {}", health_file.sources.len());
    
    // Filter enabled sources based on type and health
    let enabled_sources: Vec<_> = sources.sources.iter()
        .filter(|s| s.enabled)
        .collect();
    
    // Load rules (optional - continue without if missing)
    let rules_config = match rules::load_rules(&root) {
        Ok(r) => {
            println!("  Loaded {} hard reject, {} soft downgrade, {} positive rules",
                r.hard_reject_rules.len(),
                r.soft_downgrade_rules.len(),
                r.positive_scoring_rules.len()
            );
            Some(r)
        }
        Err(e) => {
            println!("  Warning: Could not load rules.yaml: {}", e);
            None
        }
    };
    
    // Load existing leads
    let mut leads_file = storage::load_leads(&root)?;
    println!("  Existing leads in database: {}", leads_file.leads.len());
    
    // Use name + url as unique key to prevent duplicates
    let existing_keys: HashSet<String> = leads_file.leads.iter()
        .map(|l| format!("{}|{}", l.name.to_lowercase().trim(), l.url.to_lowercase().trim()))
        .collect();
    
    // Track seen leads in this run
    let mut seen_keys: HashSet<String> = HashSet::new();
    
    println!();
    
    // ==========================================
    // Stage 1: Scrape Sources (with health tracking)
    // ==========================================
    let mut skipped_sources: Vec<(String, String)> = Vec::new();
    let sources_to_scrape: Vec<_> = enabled_sources.iter()
        .filter(|s| {
            if let Some(reason) = source_health::should_skip_source(s, &health_file, &source_filter) {
                skipped_sources.push((s.name.clone(), reason));
                false
            } else {
                true
            }
        })
        .collect();
    
    println!("Stage 1: Scraping {} sources ({} skipped)...", 
        sources_to_scrape.len(), skipped_sources.len());
    
    if !skipped_sources.is_empty() {
        println!("  Skipped sources:");
        for (name, reason) in &skipped_sources {
            println!("    - {}: {}", name, reason);
        }
    }
    
    let mut all_leads: Vec<Lead> = Vec::new();
    let mut filtered_out: Vec<(String, Vec<String>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut source_stats = SourceStats::default();
    
    for source in sources_to_scrape {
        println!("  Scraping: {} ({})", source.name, source.url);
        
        match scrapers::scrape_source(source).await {
            Ok(result) => {
                // Update health tracking
                source_health::update_health(
                    &mut health_file, 
                    source, 
                    &result, 
                    source_filter.max_consecutive_failures
                );
                
                // Track stats
                if result.status == SourceStatus::Ok {
                    source_stats.success += 1;
                    println!("    Found {} raw leads", result.leads.len());
                } else {
                    source_stats.failed += 1;
                    println!("    Failed: {}", result.status);
                    if let Some(ref err) = result.error_message {
                        errors.push(format!("{}: {}", source.name, err));
                    }
                    continue; // Skip processing leads from failed sources
                }
                
                let mut added = 0;
                for mut lead in result.leads {
                    // Create unique key
                    let key = format!("{}|{}", lead.name.to_lowercase().trim(), lead.url.to_lowercase().trim());
                    
                    // Skip duplicates
                    if existing_keys.contains(&key) || seen_keys.contains(&key) {
                        continue;
                    }
                    seen_keys.insert(key);
                    
                    // Basic keyword filtering
                    if !filter::matches_criteria(&lead, &criteria) {
                        filtered_out.push((lead.name.clone(), vec!["Keyword mismatch".to_string()]));
                        continue;
                    }
                    
                    // Profile-based filtering
                    if let Some(ref profile) = criteria.profile {
                        if filter::filter_by_profile(&mut lead, profile) {
                            lead.status = "qualified".to_string();
                            lead.source_type = source.source_type.clone();
                            lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                            all_leads.push(lead);
                            added += 1;
                        } else {
                            filtered_out.push((lead.name.clone(), lead.match_reasons.clone()));
                        }
                    } else {
                        lead.status = "qualified".to_string();
                        lead.source_type = source.source_type.clone();
                        lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                        all_leads.push(lead);
                        added += 1;
                    }
                }
                println!("    Added {} qualified leads", added);
            }
            Err(e) => {
                let err_msg = format!("Failed to scrape {}: {}", source.name, e);
                println!("    Error: {}", err_msg);
                errors.push(err_msg);
                source_stats.failed += 1;
            }
        }
    }
    
    source_stats.skipped = skipped_sources.len();
    println!("  Source stats: {} success, {} failed, {} skipped", 
        source_stats.success, source_stats.failed, source_stats.skipped);
    println!("  Total qualified leads: {}", all_leads.len());
    println!();
    
    // ==========================================
    // Stage 2: Link Health Check (optional, skip if too many)
    // ==========================================
    let mut dead_links = Vec::new();
    if all_leads.len() <= 50 {
        println!("Stage 2: Checking link health...");
        dead_links = link_health::check_links(&mut all_leads, 5).await;
        let dead_count = dead_links.iter()
            .filter(|r| matches!(r.status, LinkHealthStatus::NotFound | LinkHealthStatus::ServerError))
            .count();
        println!("  Checked {} URLs, {} dead/error links", dead_links.len(), dead_count);
        println!();
    } else {
        println!("Stage 2: Skipping link health check ({} leads, max 50)", all_leads.len());
        println!();
    }
    
    // ==========================================
    // Stage 3: Effort Scoring
    // ==========================================
    println!("Stage 3: Calculating effort scores...");
    effort::update_effort_scores(&mut all_leads);
    println!("  Updated effort scores for {} leads", all_leads.len());
    println!();
    
    // ==========================================
    // Stage 4: Apply Rules & Triage
    // ==========================================
    let triage_stats;
    if let Some(ref rules) = rules_config {
        println!("Stage 4: Applying rules and triage...");
        triage_stats = triage::triage_leads(&mut all_leads, rules);
        println!("  Bucket A: {} | Bucket B: {} | Bucket C: {}",
            triage_stats.bucket_a, triage_stats.bucket_b, triage_stats.bucket_c);
    } else {
        println!("Stage 4: Skipping rules (no rules.yaml)");
        // Default triage based on score
        for lead in all_leads.iter_mut() {
            lead.bucket = Some(if lead.match_score >= 100 {
                Bucket::A
            } else if lead.match_score >= 50 {
                Bucket::B
            } else {
                Bucket::C
            });
        }
        triage_stats = triage::TriageStats {
            total: all_leads.len(),
            bucket_a: all_leads.iter().filter(|l| l.bucket == Some(Bucket::A)).count(),
            bucket_b: all_leads.iter().filter(|l| l.bucket == Some(Bucket::B)).count(),
            bucket_c: all_leads.iter().filter(|l| l.bucket == Some(Bucket::C)).count(),
            ..Default::default()
        };
    }
    println!();
    
    // ==========================================
    // Stage 5: Sort & Finalize
    // ==========================================
    println!("Stage 5: Sorting leads...");
    sorter::sort_leads(&mut all_leads);
    
    // Split into buckets
    let (bucket_a, bucket_b, bucket_c) = triage::split_by_bucket(all_leads.clone());
    let watchlist: Vec<Lead> = all_leads.iter()
        .filter(|l| l.deadline.to_lowercase().contains("check") || l.deadline.to_lowercase().contains("tbd"))
        .cloned()
        .collect();
    
    println!("  Final: A={}, B={}, C={}, Watchlist={}",
        bucket_a.len(), bucket_b.len(), bucket_c.len(), watchlist.len());
    println!();
    
    // ==========================================
    // Stage 6: Generate Reports
    // ==========================================
    println!("Stage 6: Generating reports...");
    
    // Create output directory
    let date_str = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let productions_dir = PathBuf::from(&root).join("scripts").join("productions");
    let report_dir = productions_dir.join(&date_str);
    fs::create_dir_all(&report_dir)?;
    
    // Generate all reports
    let triage_md = triage::generate_triage_md(&bucket_a, &bucket_b, &bucket_c, &watchlist);
    let triage_csv = triage::generate_triage_csv(&bucket_a, &bucket_b, &bucket_c);
    let deadlinks_md = link_health::generate_deadlinks_report(&dead_links);
    let health_report_md = source_health::generate_health_report(&health_file);
    
    let full_report = build_full_report(&now, &all_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    let summary_report = build_summary_report(&now, &bucket_a, &bucket_b, &bucket_c, &filtered_out, &errors, leads_file.leads.len());
    let markdown_report = build_markdown_report(&now, &all_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    let html_report = build_html_report(&now, &all_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile);
    
    // Save reports
    fs::write(report_dir.join("triage.md"), &triage_md)?;
    fs::write(report_dir.join("triage.csv"), &triage_csv)?;
    fs::write(report_dir.join("deadlinks.md"), &deadlinks_md)?;
    fs::write(report_dir.join("source_health.md"), &health_report_md)?;
    fs::write(report_dir.join("report.txt"), &full_report)?;
    fs::write(report_dir.join("report.md"), &markdown_report)?;
    fs::write(report_dir.join("report.html"), &html_report)?;
    
    // Generate rules audit if rules were loaded
    if let Some(ref rules) = rules_config {
        let audit = triage::generate_rules_audit(rules, &triage_stats);
        let audit_json = serde_json::to_string_pretty(&audit)?;
        fs::write(report_dir.join("rules.audit.json"), &audit_json)?;
    }
    
    println!("  Saved reports to: {:?}", report_dir);
    
    // Save summary for Discord
    fs::write("summary.txt", &summary_report)?;
    
    // ==========================================
    // Stage 7: Update Database & Health Tracking
    // ==========================================
    println!();
    println!("Stage 7: Updating database and health tracking...");
    
    // Save source health tracking
    source_health::save_health(&root, &health_file)?;
    let disabled_count = health_file.sources.iter().filter(|h| h.auto_disabled).count();
    println!("  Updated source health ({} auto-disabled)", disabled_count);
    
    // Only save A and B bucket leads to database
    let leads_to_save: Vec<Lead> = all_leads.iter()
        .filter(|l| matches!(l.bucket, Some(Bucket::A) | Some(Bucket::B)))
        .cloned()
        .collect();
    
    let saved_count = leads_to_save.len();
    if !leads_to_save.is_empty() {
        let mut new_leads = leads_to_save;
        leads_file.leads.append(&mut new_leads);
        storage::save_leads(&root, &leads_file)?;
        println!("  Added {} leads to database", saved_count);
    }
    
    // Send notification
    println!("  Sending notification...");
    notify::send_notifications(&summary_report)?;
    
    println!();
    println!("=== Complete ===");
    
    Ok(())
}

// ==========================================
// Source Stats & Filter Config
// ==========================================

#[derive(Default)]
struct SourceStats {
    success: usize,
    failed: usize,
    skipped: usize,
}

/// Load source filter config from environment variables
fn load_source_filter_config() -> SourceFilterConfig {
    let include_types: Vec<String> = std::env::var("SOURCE_INCLUDE_TYPES")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    let exclude_types: Vec<String> = std::env::var("SOURCE_EXCLUDE_TYPES")
        .unwrap_or_else(|_| "web3".to_string()) // Default: exclude web3
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    let max_failures: u32 = std::env::var("SOURCE_MAX_FAILURES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    
    let skip_disabled: bool = std::env::var("SOURCE_SKIP_DISABLED")
        .map(|s| s != "false" && s != "0")
        .unwrap_or(true);
    
    SourceFilterConfig {
        include_types,
        exclude_types,
        max_consecutive_failures: max_failures,
        skip_auto_disabled: skip_disabled,
    }
}

// ==========================================
// Report Generation Functions
// ==========================================

fn build_full_report(
    timestamp: &str, 
    leads: &[Lead], 
    filtered_out: &[(String, Vec<String>)],
    errors: &[String], 
    total_leads: usize,
    profile: &Option<Profile>,
) -> String {
    let mut report = format!("🔍 **ScholarshipOps Search Report**\n📅 {}\n\n", timestamp);
    
    if let Some(p) = profile {
        report.push_str("👤 **Your Profile:**\n");
        report.push_str(&format!("• Nationality: {}\n", p.nationality));
        report.push_str(&format!("• Target: {} ({})\n", p.target_university, p.programme_start));
        report.push_str(&format!("• Level: {}\n", p.programme_level));
        report.push_str("\n");
    }
    
    // Group by bucket
    let bucket_a: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::A)).collect();
    let bucket_b: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::B)).collect();
    let bucket_c: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::C) || l.bucket.is_none()).collect();
    
    report.push_str(&format!("📊 **Results:** A={} | B={} | C={}\n\n", bucket_a.len(), bucket_b.len(), bucket_c.len()));
    
    if !bucket_a.is_empty() {
        report.push_str("## 🎯 Bucket A (主攻)\n\n");
        for (i, lead) in bucket_a.iter().enumerate() {
            report.push_str(&format!("{}. **{}**\n", i + 1, lead.name));
            report.push_str(&format!("   💰 {} | ⏰ {} | Score: {}\n", lead.amount, lead.deadline, lead.match_score));
            report.push_str(&format!("   🔗 {}\n\n", lead.url));
        }
    }
    
    if !bucket_b.is_empty() {
        report.push_str("## 📋 Bucket B (備援)\n\n");
        for (i, lead) in bucket_b.iter().take(10).enumerate() {
            report.push_str(&format!("{}. {} - {} (Score: {})\n", i + 1, lead.name, lead.amount, lead.match_score));
        }
        if bucket_b.len() > 10 {
            report.push_str(&format!("... and {} more\n", bucket_b.len() - 10));
        }
        report.push_str("\n");
    }
    
    if !filtered_out.is_empty() {
        report.push_str(&format!("⏭️ **Filtered out:** {} scholarships\n\n", filtered_out.len()));
    }
    
    if !errors.is_empty() {
        report.push_str(&format!("⚠️ **Errors:** {}\n\n", errors.len()));
    }
    
    report.push_str(&format!("📁 **Total in database:** {}", total_leads + bucket_a.len() + bucket_b.len()));
    
    report
}

fn build_summary_report(
    timestamp: &str,
    bucket_a: &[Lead],
    bucket_b: &[Lead],
    bucket_c: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
) -> String {
    let mut report = format!("🔍 **ScholarshipOps Triage**\n📅 {}\n\n", timestamp);
    
    report.push_str(&format!("📊 A={} | B={} | C={}\n\n", bucket_a.len(), bucket_b.len(), bucket_c.len()));
    
    if !bucket_a.is_empty() {
        report.push_str("🎯 **Top Picks:**\n");
        for (i, lead) in bucket_a.iter().take(3).enumerate() {
            let name = if lead.name.len() > 35 {
                format!("{}...", &lead.name[..32])
            } else {
                lead.name.clone()
            };
            report.push_str(&format!("{}. {} | {}\n", i + 1, name, lead.amount));
        }
        if bucket_a.len() > 3 {
            report.push_str(&format!("   +{} more in A\n", bucket_a.len() - 3));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("📁 DB: {} | ⏭️ Filtered: {} | ⚠️ Errors: {}", 
        total_leads + bucket_a.len() + bucket_b.len(), 
        filtered_out.len(), 
        errors.len()
    ));
    
    report
}

fn build_markdown_report(
    timestamp: &str,
    leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
    profile: &Option<Profile>,
) -> String {
    let mut report = format!("# ScholarshipOps Search Report\n\n**Date:** {}\n\n", timestamp);
    
    if let Some(p) = profile {
        report.push_str("## Your Profile\n\n");
        report.push_str(&format!("- **Nationality:** {}\n", p.nationality));
        report.push_str(&format!("- **Target:** {} ({})\n", p.target_university, p.programme_start));
        report.push_str(&format!("- **Level:** {}\n", p.programme_level));
        report.push_str("\n");
    }
    
    // Group by bucket
    let bucket_a: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::A)).collect();
    let bucket_b: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::B)).collect();
    
    report.push_str("## Results\n\n");
    report.push_str(&format!("- **Bucket A (主攻):** {} scholarships\n", bucket_a.len()));
    report.push_str(&format!("- **Bucket B (備援):** {} scholarships\n", bucket_b.len()));
    report.push_str(&format!("- **Filtered out:** {} scholarships\n", filtered_out.len()));
    report.push_str("\n");
    
    if !bucket_a.is_empty() {
        report.push_str("### Bucket A - High Priority\n\n");
        report.push_str("| # | Name | Amount | Deadline | Score | Effort |\n");
        report.push_str("|---|------|--------|----------|-------|--------|\n");
        
        for (i, lead) in bucket_a.iter().enumerate() {
            let effort = lead.effort_score.map(|e| format!("{}/100", e)).unwrap_or("-".to_string());
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score, effort
            ));
        }
        report.push_str("\n");
    }
    
    if !bucket_b.is_empty() {
        report.push_str("### Bucket B - Medium Priority\n\n");
        report.push_str("| # | Name | Amount | Deadline | Score |\n");
        report.push_str("|---|------|--------|----------|-------|\n");
        
        for (i, lead) in bucket_b.iter().take(20).enumerate() {
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score
            ));
        }
        if bucket_b.len() > 20 {
            report.push_str(&format!("\n*... and {} more*\n", bucket_b.len() - 20));
        }
        report.push_str("\n");
    }
    
    if !errors.is_empty() {
        report.push_str("## Errors\n\n");
        for err in errors {
            report.push_str(&format!("- {}\n", err));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("## Statistics\n\n**Total leads in database:** {}\n", 
        total_leads + bucket_a.len() + bucket_b.len()));
    
    report
}

fn build_html_report(
    timestamp: &str,
    leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    _errors: &[String],
    total_leads: usize,
    profile: &Option<Profile>,
) -> String {
    let bucket_a: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::A)).collect();
    let bucket_b: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::B)).collect();
    
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ScholarshipOps Triage Report</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
       line-height: 1.6; color: #333; background: #f5f5f5; padding: 20px; }
.container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
h1 { color: #2c3e50; margin-bottom: 10px; }
h2 { color: #34495e; margin-top: 30px; margin-bottom: 15px; border-bottom: 2px solid #3498db; padding-bottom: 5px; }
.timestamp { color: #7f8c8d; font-size: 0.9em; margin-bottom: 20px; }
.stats { display: flex; gap: 20px; margin: 20px 0; }
.stat-box { background: #ecf0f1; padding: 15px 25px; border-radius: 8px; text-align: center; }
.stat-box.a { background: #d5f5e3; border-left: 4px solid #27ae60; }
.stat-box.b { background: #fdebd0; border-left: 4px solid #f39c12; }
.stat-box.c { background: #fadbd8; border-left: 4px solid #e74c3c; }
.stat-number { font-size: 2em; font-weight: bold; }
table { width: 100%; border-collapse: collapse; margin: 20px 0; }
th { background: #3498db; color: white; padding: 12px; text-align: left; }
td { padding: 10px; border-bottom: 1px solid #ddd; }
tr:hover { background: #f8f9fa; }
a { color: #3498db; }
.bucket-a { background: #d5f5e3; }
.bucket-b { background: #fdebd0; }
</style>
</head>
<body>
<div class="container">
"#);
    
    html.push_str(&format!("<h1>🔍 ScholarshipOps Triage Report</h1>\n"));
    html.push_str(&format!("<p class=\"timestamp\">📅 {}</p>\n", timestamp));
    
    if let Some(p) = profile {
        html.push_str("<h2>👤 Your Profile</h2>\n<ul>\n");
        html.push_str(&format!("<li><strong>Nationality:</strong> {}</li>\n", p.nationality));
        html.push_str(&format!("<li><strong>Target:</strong> {} ({})</li>\n", p.target_university, p.programme_start));
        html.push_str(&format!("<li><strong>Level:</strong> {}</li>\n", p.programme_level));
        html.push_str("</ul>\n");
    }
    
    html.push_str("<div class=\"stats\">\n");
    html.push_str(&format!("<div class=\"stat-box a\"><div class=\"stat-number\">{}</div>Bucket A</div>\n", bucket_a.len()));
    html.push_str(&format!("<div class=\"stat-box b\"><div class=\"stat-number\">{}</div>Bucket B</div>\n", bucket_b.len()));
    html.push_str(&format!("<div class=\"stat-box c\"><div class=\"stat-number\">{}</div>Filtered</div>\n", filtered_out.len()));
    html.push_str("</div>\n");
    
    if !bucket_a.is_empty() {
        html.push_str("<h2>🎯 Bucket A - High Priority</h2>\n");
        html.push_str("<table><thead><tr><th>#</th><th>Name</th><th>Amount</th><th>Deadline</th><th>Score</th><th>Effort</th><th>Link</th></tr></thead><tbody>\n");
        
        for (i, lead) in bucket_a.iter().enumerate() {
            let effort = lead.effort_score.map(|e| format!("{}/100", e)).unwrap_or("-".to_string());
            html.push_str(&format!(
                "<tr class=\"bucket-a\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><a href=\"{}\" target=\"_blank\">Link</a></td></tr>\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score, effort, lead.url
            ));
        }
        html.push_str("</tbody></table>\n");
    }
    
    if !bucket_b.is_empty() {
        html.push_str("<h2>📋 Bucket B - Medium Priority</h2>\n");
        html.push_str("<table><thead><tr><th>#</th><th>Name</th><th>Amount</th><th>Deadline</th><th>Score</th><th>Link</th></tr></thead><tbody>\n");
        
        for (i, lead) in bucket_b.iter().take(30).enumerate() {
            html.push_str(&format!(
                "<tr class=\"bucket-b\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><a href=\"{}\" target=\"_blank\">Link</a></td></tr>\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score, lead.url
            ));
        }
        html.push_str("</tbody></table>\n");
        
        if bucket_b.len() > 30 {
            html.push_str(&format!("<p><em>... and {} more</em></p>\n", bucket_b.len() - 30));
        }
    }
    
    html.push_str(&format!("<h2>📊 Statistics</h2>\n<p><strong>Total leads in database:</strong> {}</p>\n", 
        total_leads + bucket_a.len() + bucket_b.len()));
    
    html.push_str("</div>\n</body>\n</html>");
    
    html
}

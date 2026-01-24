//! Triage Module
//! 
//! Implements A/B/C bucket classification and generates triage reports
//!
//! Bucketing Rules:
//! - A (Apply Today): deadline <= 30 days AND confidence >= 0.7
//! - B (Prepare): 31-90 days OR confidence 0.5-0.7
//! - Watchlist: deadline unknown/annual OR confidence < 0.5
//! - C (Rejected): Hard reject from rules engine
//! - X (Missed): Deadline passed

use crate::types::{Lead, Bucket, RulesConfig, RulesAudit, BucketCounts, RuleHitCount};
use crate::rules::apply_rules;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;

/// Bucket thresholds for deadline-based classification
const APPLY_NOW_DAYS: i64 = 30;
const PREPARE_MAX_DAYS: i64 = 90;
const HIGH_CONFIDENCE: f32 = 0.7;
const MED_CONFIDENCE: f32 = 0.5;

/// Perform triage on leads using rules engine + deadline/confidence logic
pub fn triage_leads(leads: &mut [Lead], rules: &RulesConfig) -> TriageStats {
    let mut stats = TriageStats::default();
    let mut rule_hits: HashMap<String, usize> = HashMap::new();
    
    let today = Utc::now().date_naive();
    
    for lead in leads.iter_mut() {
        // First, calculate confidence if not set
        if lead.confidence.is_none() {
            lead.confidence = Some(calculate_confidence(lead));
        }
        
        // Apply rules engine
        let result = apply_rules(lead, rules);
        
        // Update match score
        lead.match_score += result.total_score_add;
        // Apply effort adjustments (reduce from positive rules, add from soft e.g. B-EFFORT-VIDEO-001)
        let eff = lead.effort_score.unwrap_or(0);
        lead.effort_score = Some((eff - result.total_effort_reduce + result.total_effort_add).clamp(0, 100));
        
        // Track matched rules and populate reason fields separately
        for rule_match in &result.matched_rules {
            lead.matched_rule_ids.push(rule_match.rule_id.clone());
            *rule_hits.entry(rule_match.rule_id.clone()).or_insert(0) += 1;
        }
        
        // Populate reason fields separately
        lead.match_reasons.extend(result.match_reasons.clone());
        lead.hard_fail_reasons.extend(result.hard_fail_reasons.clone());
        lead.soft_flags.extend(result.soft_flags.clone());
        
        // Determine final bucket: rules engine takes precedence, then deadline/confidence
        lead.bucket = if result.hard_rejected {
            // Hard reject from rules engine
            // Store rejection reason in hard_fail_reasons (already populated above)
            result.bucket
        } else {
            // Apply deadline + confidence based bucketing
            determine_bucket_by_deadline_and_confidence(lead, result.bucket, today)
        };
        
        // Update stats
        match lead.bucket {
            Some(Bucket::A) => stats.bucket_a += 1,
            Some(Bucket::B) => stats.bucket_b += 1,
            Some(Bucket::C) => stats.bucket_c += 1,
            Some(Bucket::X) => stats.bucket_x += 1,
            None => stats.bucket_c += 1, // Default to C
        }
        
        if result.add_to_watchlist || is_watchlist_candidate(lead) {
            stats.watchlist += 1;
        }
    }
    
    stats.rule_hits = rule_hits;
    stats.total = leads.len();
    stats
}

/// Calculate confidence score for a lead (0.0 - 1.0)
fn calculate_confidence(lead: &Lead) -> f32 {
    let mut score: f32 = 0.0;
    let mut max_score: f32 = 0.0;
    
    // Deadline confidence (30%)
    max_score += 0.3;
    if lead.deadline_date.is_some() {
        score += 0.3;
    } else if !lead.deadline.is_empty() && lead.deadline != "Check website" && lead.deadline != "TBD" {
        score += 0.15;
    }
    
    // Eligibility data (25%)
    max_score += 0.25;
    if lead.is_taiwan_eligible == Some(true) {
        score += 0.25;
    } else if !lead.eligibility.is_empty() {
        score += 0.1;
    }
    
    // Trust tier (20%)
    max_score += 0.2;
    match lead.trust_tier.as_deref() {
        Some("S") => score += 0.2,
        Some("A") => score += 0.15,
        Some("B") => score += 0.1,
        _ => {}
    }
    
    // HTTP status (15%)
    max_score += 0.15;
    if lead.http_status == Some(200) {
        score += 0.15;
    } else if lead.http_status.is_some() && lead.http_status != Some(404) && lead.http_status != Some(410) {
        score += 0.05;
    }
    
    // Amount specified (10%)
    max_score += 0.1;
    if !lead.amount.is_empty() && lead.amount != "See website" {
        score += 0.1;
    }
    
    // Normalize to 0-1
    if max_score > 0.0 {
        score / max_score
    } else {
        0.0
    }
}

/// Determine bucket based on deadline days and confidence
fn determine_bucket_by_deadline_and_confidence(
    lead: &Lead, 
    rules_bucket: Option<Bucket>,
    today: NaiveDate,
) -> Option<Bucket> {
    let confidence = lead.confidence.unwrap_or(0.0);
    
    // If rules engine already assigned B (soft downgrade), respect that
    if rules_bucket == Some(Bucket::B) {
        return Some(Bucket::B);
    }
    
    // Parse deadline
    let deadline_str = lead.deadline_date.as_deref().unwrap_or(&lead.deadline);
    let deadline_date = parse_deadline_date(deadline_str);
    
    if let Some(dl_date) = deadline_date {
        let days_until = (dl_date - today).num_days();
        
        if days_until < 0 {
            // Deadline passed
            return Some(Bucket::X);
        } else if days_until <= APPLY_NOW_DAYS && confidence >= HIGH_CONFIDENCE {
            // Apply now: <= 30 days AND high confidence
            return Some(Bucket::A);
        } else if days_until <= PREPARE_MAX_DAYS || confidence >= MED_CONFIDENCE {
            // Prepare: 31-90 days OR medium confidence
            return Some(Bucket::B);
        } else if days_until > PREPARE_MAX_DAYS {
            // Too far out for immediate action
            return Some(Bucket::B);
        }
    } else {
        // No deadline - use confidence alone
        if confidence >= HIGH_CONFIDENCE {
            return Some(Bucket::B); // Good confidence but unknown deadline
        } else if confidence < MED_CONFIDENCE {
            return Some(Bucket::C); // Low confidence
        }
    }
    
    // Default based on score if confidence/deadline didn't determine
    rules_bucket.or(Some(Bucket::B))
}

/// Check if lead should be on watchlist (deadline unknown or annual cycle)
fn is_watchlist_candidate(lead: &Lead) -> bool {
    let dl = lead.deadline.to_lowercase();
    lead.deadline_date.is_none() && 
        (dl.is_empty() || dl.contains("check") || dl.contains("tbd") || dl.contains("annual") || dl.contains("rolling"))
}

/// Parse deadline string to NaiveDate with validation
/// Rejects invalid dates like "68-58-58" by validating year (2020-2100), month (1-12), and day validity
fn parse_deadline_date(deadline: &str) -> Option<NaiveDate> {
    let formats = [
        "%Y-%m-%d",
        "%d/%m/%Y",
        "%m/%d/%Y",
        "%d %B %Y",
        "%B %d, %Y",
        "%d-%m-%Y",
    ];
    
    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(deadline, fmt) {
            // Validate date: check year, month, and day validity
            let year = date.year();
            let month = date.month();
            let day = date.day();
            
            // Validate year range (2020-2100)
            if year < 2020 || year > 2100 {
                continue; // Try next format
            }
            
            // Validate month range (1-12)
            if month < 1 || month > 12 {
                continue; // Try next format
            }
            
            // Validate day - from_ymd_opt would have failed if invalid, but double-check
            if NaiveDate::from_ymd_opt(year, month, day).is_some() {
                return Some(date);
            }
        }
    }
    
    // Try regex for embedded dates
    // This handles cases like "68-58-58" which should be rejected
    if let Ok(re) = regex::Regex::new(r"(\d{1,4})-(\d{1,2})-(\d{1,2})") {
        if let Some(caps) = re.captures(deadline) {
            let year: i32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let day: u32 = caps[3].parse().ok()?;
            
            // Validate year range (2020-2100)
            if year < 2020 || year > 2100 {
                return None;
            }
            
            // Validate month range (1-12)
            if month < 1 || month > 12 {
                return None;
            }
            
            // Validate day - from_ymd_opt will return None for invalid dates (e.g., Feb 30)
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Some(date);
            }
        }
    }
    
    None
}

/// Split leads into buckets (A, B, C, X)
pub fn split_by_bucket(leads: Vec<Lead>) -> (Vec<Lead>, Vec<Lead>, Vec<Lead>, Vec<Lead>) {
    let mut bucket_a = Vec::new();
    let mut bucket_b = Vec::new();
    let mut bucket_c = Vec::new();
    let mut bucket_x = Vec::new();
    
    for lead in leads {
        match lead.bucket {
            Some(Bucket::A) => bucket_a.push(lead),
            Some(Bucket::B) => bucket_b.push(lead),
            Some(Bucket::X) => bucket_x.push(lead),
            Some(Bucket::C) | None => bucket_c.push(lead),
        }
    }
    
    // Sort each bucket by score
    bucket_a.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    bucket_b.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    bucket_c.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    bucket_x.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    
    (bucket_a, bucket_b, bucket_c, bucket_x)
}

/// Generate triage.md report
pub fn generate_triage_md(
    bucket_a: &[Lead],
    bucket_b: &[Lead],
    bucket_c: &[Lead],
    bucket_x: &[Lead],
    watchlist: &[Lead],
) -> String {
    let mut report = String::from("# ScholarshipOps Triage Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M UTC")));
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **A (主攻):** {} scholarships\n", bucket_a.len()));
    report.push_str(&format!("- **B (備援):** {} scholarships\n", bucket_b.len()));
    report.push_str(&format!("- **C (淘汰):** {} scholarships\n", bucket_c.len()));
    if !bucket_x.is_empty() {
        report.push_str(&format!("- **X (已截止):** {} scholarships (saved for next cycle)\n", bucket_x.len()));
    }
    if !watchlist.is_empty() {
        report.push_str(&format!("- **Watchlist:** {} scholarships\n", watchlist.len()));
    }
    report.push_str("\n---\n\n");
    
    // Bucket A - High Priority
    report.push_str("## A: 主攻 (High Priority)\n\n");
    if bucket_a.is_empty() {
        report.push_str("*No scholarships in this bucket*\n\n");
    } else {
        report.push_str("| # | Name | Amount | Deadline | Score | Effort | Trust | Reason |\n");
        report.push_str("|---|------|--------|----------|-------|--------|-------|--------|\n");
        
        for (i, lead) in bucket_a.iter().enumerate() {
            let effort = lead.effort_score.map(|e| e.to_string()).unwrap_or("-".to_string());
            let trust = lead.trust_tier.as_deref().unwrap_or("-");
            let reasons = if lead.match_reasons.is_empty() {
                "-".to_string()
            } else {
                lead.match_reasons.iter().take(2).cloned().collect::<Vec<_>>().join(", ")
            };
            
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                i + 1, 
                truncate_str(&lead.name, 40),
                lead.amount,
                lead.deadline,
                lead.match_score,
                effort,
                trust,
                truncate_str(&reasons, 30)
            ));
        }
        report.push_str("\n");
        
        // Detailed info for A bucket
        report.push_str("### Detailed Information\n\n");
        for (i, lead) in bucket_a.iter().enumerate() {
            report.push_str(&format!("#### {}. {}\n\n", i + 1, lead.name));
            report.push_str(&format!("- **Amount:** {}\n", lead.amount));
            report.push_str(&format!("- **Deadline:** {}\n", lead.deadline));
            report.push_str(&format!("- **Score:** {}\n", lead.match_score));
            if let Some(effort) = lead.effort_score {
                report.push_str(&format!("- **Effort Score:** {}/100\n", effort));
            }
            if !lead.match_reasons.is_empty() {
                report.push_str(&format!("- **Why:** {}\n", lead.match_reasons.join(" | ")));
            }
            report.push_str(&format!("- **URL:** {}\n", lead.url));
            report.push_str("\n");
        }
    }
    
    report.push_str("---\n\n");
    
    // Bucket B - Medium Priority
    report.push_str("## B: 備援 (Needs Verification)\n\n");
    if bucket_b.is_empty() {
        report.push_str("*No scholarships in this bucket*\n\n");
    } else {
        report.push_str("| # | Name | Amount | Deadline | Score | Reason |\n");
        report.push_str("|---|------|--------|----------|-------|--------|\n");
        
        for (i, lead) in bucket_b.iter().take(20).enumerate() {
            let reasons = if lead.match_reasons.is_empty() {
                "-".to_string()
            } else {
                lead.match_reasons.first().cloned().unwrap_or("-".to_string())
            };
            
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                i + 1,
                truncate_str(&lead.name, 40),
                lead.amount,
                lead.deadline,
                lead.match_score,
                truncate_str(&reasons, 40)
            ));
        }
        
        if bucket_b.len() > 20 {
            report.push_str(&format!("\n*... and {} more*\n", bucket_b.len() - 20));
        }
        report.push_str("\n");
    }
    
    report.push_str("---\n\n");
    
    // Bucket C - Rejected (summary only)
    report.push_str("## C: 淘汰 (Rejected)\n\n");
    if bucket_c.is_empty() {
        report.push_str("*No scholarships in this bucket*\n\n");
    } else {
        report.push_str(&format!("{} scholarships were rejected.\n\n", bucket_c.len()));
        
        // Group by rejection reason
        // For C bucket, use hard_fail_reasons (hard reject reasons) or matched_rule_ids
        let mut reason_counts: HashMap<String, usize> = HashMap::new();
        for lead in bucket_c {
            // Priority: use hard_fail_reasons first, then matched_rule_ids
            let reason = if !lead.hard_fail_reasons.is_empty() {
                lead.hard_fail_reasons[0].clone()
            } else if !lead.matched_rule_ids.is_empty() {
                format!("Rule: {}", lead.matched_rule_ids[0])
            } else {
                "Unknown reason".to_string()
            };
            *reason_counts.entry(reason).or_insert(0) += 1;
        }
        
        report.push_str("**Rejection reasons:**\n\n");
        let mut reasons: Vec<_> = reason_counts.into_iter().collect();
        reasons.sort_by(|a, b| b.1.cmp(&a.1));
        
        for (reason, count) in reasons.iter().take(10) {
            report.push_str(&format!("- {} ({})\n", reason, count));
        }
        report.push_str("\n");
    }
    
    // Bucket X - Missed/Closed (saved for next cycle)
    if !bucket_x.is_empty() {
        report.push_str("---\n\n");
        report.push_str("## X: 已截止 (Missed - Next Cycle)\n\n");
        report.push_str("These scholarships have passed their deadline but are saved for next application cycle:\n\n");
        
        report.push_str("| # | Name | Amount | Deadline | Score | URL |\n");
        report.push_str("|---|------|--------|----------|-------|-----|\n");
        
        for (i, lead) in bucket_x.iter().take(15).enumerate() {
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} | [Link]({}) |\n",
                i + 1,
                truncate_str(&lead.name, 40),
                lead.amount,
                lead.deadline,
                lead.match_score,
                lead.url
            ));
        }
        
        if bucket_x.len() > 15 {
            report.push_str(&format!("\n*... and {} more*\n", bucket_x.len() - 15));
        }
        report.push_str("\n");
    }
    
    // Watchlist
    if !watchlist.is_empty() {
        report.push_str("---\n\n");
        report.push_str("## Watchlist\n\n");
        report.push_str("These scholarships need monitoring (deadline unknown or eligibility unclear):\n\n");
        
        for lead in watchlist.iter().take(10) {
            report.push_str(&format!("- **{}** - {}\n", lead.name, lead.url));
        }
        
        if watchlist.len() > 10 {
            report.push_str(&format!("\n*... and {} more*\n", watchlist.len() - 10));
        }
    }
    
    report
}

/// Generate triage.csv
pub fn generate_triage_csv(
    bucket_a: &[Lead],
    bucket_b: &[Lead],
    bucket_c: &[Lead],
    bucket_x: &[Lead],
) -> String {
    let mut csv = String::from("bucket,name,amount,deadline,score,effort,trust_tier,reason,url\n");
    
    let write_lead = |csv: &mut String, bucket: &str, lead: &Lead| {
        let effort = lead.effort_score.map(|e| e.to_string()).unwrap_or_default();
        let trust = lead.trust_tier.as_deref().unwrap_or("");
        let reason = lead.match_reasons.first().cloned().unwrap_or_default();
        
        // Escape CSV fields
        let name = escape_csv(&lead.name);
        let amount = escape_csv(&lead.amount);
        let reason = escape_csv(&reason);
        
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            bucket, name, amount, lead.deadline, lead.match_score, effort, trust, reason, lead.url
        ));
    };
    
    for lead in bucket_a {
        write_lead(&mut csv, "A", lead);
    }
    
    for lead in bucket_b {
        write_lead(&mut csv, "B", lead);
    }
    
    for lead in bucket_c {
        write_lead(&mut csv, "C", lead);
    }
    
    for lead in bucket_x {
        write_lead(&mut csv, "X", lead);
    }
    
    csv
}

/// Generate rules audit JSON
pub fn generate_rules_audit(
    rules: &RulesConfig,
    stats: &TriageStats,
) -> RulesAudit {
    let total_rules = rules.hard_reject_rules.len() 
        + rules.soft_downgrade_rules.len() 
        + rules.positive_scoring_rules.len();
    
    let rule_hits: Vec<RuleHitCount> = stats.rule_hits.iter()
        .map(|(id, count)| {
            // Find rule name
            let name = rules.hard_reject_rules.iter()
                .chain(rules.soft_downgrade_rules.iter())
                .chain(rules.positive_scoring_rules.iter())
                .find(|r| &r.id == id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| id.clone());
            
            RuleHitCount {
                rule_id: id.clone(),
                rule_name: name,
                hit_count: *count,
            }
        })
        .collect();
    
    RulesAudit {
        version: "1.0".to_string(),
        timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        rules_file: "Config/rules.yaml".to_string(),
        total_rules,
        items_processed: stats.total,
        buckets: BucketCounts {
            a: stats.bucket_a,
            b: stats.bucket_b,
            c: stats.bucket_c,
            x: stats.bucket_x,
        },
        rule_hits,
    }
}

/// Helper to truncate strings for table display (Unicode-safe)
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max_chars {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Helper to escape CSV fields
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Statistics from triage process
#[derive(Debug, Default)]
pub struct TriageStats {
    pub total: usize,
    pub bucket_a: usize,
    pub bucket_b: usize,
    pub bucket_c: usize,
    pub bucket_x: usize,  // Missed/Closed - saved for next cycle
    pub watchlist: usize,
    pub rule_hits: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_lead() -> Lead {
        Lead {
            name: "Test Scholarship".to_string(),
            amount: "£5000".to_string(),
            deadline: "2026-06-30".to_string(),
            source: "https://example.com".to_string(),
            source_type: "university".to_string(),
            status: "new".to_string(),
            eligibility: vec!["International students".to_string()],
            notes: "Merit-based".to_string(),
            added_date: "2026-01-01".to_string(),
            url: "https://example.com/scholarship".to_string(),
            match_score: 0,
            match_reasons: vec![],
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: None,
            trust_tier: None,
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
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
            is_index_only: false,
            first_seen_at: None,
            last_checked_at: None,
            next_check_at: None,
            persistence_status: None,
            source_seed: None,
            check_count: None,
        extraction_evidence: vec![],
        }
    }
    
    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("this is a very long string", 10), "this is...");
    }
    
    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }
    
    #[test]
    fn test_calculate_confidence_high() {
        let mut lead = create_test_lead();
        lead.deadline_date = Some("2026-06-30".to_string());
        lead.is_taiwan_eligible = Some(true);
        lead.trust_tier = Some("S".to_string());
        lead.http_status = Some(200);
        lead.amount = "£10,000".to_string();
        
        let confidence = calculate_confidence(&lead);
        assert!(confidence >= 0.9, "High quality lead should have confidence >= 0.9, got {}", confidence);
    }
    
    #[test]
    fn test_calculate_confidence_low() {
        let mut lead = create_test_lead();
        lead.deadline_date = None;
        lead.deadline = "Check website".to_string();
        lead.is_taiwan_eligible = None;
        lead.trust_tier = None;
        lead.http_status = None;
        lead.amount = "See website".to_string();
        
        let confidence = calculate_confidence(&lead);
        assert!(confidence < 0.3, "Low quality lead should have confidence < 0.3, got {}", confidence);
    }
    
    #[test]
    fn test_parse_deadline_date() {
        assert_eq!(parse_deadline_date("2026-06-30"), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        assert_eq!(parse_deadline_date("30/06/2026"), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        assert_eq!(parse_deadline_date("Check website"), None);
    }
    
    #[test]
    fn test_is_watchlist_candidate() {
        let mut lead = create_test_lead();
        lead.deadline = "Check website".to_string();
        lead.deadline_date = None;
        assert!(is_watchlist_candidate(&lead));
        
        lead.deadline = "rolling".to_string();
        assert!(is_watchlist_candidate(&lead));
        
        lead.deadline_date = Some("2026-06-30".to_string());
        assert!(!is_watchlist_candidate(&lead));
    }
}

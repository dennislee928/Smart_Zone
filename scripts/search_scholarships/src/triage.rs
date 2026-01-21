//! Triage Module
//! 
//! Implements A/B/C bucket classification and generates triage reports

use crate::types::{Lead, Bucket, RulesConfig, RulesAudit, BucketCounts, RuleHitCount};
use crate::rules::apply_rules;
use chrono::Utc;
use std::collections::HashMap;

/// Perform triage on leads using rules engine
pub fn triage_leads(leads: &mut [Lead], rules: &RulesConfig) -> TriageStats {
    let mut stats = TriageStats::default();
    let mut rule_hits: HashMap<String, usize> = HashMap::new();
    
    for lead in leads.iter_mut() {
        let result = apply_rules(lead, rules);
        
        // Update lead with triage result
        lead.bucket = result.bucket;
        lead.match_score += result.total_score_add;
        
        // Track matched rules
        for rule_match in &result.matched_rules {
            lead.matched_rule_ids.push(rule_match.rule_id.clone());
            *rule_hits.entry(rule_match.rule_id.clone()).or_insert(0) += 1;
        }
        
        // Update stats
        match result.bucket {
            Some(Bucket::A) => stats.bucket_a += 1,
            Some(Bucket::B) => stats.bucket_b += 1,
            Some(Bucket::C) => stats.bucket_c += 1,
            Some(Bucket::X) => stats.bucket_x += 1,
            None => stats.bucket_c += 1, // Default to C
        }
        
        if result.add_to_watchlist {
            stats.watchlist += 1;
        }
    }
    
    stats.rule_hits = rule_hits;
    stats.total = leads.len();
    stats
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
        let mut reason_counts: HashMap<String, usize> = HashMap::new();
        for lead in bucket_c {
            let reason = lead.match_reasons.first()
                .cloned()
                .unwrap_or_else(|| "Unknown reason".to_string());
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
}

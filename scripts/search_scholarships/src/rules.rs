//! Rules Engine for ScholarshipOps Triage
//! 
//! Loads and executes rules from Config/rules.yaml

use crate::types::{Lead, RuleMatch, Bucket, RulesConfig, RuleCondition};
use anyhow::{Result, Context};
use chrono::{NaiveDate, Utc};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Load rules configuration from YAML file
pub fn load_rules(root: &str) -> Result<RulesConfig> {
    let path = Path::new(root).join("Config").join("rules.yaml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read rules from {:?}", path))?;
    
    let config: RulesConfig = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse rules.yaml")?;
    
    Ok(config)
}

/// Apply all rules to a lead and return the result
pub fn apply_rules(lead: &Lead, rules: &RulesConfig) -> RuleApplicationResult {
    let mut result = RuleApplicationResult {
        bucket: None,
        matched_rules: Vec::new(),
        score_adjustments: Vec::new(),
        effort_adjustments: Vec::new(),
        total_score_add: 0,
        total_effort_reduce: 0,
        hard_rejected: false,
        rejection_reason: None,
        add_to_watchlist: false,
    };
    
    // Build searchable text from lead
    let search_text = build_search_text(lead);
    
    // Stage 1: Apply hard reject rules (C or X bucket)
    for rule in &rules.hard_reject_rules {
        if check_rule_condition(&rule.when, lead, &search_text) {
            // Determine target bucket from rule action (default to C)
            let target_bucket = match rule.action.bucket.as_deref() {
                Some("X") => Bucket::X,
                Some("C") | _ => Bucket::C,
            };
            
            result.matched_rules.push(RuleMatch {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                stage: rule.stage.clone(),
                action: format!("Hard reject -> Bucket {}", target_bucket),
                reason: rule.action.reason.clone(),
            });
            result.bucket = Some(target_bucket);
            result.hard_rejected = true;
            result.rejection_reason = Some(rule.action.reason.clone());
            // Hard reject stops further processing
            return result;
        }
    }
    
    // Stage 2: Apply soft downgrade rules (B bucket)
    for rule in &rules.soft_downgrade_rules {
        if check_rule_condition(&rule.when, lead, &search_text) {
            result.matched_rules.push(RuleMatch {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                stage: rule.stage.clone(),
                action: format!("Soft downgrade -> Bucket B"),
                reason: rule.action.reason.clone(),
            });
            
            // Only downgrade if not already set to C
            if result.bucket.is_none() {
                result.bucket = Some(Bucket::B);
            }
            
            if rule.action.add_to_watchlist.unwrap_or(false) {
                result.add_to_watchlist = true;
            }
        }
    }
    
    // Stage 3: Apply positive scoring rules
    for rule in &rules.positive_scoring_rules {
        if check_rule_condition(&rule.when, lead, &search_text) {
            let score_add = rule.action.score_add.unwrap_or(0);
            let effort_reduce = rule.action.effort_reduce.unwrap_or(0);
            
            result.matched_rules.push(RuleMatch {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                stage: rule.stage.clone(),
                action: format!("Score +{}, Effort -{}", score_add, effort_reduce),
                reason: rule.action.reason.clone(),
            });
            
            if score_add > 0 {
                result.score_adjustments.push((rule.id.clone(), score_add));
                result.total_score_add += score_add;
            }
            
            if effort_reduce > 0 {
                result.effort_adjustments.push((rule.id.clone(), effort_reduce));
                result.total_effort_reduce += effort_reduce;
            }
        }
    }
    
    // Determine final bucket based on thresholds if not already set
    if result.bucket.is_none() {
        let final_score = lead.match_score + result.total_score_add;
        
        if let Some(ref thresholds) = rules.bucket_thresholds {
            if let Some(ref a_threshold) = thresholds.a {
                if final_score >= a_threshold.min_final_score {
                    result.bucket = Some(Bucket::A);
                }
            }
            
            if result.bucket.is_none() {
                if let Some(ref b_threshold) = thresholds.b {
                    if final_score >= b_threshold.min_final_score {
                        result.bucket = Some(Bucket::B);
                    }
                }
            }
        }
        
        // Default to B if score is positive but doesn't meet A threshold
        if result.bucket.is_none() && final_score > 0 {
            result.bucket = Some(Bucket::B);
        }
        
        // Default to C if no positive score
        if result.bucket.is_none() {
            result.bucket = Some(Bucket::C);
        }
    }
    
    result
}

/// Build searchable text from lead fields
fn build_search_text(lead: &Lead) -> String {
    format!(
        "{} {} {} {} {} {}",
        lead.name,
        lead.amount,
        lead.notes,
        lead.eligibility.join(" "),
        lead.url,
        lead.source
    ).to_lowercase()
}

/// Check if a rule condition matches
fn check_rule_condition(condition: &RuleCondition, lead: &Lead, search_text: &str) -> bool {
    // Check regex patterns
    if let Some(ref patterns) = condition.any_regex {
        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(search_text) {
                    return true;
                }
            }
        }
    }
    
    // Check deadline conditions
    if let Some(ref deadline_cond) = condition.deadline {
        if deadline_cond.lt_today.unwrap_or(false) {
            if let Some(deadline_date) = parse_deadline(&lead.deadline) {
                let today = Utc::now().date_naive();
                if deadline_date < today {
                    return true;
                }
            }
        }
        
        if deadline_cond.is_null.unwrap_or(false) {
            if lead.deadline.is_empty() || lead.deadline.to_lowercase() == "check website" || lead.deadline.to_lowercase() == "tbd" {
                return true;
            }
        }
    }
    
    // Check HTTP status conditions
    if let Some(ref http_cond) = condition.http_status {
        if let Some(ref any_of) = http_cond.any_of {
            if let Some(status) = lead.http_status {
                if any_of.contains(&status) {
                    return true;
                }
            }
        }
    }
    
    // Check effort score conditions
    if let Some(ref effort_cond) = condition.effort_score {
        if let Some(gt) = effort_cond.gt {
            if lead.effort_score.unwrap_or(0) > gt {
                return true;
            }
        }
    }
    
    // Check country eligibility condition
    if let Some(expected_eligible) = condition.is_taiwan_eligible {
        if let Some(actual_eligible) = lead.is_taiwan_eligible {
            if actual_eligible == expected_eligible {
                return true;
            }
        }
    }
    
    false
}

/// Parse deadline string to NaiveDate
fn parse_deadline(deadline: &str) -> Option<NaiveDate> {
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
            return Some(date);
        }
    }
    
    // Try to extract year-month-day from string
    if let Ok(re) = Regex::new(r"(\d{4})-(\d{2})-(\d{2})") {
        if let Some(caps) = re.captures(deadline) {
            let year: i32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let day: u32 = caps[3].parse().ok()?;
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }
    
    None
}

/// Result of applying rules to a lead
#[derive(Debug, Clone)]
pub struct RuleApplicationResult {
    pub bucket: Option<Bucket>,
    pub matched_rules: Vec<RuleMatch>,
    pub score_adjustments: Vec<(String, i32)>,
    pub effort_adjustments: Vec<(String, i32)>,
    pub total_score_add: i32,
    pub total_effort_reduce: i32,
    pub hard_rejected: bool,
    pub rejection_reason: Option<String>,
    pub add_to_watchlist: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_search_text() {
        let lead = Lead {
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
            bucket: None,
            http_status: None,
            effort_score: None,
            trust_tier: None,
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: None,
            deadline_date: None,
            deadline_label: None,
            intake_year: None,
            study_start: None,
            deadline_confidence: None,
            canonical_url: None,
            is_directory_page: false,
            official_source_url: None,
        };
        
        let text = build_search_text(&lead);
        assert!(text.contains("test scholarship"));
        assert!(text.contains("international students"));
    }
}

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

/// Check if a rule condition matches (AND semantics - all specified conditions must pass)
fn check_rule_condition(condition: &RuleCondition, lead: &Lead, search_text: &str) -> bool {
    let mut has_any_condition = false;
    let mut all_passed = true;
    
    // Check positive regex patterns (any_regex: at least one pattern must match)
    if let Some(ref patterns) = condition.any_regex {
        has_any_condition = true;
        let regex_matched = patterns.iter().any(|pattern| {
            Regex::new(pattern)
                .map(|re| re.is_match(search_text))
                .unwrap_or(false)
        });
        if !regex_matched {
            all_passed = false;
        }
    }
    
    // Check negative regex patterns (not_any_regex: NONE of the patterns should match)
    if let Some(ref patterns) = condition.not_any_regex {
        has_any_condition = true;
        let any_matched = patterns.iter().any(|pattern| {
            Regex::new(pattern)
                .map(|re| re.is_match(search_text))
                .unwrap_or(false)
        });
        // Condition passes only if NONE of the patterns matched
        if any_matched {
            all_passed = false;
        }
    }
    
    // Check deadline conditions
    // Priority: use deadline_date first, fallback to deadline
    if let Some(ref deadline_cond) = condition.deadline {
        has_any_condition = true;
        let mut deadline_condition_passed = false;
        
        // Get deadline string with priority: deadline_date > deadline
        let deadline_str = lead.deadline_date.as_deref().unwrap_or(&lead.deadline);
        
        if deadline_cond.lt_today.unwrap_or(false) {
            if let Some(deadline_date) = parse_deadline(deadline_str) {
                let today = Utc::now().date_naive();
                if deadline_date < today {
                    deadline_condition_passed = true;
                }
            }
        }
        
        if deadline_cond.is_null.unwrap_or(false) {
            let dl_lower = deadline_str.to_lowercase();
            if deadline_str.is_empty() || dl_lower == "check website" || dl_lower == "tbd" || dl_lower == "unknown" {
                deadline_condition_passed = true;
            }
        }
        
        // Check if deadline is after study start (wrong intake cycle)
        if deadline_cond.gt_study_start.unwrap_or(false) {
            if let Some(deadline_date) = parse_deadline(deadline_str) {
                // Use lead's study_start if available, otherwise use target date (2026-09-14)
                let study_start = lead.study_start.as_ref()
                    .and_then(|s| parse_deadline(s))
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 9, 14).unwrap());
                
                // Apply safety margin (default 60 days before study start)
                let margin_days = deadline_cond.safety_margin_days.unwrap_or(60);
                let cutoff_date = study_start - chrono::Duration::days(margin_days);
                
                if deadline_date > cutoff_date {
                    deadline_condition_passed = true;
                }
            }
        }
        
        if !deadline_condition_passed {
            all_passed = false;
        }
    }
    
    // Check HTTP status conditions
    if let Some(ref http_cond) = condition.http_status {
        if let Some(ref any_of) = http_cond.any_of {
            has_any_condition = true;
            if let Some(status) = lead.http_status {
                if !any_of.contains(&status) {
                    all_passed = false;
                }
            } else {
                // No HTTP status available, condition not met
                all_passed = false;
            }
        }
    }
    
    // Check effort score conditions
    if let Some(ref effort_cond) = condition.effort_score {
        if let Some(gt) = effort_cond.gt {
            has_any_condition = true;
            if lead.effort_score.unwrap_or(0) <= gt {
                all_passed = false;
            }
        }
    }
    
    // Check country eligibility condition
    if let Some(expected_eligible) = condition.is_taiwan_eligible {
        has_any_condition = true;
        if let Some(actual_eligible) = lead.is_taiwan_eligible {
            if actual_eligible != expected_eligible {
                all_passed = false;
            }
        } else {
            // No eligibility info available, condition not met for false check
            // For true check, we can't confirm so fail
            all_passed = false;
        }
    }
    
    // Rule triggers only if there's at least one condition AND all conditions pass
    has_any_condition && all_passed
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
    use crate::types::DeadlineCondition;
    
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
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        }
    }
    
    #[test]
    fn test_build_search_text() {
        let lead = create_test_lead();
        let text = build_search_text(&lead);
        assert!(text.contains("test scholarship"));
        assert!(text.contains("international students"));
    }
    
    #[test]
    fn test_rule_condition_and_semantics_regex_only() {
        let lead = create_test_lead();
        let search_text = build_search_text(&lead);
        
        // Test: any_regex matches -> should pass
        let condition = RuleCondition {
            any_regex: Some(vec!["(?i)scholarship".to_string()]),
            ..Default::default()
        };
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Test: any_regex does not match -> should fail
        let condition = RuleCondition {
            any_regex: Some(vec!["(?i)phd only".to_string()]),
            ..Default::default()
        };
        assert!(!check_rule_condition(&condition, &lead, &search_text));
    }
    
    #[test]
    fn test_rule_condition_and_semantics_combined() {
        let mut lead = create_test_lead();
        lead.http_status = Some(403);
        let search_text = build_search_text(&lead);
        
        // Test: regex matches + http_status matches -> should pass (both conditions)
        let condition = RuleCondition {
            any_regex: Some(vec!["(?i)scholarship".to_string()]),
            http_status: Some(crate::types::HttpStatusCondition {
                any_of: Some(vec![403, 429]),
            }),
            ..Default::default()
        };
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Test: regex matches + http_status does NOT match -> should FAIL (AND semantics)
        lead.http_status = Some(200);
        let condition = RuleCondition {
            any_regex: Some(vec!["(?i)scholarship".to_string()]),
            http_status: Some(crate::types::HttpStatusCondition {
                any_of: Some(vec![403, 429]),
            }),
            ..Default::default()
        };
        assert!(!check_rule_condition(&condition, &lead, &search_text));
    }
    
    #[test]
    fn test_rule_condition_not_any_regex() {
        let mut lead = create_test_lead();
        lead.url = "https://random-site.com".to_string();
        let search_text = build_search_text(&lead);
        
        // Test: not_any_regex - none match -> should pass (target NOT in allowlist)
        let condition = RuleCondition {
            not_any_regex: Some(vec!["(?i)gla\\.ac\\.uk".to_string(), "(?i)glasgow".to_string()]),
            ..Default::default()
        };
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Test: not_any_regex - one matches -> should fail
        lead.url = "https://gla.ac.uk/scholarship".to_string();
        let search_text = build_search_text(&lead);
        assert!(!check_rule_condition(&condition, &lead, &search_text));
    }
    
    #[test]
    fn test_deadline_date_priority() {
        let mut lead = create_test_lead();
        lead.deadline = "2020-01-01".to_string(); // Old date in deadline
        lead.deadline_date = Some("2026-12-31".to_string()); // Future date in deadline_date
        let search_text = build_search_text(&lead);
        
        // Test: deadline_date should take priority over deadline
        // lt_today check should use deadline_date (2026-12-31), not deadline (2020-01-01)
        let condition = RuleCondition {
            deadline: Some(DeadlineCondition {
                lt_today: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        // Should NOT trigger because deadline_date (2026-12-31) is in the future
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Now test without deadline_date - should fallback to deadline
        lead.deadline_date = None;
        // Should trigger because deadline (2020-01-01) is in the past
        assert!(check_rule_condition(&condition, &lead, &search_text));
    }
    
    #[test]
    fn test_parse_deadline_formats() {
        // ISO format
        assert_eq!(parse_deadline("2026-06-30"), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        
        // UK format
        assert_eq!(parse_deadline("30/06/2026"), Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
        
        // Invalid
        assert_eq!(parse_deadline("check website"), None);
        assert_eq!(parse_deadline(""), None);
    }
}

//! Rules Engine for ScholarshipOps Triage
//! 
//! Loads and executes rules from Config/rules.yaml

use crate::types::{Lead, RuleMatch, Bucket, RulesConfig, RuleCondition, TrustTier};
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
        match_reasons: Vec::new(),
        hard_fail_reasons: Vec::new(),
        soft_flags: Vec::new(),
        score_adjustments: Vec::new(),
        effort_adjustments: Vec::new(),
        total_score_add: 0,
        total_effort_reduce: 0,
        total_effort_add: 0,
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
            result.hard_fail_reasons.push(rule.action.reason.clone());
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
            result.soft_flags.push(rule.action.reason.clone());

            let effort_add = rule.action.effort_add.unwrap_or(0);
            if effort_add > 0 {
                result.effort_adjustments.push((rule.id.clone(), effort_add));
                result.total_effort_add += effort_add;
            }
            
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
            result.match_reasons.push(rule.action.reason.clone());
            
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
        let lead_effort = lead.effort_score.unwrap_or(0);
        let lead_trust_tier = lead.trust_tier.as_ref()
            .map(|s| TrustTier::from_str(s))
            .unwrap_or(TrustTier::C);
        
        if let Some(ref thresholds) = rules.bucket_thresholds {
            // Check Bucket A requirements: score + trust tier + effort constraints
            if let Some(ref a_threshold) = thresholds.a {
                let score_ok = final_score >= a_threshold.min_final_score;
                
                // Check trust tier (higher is better: S > A > B > C)
                let trust_ok = a_threshold.min_trust_tier.as_ref()
                    .map(|min_tier| {
                        let min = TrustTier::from_str(min_tier);
                        trust_tier_meets_minimum(lead_trust_tier, min)
                    })
                    .unwrap_or(true);
                
                // Check effort score (lower is better)
                let effort_ok = a_threshold.max_effort_score
                    .map(|max| lead_effort <= max)
                    .unwrap_or(true);
                
                if score_ok && trust_ok && effort_ok {
                    result.bucket = Some(Bucket::A);
                }
            }
            
            // Check Bucket B requirements if not already assigned to A
            if result.bucket.is_none() {
                if let Some(ref b_threshold) = thresholds.b {
                    let score_ok = final_score >= b_threshold.min_final_score;
                    
                    let trust_ok = b_threshold.min_trust_tier.as_ref()
                        .map(|min_tier| {
                            let min = TrustTier::from_str(min_tier);
                            trust_tier_meets_minimum(lead_trust_tier, min)
                        })
                        .unwrap_or(true);
                    
                    let effort_ok = b_threshold.max_effort_score
                        .map(|max| lead_effort <= max)
                        .unwrap_or(true);
                    
                    if score_ok && trust_ok && effort_ok {
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

/// Check if actual trust tier meets or exceeds minimum required tier
/// Order: S > A > B > C (S is highest, C is lowest)
fn trust_tier_meets_minimum(actual: TrustTier, minimum: TrustTier) -> bool {
    let tier_rank = |t: TrustTier| -> u8 {
        match t {
            TrustTier::S => 4,
            TrustTier::A => 3,
            TrustTier::B => 2,
            TrustTier::C => 1,
        }
    };
    tier_rank(actual) >= tier_rank(minimum)
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
    
    // Check all_regex patterns (all must match - AND logic)
    if let Some(ref patterns) = condition.all_regex {
        has_any_condition = true;
        let all_matched = patterns.iter().all(|pattern| {
            Regex::new(pattern)
                .map(|re| re.is_match(search_text))
                .unwrap_or(false)
        });
        if !all_matched {
            all_passed = false;
        }
    }
    
    // Check country eligibility condition (tri-state logic)
    // - expected=true  => require explicit Some(true) to pass
    // - expected=false => only trigger rejection if eligible_countries list exists AND is_taiwan_eligible == Some(false) AND confidence == "explicit_list"
    // - expected="unknown" => trigger when is_taiwan_eligible == None
    if let Some(ref expected_eligible) = condition.is_taiwan_eligible {
        has_any_condition = true;
        
        // Handle both bool and string "unknown" values
        if expected_eligible.is_boolean() {
            let expected_bool = expected_eligible.as_bool().unwrap();
            match expected_bool {
                true => {
                    // Must be explicitly confirmed eligible
                    if lead.is_taiwan_eligible != Some(true) {
                        all_passed = false;
                    }
                }
                false => {
                    // Only reject when explicit list exists AND Taiwan is not included AND confidence is explicit_list
                    let has_explicit_list = !lead.eligible_countries.is_empty();
                    let is_explicitly_false = lead.is_taiwan_eligible == Some(false);
                    let confidence_ok = condition.taiwan_eligibility_confidence.as_ref()
                        .map(|c| c == "explicit_list")
                        .unwrap_or(true); // If not specified, don't check confidence
                    
                    if !has_explicit_list || !is_explicitly_false || !confidence_ok {
                        // Condition not met: either no explicit list, or Taiwan is eligible/unknown, or confidence not explicit
                        all_passed = false;
                    }
                }
            }
        } else if expected_eligible.is_string() {
            // Handle "unknown" string value
            if let Some(s) = expected_eligible.as_str() {
                if s == "unknown" {
                    // Trigger when is_taiwan_eligible is None
                    if lead.is_taiwan_eligible.is_some() {
                        all_passed = false;
                    }
                }
            }
        }
    }
    
    // Check taiwan_eligibility_confidence condition
    if let Some(ref expected_confidence) = condition.taiwan_eligibility_confidence {
        has_any_condition = true;
        if lead.taiwan_eligibility_confidence.as_deref() != Some(expected_confidence.as_str()) {
            all_passed = false;
        }
    }
    
    // Check directory page gate
    // Used to skip certain rules (like E-NONTARGET-001) on discovery/index pages
    if let Some(expected) = condition.is_directory_page {
        has_any_condition = true;
        if lead.is_directory_page != expected {
            all_passed = false;
        }
    }
    
    // Rule triggers only if there's at least one condition AND all conditions pass
    has_any_condition && all_passed
}

/// Parse deadline string to NaiveDate with validation
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
            // Validate date: year must be 2020-2100, month 1-12
            if date.year() >= 2020 && date.year() <= 2100 && date.month() >= 1 && date.month() <= 12 {
                return Some(date);
            }
        }
    }
    
    // Try to extract year-month-day from string
    if let Ok(re) = Regex::new(r"(\d{4})-(\d{2})-(\d{2})") {
        if let Some(caps) = re.captures(deadline) {
            let year: i32 = caps[1].parse().ok()?;
            let month: u32 = caps[2].parse().ok()?;
            let day: u32 = caps[3].parse().ok()?;
            
            // Validate before creating date
            if year < 2020 || year > 2100 || month < 1 || month > 12 {
                return None;
            }
            
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                // Double-check validation (from_ymd_opt handles invalid days)
                if date.year() >= 2020 && date.year() <= 2100 && date.month() >= 1 && date.month() <= 12 {
                    return Some(date);
                }
            }
        }
    }
    
    None
}

/// Result of applying rules to a lead
#[derive(Debug, Clone)]
pub struct RuleApplicationResult {
    pub bucket: Option<Bucket>,
    pub matched_rules: Vec<RuleMatch>,
    pub match_reasons: Vec<String>,           // Positive scoring reasons
    pub hard_fail_reasons: Vec<String>,      // Hard reject reasons
    pub soft_flags: Vec<String>,            // Soft downgrade flags
    pub score_adjustments: Vec<(String, i32)>,
    pub effort_adjustments: Vec<(String, i32)>,
    pub total_score_add: i32,
    pub total_effort_reduce: i32,
    pub total_effort_add: i32,
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
    
    // =============================================
    // Tests for tri-state Taiwan eligibility logic
    // =============================================
    
    #[test]
    fn test_taiwan_eligibility_tristate_expected_true() {
        let mut lead = create_test_lead();
        let search_text = build_search_text(&lead);
        
        // Condition: is_taiwan_eligible: true (require explicit confirmation)
        let condition = RuleCondition {
            is_taiwan_eligible: Some(serde_json::Value::Bool(true)),
            ..Default::default()
        };
        
        // Case 1: Lead has explicit Some(true) -> should PASS
        lead.is_taiwan_eligible = Some(true);
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Case 2: Lead has explicit Some(false) -> should FAIL
        lead.is_taiwan_eligible = Some(false);
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Case 3: Lead has None (unknown) -> should FAIL (can't confirm eligibility)
        lead.is_taiwan_eligible = None;
        assert!(!check_rule_condition(&condition, &lead, &search_text));
    }
    
    #[test]
    fn test_taiwan_eligibility_tristate_expected_false() {
        let mut lead = create_test_lead();
        let search_text = build_search_text(&lead);
        
        // Condition: is_taiwan_eligible: false (trigger rejection only when explicit)
        let condition = RuleCondition {
            is_taiwan_eligible: Some(serde_json::Value::Bool(false)),
            ..Default::default()
        };
        
        // Case 1: No eligible countries list + is_taiwan_eligible = Some(false)
        // -> should NOT trigger rejection (no explicit list exists)
        lead.eligible_countries = vec![];
        lead.is_taiwan_eligible = Some(false);
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Case 2: Has eligible countries list + is_taiwan_eligible = Some(false)
        // -> SHOULD trigger rejection (explicit list exists and Taiwan not included)
        lead.eligible_countries = vec!["UK".to_string(), "India".to_string()];
        lead.is_taiwan_eligible = Some(false);
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Case 3: Has eligible countries list + is_taiwan_eligible = Some(true)
        // -> should NOT trigger rejection (Taiwan is eligible)
        lead.eligible_countries = vec!["UK".to_string(), "Taiwan".to_string()];
        lead.is_taiwan_eligible = Some(true);
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Case 4: Has eligible countries list + is_taiwan_eligible = None
        // -> should NOT trigger rejection (unknown eligibility)
        lead.eligible_countries = vec!["UK".to_string()];
        lead.is_taiwan_eligible = None;
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Case 5: No eligible countries list + is_taiwan_eligible = None
        // -> should NOT trigger rejection (no explicit data)
        lead.eligible_countries = vec![];
        lead.is_taiwan_eligible = None;
        assert!(!check_rule_condition(&condition, &lead, &search_text));
    }
    
    // =============================================
    // Tests for is_directory_page condition
    // =============================================
    
    #[test]
    fn test_directory_page_condition() {
        let mut lead = create_test_lead();
        let search_text = build_search_text(&lead);
        
        // Condition: is_directory_page: false (only apply to detail pages)
        let condition = RuleCondition {
            is_directory_page: Some(false),
            ..Default::default()
        };
        
        // Case 1: Lead is NOT a directory page -> condition should PASS
        lead.is_directory_page = false;
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Case 2: Lead IS a directory page -> condition should FAIL (skip this rule)
        lead.is_directory_page = true;
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Test the opposite condition
        let condition_for_directory = RuleCondition {
            is_directory_page: Some(true),
            ..Default::default()
        };
        
        // Case 3: Lead IS a directory page, condition wants directory -> should PASS
        lead.is_directory_page = true;
        assert!(check_rule_condition(&condition_for_directory, &lead, &search_text));
        
        // Case 4: Lead is NOT a directory page, condition wants directory -> should FAIL
        lead.is_directory_page = false;
        assert!(!check_rule_condition(&condition_for_directory, &lead, &search_text));
    }
    
    #[test]
    fn test_directory_page_combined_with_regex() {
        let mut lead = create_test_lead();
        lead.url = "https://random-site.com".to_string();
        let search_text = build_search_text(&lead);
        
        // Simulating E-NONTARGET-001: not_any_regex + is_directory_page: false
        let condition = RuleCondition {
            is_directory_page: Some(false),
            not_any_regex: Some(vec!["(?i)gla\\.ac\\.uk".to_string(), "(?i)glasgow".to_string()]),
            ..Default::default()
        };
        
        // Case 1: Detail page + not matching Glasgow -> rule SHOULD trigger (reject)
        lead.is_directory_page = false;
        assert!(check_rule_condition(&condition, &lead, &search_text));
        
        // Case 2: Directory page + not matching Glasgow -> rule should NOT trigger (skip)
        lead.is_directory_page = true;
        assert!(!check_rule_condition(&condition, &lead, &search_text));
        
        // Case 3: Detail page + matching Glasgow -> rule should NOT trigger (allowlist match)
        lead.is_directory_page = false;
        lead.url = "https://gla.ac.uk/scholarships".to_string();
        let search_text = build_search_text(&lead);
        assert!(!check_rule_condition(&condition, &lead, &search_text));
    }
    
    // =============================================
    // Tests for trust tier comparison
    // =============================================
    
    #[test]
    fn test_trust_tier_meets_minimum() {
        // S is highest, C is lowest: S > A > B > C
        
        // S meets all minimums
        assert!(trust_tier_meets_minimum(TrustTier::S, TrustTier::S));
        assert!(trust_tier_meets_minimum(TrustTier::S, TrustTier::A));
        assert!(trust_tier_meets_minimum(TrustTier::S, TrustTier::B));
        assert!(trust_tier_meets_minimum(TrustTier::S, TrustTier::C));
        
        // A meets A, B, C but not S
        assert!(!trust_tier_meets_minimum(TrustTier::A, TrustTier::S));
        assert!(trust_tier_meets_minimum(TrustTier::A, TrustTier::A));
        assert!(trust_tier_meets_minimum(TrustTier::A, TrustTier::B));
        assert!(trust_tier_meets_minimum(TrustTier::A, TrustTier::C));
        
        // B meets B, C but not S, A
        assert!(!trust_tier_meets_minimum(TrustTier::B, TrustTier::S));
        assert!(!trust_tier_meets_minimum(TrustTier::B, TrustTier::A));
        assert!(trust_tier_meets_minimum(TrustTier::B, TrustTier::B));
        assert!(trust_tier_meets_minimum(TrustTier::B, TrustTier::C));
        
        // C only meets C
        assert!(!trust_tier_meets_minimum(TrustTier::C, TrustTier::S));
        assert!(!trust_tier_meets_minimum(TrustTier::C, TrustTier::A));
        assert!(!trust_tier_meets_minimum(TrustTier::C, TrustTier::B));
        assert!(trust_tier_meets_minimum(TrustTier::C, TrustTier::C));
    }
}

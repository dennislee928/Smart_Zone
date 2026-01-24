//! Effort Scoring Module
//! 
//! Estimates application effort based on requirements

use crate::types::Lead;
use regex::Regex;

/// Calculate effort score for a lead (0-100, higher = more effort)
pub fn calculate_effort_score(lead: &Lead) -> i32 {
    let text = format!(
        "{} {} {}",
        lead.name.to_lowercase(),
        lead.notes.to_lowercase(),
        lead.eligibility.join(" ").to_lowercase()
    );
    
    let mut score = 30; // Base effort
    
    // === Low effort indicators (reduce score) ===
    
    // Auto-considered (no separate application)
    if contains_any(&text, &[
        "auto", "automatic", "no separate application", 
        "considered automatically", "no additional",
    ]) {
        score -= 25;
    }
    
    // Merit-based (usually just grades)
    if contains_any(&text, &["merit", "academic excellence", "gpa based"]) {
        score -= 10;
    }
    
    // === Medium effort indicators ===
    
    // Personal statement / essay
    if contains_any(&text, &[
        "personal statement", "essay", "statement of purpose",
        "motivation letter", "cover letter",
    ]) {
        score += 15;
    }
    
    // Multiple essays
    if let Some(count) = count_essays(&text) {
        if count > 1 {
            score += (count - 1) * 10;
        }
    }
    
    // CV / Resume
    if contains_any(&text, &["cv", "resume", "curriculum vitae"]) {
        score += 5;
    }
    
    // === High effort indicators ===
    
    // References / Recommendations
    if contains_any(&text, &[
        "reference", "recommendation", "referee",
        "letter of recommendation", "academic reference",
    ]) {
        score += 15;
        
        // Multiple references
        if contains_any(&text, &["two reference", "2 reference", "three reference", "3 reference"]) {
            score += 10;
        }
    }
    
    // Interview
    if contains_any(&text, &["interview", "panel", "selection committee"]) {
        score += 20;
    }
    
    // Portfolio / Work samples
    if contains_any(&text, &["portfolio", "work sample", "project sample", "writing sample"]) {
        score += 15;
    }
    
    // Video essay / statement / interview (fine-grained; B-EFFORT-VIDEO-001 also applies via rules)
    if contains_any(&text, &["video essay", "video statement", "video interview"]) {
        score += 15;
    } else if contains_any(&text, &["video", "recording", "presentation"]) {
        score += 15;
    }
    
    // Research proposal
    if contains_any(&text, &["research proposal", "project proposal", "study plan"]) {
        score += 20;
    }
    
    // === Very high effort indicators ===
    
    // Nomination required
    if contains_any(&text, &["nomination", "nominated", "nominate"]) {
        score += 25;
    }
    
    // Exam / Test required
    if contains_any(&text, &["exam", "test", "assessment", "aptitude"]) {
        score += 20;
    }
    
    // Multi-stage process
    if contains_any(&text, &["stage", "round", "shortlist", "final selection"]) {
        score += 15;
    }
    
    // Membership / Affiliation required
    if contains_any(&text, &["member", "membership", "affiliation"]) {
        score += 20;
    }
    
    // External endorsement
    if contains_any(&text, &["endorsement", "sponsor", "backing"]) {
        score += 15;
    }
    
    // Clamp to 0-100
    score.clamp(0, 100)
}

/// Update effort scores for all leads
pub fn update_effort_scores(leads: &mut [Lead]) {
    for lead in leads.iter_mut() {
        if lead.effort_score.is_none() {
            lead.effort_score = Some(calculate_effort_score(lead));
        }
    }
}

/// Check if text contains any of the patterns
fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

/// Try to count number of essays mentioned
fn count_essays(text: &str) -> Option<i32> {
    // Look for patterns like "2 essays", "three essays", etc.
    let patterns = [
        (r"(\d+)\s*essay", 1),
        (r"two\s*essay", 2),
        (r"three\s*essay", 3),
        (r"four\s*essay", 4),
    ];
    
    for (pattern, count) in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if *count > 1 {
                    return Some(*count);
                }
                if let Some(num) = caps.get(1) {
                    if let Ok(n) = num.as_str().parse::<i32>() {
                        return Some(n);
                    }
                }
            }
        }
    }
    
    // Check for single essay mention
    if text.contains("essay") {
        return Some(1);
    }
    
    None
}

/// Get effort level description
#[allow(dead_code)]
pub fn effort_level(score: i32) -> &'static str {
    match score {
        0..=20 => "Very Low (auto-considered)",
        21..=40 => "Low (basic application)",
        41..=60 => "Medium (essays + documents)",
        61..=80 => "High (references + interview)",
        _ => "Very High (multi-stage + nomination)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_test_lead(notes: &str) -> Lead {
        Lead {
            name: "Test".to_string(),
            amount: "£5000".to_string(),
            deadline: "2026-06-30".to_string(),
            source: "test".to_string(),
            source_type: "university".to_string(),
            status: "new".to_string(),
            eligibility: vec![],
            notes: notes.to_string(),
            added_date: String::new(),
            url: "https://test.com".to_string(),
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
        }
    }
    
    #[test]
    fn test_auto_considered() {
        let lead = make_test_lead("Automatically considered upon application");
        let score = calculate_effort_score(&lead);
        assert!(score < 20, "Auto-considered should have low effort");
    }
    
    #[test]
    fn test_interview_required() {
        let lead = make_test_lead("Shortlisted candidates will be invited for interview");
        let score = calculate_effort_score(&lead);
        assert!(score >= 50, "Interview should increase effort");
    }
    
    #[test]
    fn test_nomination_required() {
        let lead = make_test_lead("Requires nomination from your institution");
        let score = calculate_effort_score(&lead);
        assert!(score >= 55, "Nomination should significantly increase effort");
    }
}

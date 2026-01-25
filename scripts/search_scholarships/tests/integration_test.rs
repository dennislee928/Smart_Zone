//! Integration tests for ScholarshipOps pipeline
//! Tests parser extraction and triage bucket assignment using fixture HTML pages

use search_scholarships::filter::{parse_eligible_countries, update_structured_dates};
use search_scholarships::triage::triage_leads;
use search_scholarships::types::{Lead, Bucket};
use search_scholarships::rules::load_rules;
use std::fs;
use std::path::PathBuf;

/// Helper function to create a test lead from HTML content
fn create_lead_from_html(html: &str, url: &str, name: &str) -> Lead {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    
    // Extract basic fields (simplified extraction for testing)
    let mut eligibility = Vec::new();
    let mut notes = String::new();
    let mut amount = String::new();
    let mut deadline = String::new();
    
    // Extract eligibility list
    if let Ok(selector) = Selector::parse(".eligibility li") {
        for element in document.select(&selector) {
            eligibility.push(element.text().collect::<String>().trim().to_string());
        }
    }
    
    // Extract notes from eligibility section
    if let Ok(selector) = Selector::parse(".eligibility p") {
        for element in document.select(&selector) {
            notes.push_str(&element.text().collect::<String>());
            notes.push_str(" ");
        }
    }
    
    // Extract amount and deadline from details section
    if let Ok(selector) = Selector::parse(".details p") {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            if text.contains("Amount:") {
                amount = text.replace("Amount:", "").trim().to_string();
            } else if text.contains("Deadline:") {
                deadline = text.replace("Deadline:", "").trim().to_string();
            }
        }
    }
    
    // Combine all text for eligibility parsing
    let full_text = format!("{} {} {}", html, notes, eligibility.join(" "));
    
    // Parse eligible countries
    let (eligible_countries, is_taiwan_eligible) = parse_eligible_countries(&full_text);
    
    let mut lead = Lead {
        name: name.to_string(),
        amount: if amount.is_empty() { "See website".to_string() } else { amount },
        deadline: if deadline.is_empty() { "Check website".to_string() } else { deadline },
        source: url.to_string(),
        source_type: "university".to_string(),
        status: "new".to_string(),
        eligibility,
        notes: notes.trim().to_string(),
        added_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        url: url.to_string(),
        match_score: 0,
        match_reasons: vec![],
        hard_fail_reasons: vec![],
        soft_flags: vec![],
        bucket: None,
        http_status: Some(200),
        effort_score: None,
        trust_tier: Some("S".to_string()),
        risk_flags: vec![],
        matched_rule_ids: vec![],
        eligible_countries,
        is_taiwan_eligible,
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
    };
    
    // Update structured dates
    update_structured_dates(&mut lead);
    
    lead
}

/// Load rules configuration for testing
fn load_test_rules() -> search_scholarships::types::RulesConfig {
    let root = std::env::var("ROOT").unwrap_or_else(|_| "..".to_string());
    load_rules(&root).expect("Failed to load rules")
}

#[test]
#[ignore] // Mark as integration test
fn test_taiwan_excluded_hard_reject() {
    let html = fs::read_to_string("tests/fixtures/taiwan_excluded.html")
        .expect("Failed to read taiwan_excluded.html");
    
    let mut lead = create_lead_from_html(
        &html,
        "https://example.com/commonwealth-scholarship",
        "Commonwealth Scholarship"
    );
    
    let rules = load_test_rules();
    let mut leads = vec![lead];
    let stats = triage_leads(&mut leads, &rules);
    
    let lead = &leads[0];
    
    // Should be hard rejected (bucket C) because Taiwan is explicitly excluded
    assert_eq!(lead.bucket, Some(Bucket::C), "Taiwan-excluded scholarship should be bucket C");
    assert!(
        lead.hard_fail_reasons.iter().any(|r| r.contains("Taiwan") || r.contains("not eligible")),
        "Should have hard fail reason mentioning Taiwan exclusion"
    );
    assert_eq!(stats.bucket_c, 1, "Should have 1 C bucket lead");
}

#[test]
#[ignore]
fn test_home_fee_only_hard_reject() {
    let html = fs::read_to_string("tests/fixtures/home_fee_only.html")
        .expect("Failed to read home_fee_only.html");
    
    let mut lead = create_lead_from_html(
        &html,
        "https://example.ac.uk/uk-domestic-scholarship",
        "UK Domestic Excellence Scholarship"
    );
    
    let rules = load_test_rules();
    let mut leads = vec![lead];
    let stats = triage_leads(&mut leads, &rules);
    
    let lead = &leads[0];
    
    // Should be hard rejected (bucket C) because it requires Home fee status
    assert_eq!(lead.bucket, Some(Bucket::C), "Home-fee-only scholarship should be bucket C");
    assert!(
        lead.hard_fail_reasons.iter().any(|r| 
            r.contains("Home") || r.contains("UK fee status") || r.contains("E-FEE-001")
        ),
        "Should have hard fail reason mentioning Home/UK fee status"
    );
    assert_eq!(stats.bucket_c, 1, "Should have 1 C bucket lead");
}

#[test]
#[ignore]
fn test_phd_only_hard_reject() {
    let html = fs::read_to_string("tests/fixtures/phd_only.html")
        .expect("Failed to read phd_only.html");
    
    let mut lead = create_lead_from_html(
        &html,
        "https://example.com/phd-fellowship",
        "Doctoral Research Fellowship"
    );
    
    let rules = load_test_rules();
    let mut leads = vec![lead];
    let stats = triage_leads(&mut leads, &rules);
    
    let lead = &leads[0];
    
    // Should be hard rejected (bucket C) because it's PhD-only
    assert_eq!(lead.bucket, Some(Bucket::C), "PhD-only scholarship should be bucket C");
    assert!(
        lead.hard_fail_reasons.iter().any(|r| 
            r.contains("PhD") || r.contains("Doctoral") || r.contains("E-LEVEL-PHD-001")
        ),
        "Should have hard fail reason mentioning PhD/Doctoral"
    );
    assert_eq!(stats.bucket_c, 1, "Should have 1 C bucket lead");
}

#[test]
#[ignore]
fn test_disability_only_hard_reject() {
    let html = fs::read_to_string("tests/fixtures/disability_only.html")
        .expect("Failed to read disability_only.html");
    
    let mut lead = create_lead_from_html(
        &html,
        "https://example.com/disability-scholarship",
        "Disability Support Scholarship"
    );
    
    let rules = load_test_rules();
    let mut leads = vec![lead];
    let stats = triage_leads(&mut leads, &rules);
    
    let lead = &leads[0];
    
    // Should be hard rejected (bucket C) because it requires disability certificate
    // (assuming profile doesn't have disability certificate configured)
    assert_eq!(lead.bucket, Some(Bucket::C), "Disability-only scholarship should be bucket C");
    assert!(
        lead.hard_fail_reasons.iter().any(|r| 
            r.contains("disability") || r.contains("certificate") || r.contains("E-DISABILITY")
        ),
        "Should have hard fail reason mentioning disability requirement"
    );
    assert_eq!(stats.bucket_c, 1, "Should have 1 C bucket lead");
}

#[test]
#[ignore]
fn test_international_eligible_should_pass() {
    let html = fs::read_to_string("tests/fixtures/international_eligible.html")
        .expect("Failed to read international_eligible.html");
    
    let mut lead = create_lead_from_html(
        &html,
        "https://www.gla.ac.uk/scholarships/globalleadershipscholarship/",
        "Glasgow Global Leadership Scholarship"
    );
    
    let rules = load_test_rules();
    let mut leads = vec![lead];
    let stats = triage_leads(&mut leads, &rules);
    
    let lead = &leads[0];
    
    // Should NOT be hard rejected - should be A or B bucket
    assert_ne!(
        lead.bucket, 
        Some(Bucket::C), 
        "International-eligible Glasgow scholarship should NOT be bucket C"
    );
    assert!(
        lead.bucket == Some(Bucket::A) || lead.bucket == Some(Bucket::B),
        "Should be bucket A or B"
    );
    assert!(
        lead.is_taiwan_eligible == Some(true) || lead.is_taiwan_eligible.is_none(),
        "Taiwan eligibility should be true or unknown (not false)"
    );
    assert_eq!(stats.bucket_c, 0, "Should have 0 C bucket leads");
}

#[test]
#[ignore]
fn test_e2e_smoke_multiple_leads() {
    // Test multiple leads in one run to ensure no crashes
    let mut leads = Vec::new();
    
    // Load all fixtures
    let fixtures = vec![
        ("tests/fixtures/taiwan_excluded.html", "Commonwealth Scholarship"),
        ("tests/fixtures/home_fee_only.html", "UK Domestic Scholarship"),
        ("tests/fixtures/phd_only.html", "PhD Fellowship"),
        ("tests/fixtures/international_eligible.html", "Glasgow Scholarship"),
    ];
    
    for (path, name) in fixtures {
        if let Ok(html) = fs::read_to_string(path) {
            let lead = create_lead_from_html(&html, &format!("https://example.com/{}", name), name);
            leads.push(lead);
        }
    }
    
    let rules = load_test_rules();
    let stats = triage_leads(&mut leads, &rules);
    
    // Assert no crashes and reasonable bucket distribution
    assert_eq!(stats.total, leads.len(), "Should process all leads");
    assert!(stats.bucket_c >= 2, "Should have at least 2 C bucket leads (Taiwan excluded + Home fee)");
    assert!(stats.bucket_a + stats.bucket_b >= 1, "Should have at least 1 A or B bucket lead (Glasgow)");
}

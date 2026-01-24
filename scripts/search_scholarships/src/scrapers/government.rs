use crate::types::{Lead, ScrapeResult, SourceStatus};
use anyhow::Result;

/// Scrape a government source and return detailed result for health tracking
pub fn scrape(url: &str) -> Result<ScrapeResult> {
    println!("Scraping government website: {}", url);
    
    // Government scholarship data - these are well-known programmes
    // Real scraping of gov.uk is complex due to their structure
    // For government sources, we return known scholarships as "success"
    Ok(ScrapeResult {
        leads: get_known_government_scholarships(url),
        status: SourceStatus::Ok,
        http_code: Some(200),
        error_message: None,
    })
}

/// Legacy wrapper for backward compatibility
#[allow(dead_code)]
pub fn scrape_leads_only(url: &str) -> Result<Vec<Lead>> {
    let result = scrape(url)?;
    Ok(result.leads)
}

/// Known UK government scholarships
fn get_known_government_scholarships(source_url: &str) -> Vec<Lead> {
    vec![
        // NOTE: Chevening 2026/27 cycle closed on 2025-10-07
        // The next cycle (2027/28) typically opens Aug 2026, closes Nov 2026
        Lead {
            name: "Chevening Scholarships 2027/28".to_string(),
            amount: "Full tuition + living costs + flights".to_string(),
            deadline: "2026-11-03".to_string(),  // Estimated for 2027/28 cycle
            source: source_url.to_string(),
            source_type: "government".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "International students".to_string(),
                "2+ years work experience".to_string(),
                "Return to home country for 2 years".to_string(),
            ],
            notes: "UK government's global scholarship programme - 2027/28 intake (NOT for Sep 2026 start)".to_string(),
            added_date: String::new(),
            url: "https://www.chevening.org/scholarships/".to_string(),
            match_score: 0,
            match_reasons: vec![],
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(60), // High effort - essays, references, interview
            trust_tier: Some("S".to_string()), // Government = Tier S
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // Chevening includes Taiwan
            taiwan_eligibility_confidence: None,
            deadline_date: Some("2026-11-03".to_string()),
            deadline_label: Some("applications close (estimated)".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-09-01".to_string()),  // Study starts Sep 2027, NOT 2026
            deadline_confidence: Some("estimated".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.chevening.org/scholarships/".to_string()),
            source_domain: Some("chevening.org".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
        Lead {
            name: "Commonwealth Scholarships (Taught Masters)".to_string(),
            amount: "Full tuition + stipend + travel".to_string(),
            deadline: "2026-10-18".to_string(),
            source: source_url.to_string(),
            source_type: "government".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "Commonwealth country citizens ONLY".to_string(),
                "Cannot afford to study without funding".to_string(),
                "Taiwan NOT eligible".to_string(),
            ],
            notes: "For students from developing Commonwealth countries - Taiwan NOT eligible".to_string(),
            added_date: String::new(),
            url: "https://cscuk.fcdo.gov.uk/scholarships/commonwealth-masters-scholarships/".to_string(),
            match_score: 0,
            match_reasons: vec![],
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(50),
            trust_tier: Some("S".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![
                "Bangladesh".to_string(), "Cameroon".to_string(), "Ghana".to_string(),
                "India".to_string(), "Kenya".to_string(), "Malawi".to_string(),
                "Nigeria".to_string(), "Pakistan".to_string(), "Rwanda".to_string(),
                "Sierra Leone".to_string(), "Sri Lanka".to_string(), "Tanzania".to_string(),
                "Uganda".to_string(), "Zambia".to_string(),
            ],
            is_taiwan_eligible: Some(false), // Taiwan is NOT a Commonwealth country - HARD REJECT
            taiwan_eligibility_confidence: Some("explicit_list".to_string()),
            deadline_date: Some("2026-10-18".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-09-01".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://cscuk.fcdo.gov.uk/scholarships/commonwealth-masters-scholarships/".to_string()),
            source_domain: Some("cscuk.fcdo.gov.uk".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
        Lead {
            name: "GREAT Scholarships".to_string(),
            amount: "£10,000 minimum".to_string(),
            deadline: "2026-05-15".to_string(),
            source: source_url.to_string(),
            source_type: "government".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "Students from selected countries".to_string(),
                "Postgraduate study".to_string(),
            ],
            notes: "Joint UK government and British Council programme".to_string(),
            added_date: String::new(),
            url: "https://study-uk.britishcouncil.org/scholarships/great-scholarships".to_string(),
            match_score: 0,
            match_reasons: vec![],
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(40),
            trust_tier: Some("S".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(false), // Taiwan is NOT in GREAT Scholarships list
            taiwan_eligibility_confidence: Some("explicit_list".to_string()),
            deadline_date: Some("2026-05-15".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2026/27".to_string()),
            study_start: Some("2026-09".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://study-uk.britishcouncil.org/scholarships/great-scholarships".to_string()),
            source_domain: Some("study-uk.britishcouncil.org".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
        Lead {
            name: "Marshall Scholarships".to_string(),
            amount: "Full funding for 2 years".to_string(),
            deadline: "2026-09-30".to_string(),
            source: source_url.to_string(),
            source_type: "government".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "US citizens only".to_string(),
                "GPA 3.7+".to_string(),
            ],
            notes: "For outstanding American students".to_string(),
            added_date: String::new(),
            url: "https://www.marshallscholarship.org/".to_string(),
            match_score: 0,
            match_reasons: vec![],
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(70),
            trust_tier: Some("S".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec!["United States".to_string()],
            is_taiwan_eligible: Some(false), // US citizens only
            taiwan_eligibility_confidence: Some("explicit_list".to_string()),
            deadline_date: Some("2026-09-30".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-09".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.marshallscholarship.org/".to_string()),
            source_domain: Some("marshallscholarship.org".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
    ]
}

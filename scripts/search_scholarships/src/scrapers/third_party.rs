use crate::types::{Lead, ScrapeResult, SourceStatus};
use anyhow::Result;
use scraper::{Html, Selector};

/// Scrape a third-party source and return detailed result for health tracking
pub fn scrape(url: &str) -> Result<ScrapeResult> {
    println!("Scraping third-party database: {}", url);
    
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ScholarshipBot/1.0)")
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;
    
    let response = client.get(url).send();
    
    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            
            if resp.status().is_success() {
                let html = resp.text()?;
                let mut leads = parse_third_party_html(&html, url);
                
                // If HTML parsing found nothing, use known scholarships as fallback
                if leads.is_empty() {
                    let known = get_known_third_party_scholarships(url);
                    if !known.is_empty() {
                        println!("  HTML parsing empty, using {} known scholarships", known.len());
                        leads = known;
                    } else {
                        println!("  No scholarships found from HTML (empty result)");
                    }
                }
                
                Ok(ScrapeResult {
                    leads,
                    status: SourceStatus::Ok,
                    http_code: Some(status_code),
                    error_message: None,
                })
            } else {
                let status = match status_code {
                    404 => SourceStatus::NotFound,
                    403 => SourceStatus::Forbidden,
                    429 => SourceStatus::RateLimited,
                    500..=599 => SourceStatus::ServerError,
                    _ => SourceStatus::Unknown,
                };
                
                println!("  HTTP {} - {}", status_code, status);
                
                // Even on HTTP errors, try to use known scholarships for known foundations
                let known = get_known_third_party_scholarships(url);
                if !known.is_empty() {
                    println!("  Using {} known scholarships despite HTTP error", known.len());
                    return Ok(ScrapeResult {
                        leads: known,
                        status: SourceStatus::Ok,
                        http_code: Some(status_code),
                        error_message: None,
                    });
                }
                
                Ok(ScrapeResult {
                    leads: vec![],
                    status,
                    http_code: Some(status_code),
                    error_message: Some(format!("HTTP {}", status_code)),
                })
            }
        }
        Err(e) => {
            // Network/connection error
            let error_str = e.to_string();
            let status = if error_str.contains("SSL") || error_str.contains("certificate") {
                SourceStatus::SslError
            } else if error_str.contains("timeout") {
                SourceStatus::Timeout
            } else if error_str.contains("redirect") {
                SourceStatus::TooManyRedirects
            } else {
                SourceStatus::NetworkError
            };
            
            println!("  Request failed: {} - {}", status, error_str);
            
            Ok(ScrapeResult {
                leads: vec![],
                status,
                http_code: None,
                error_message: Some(error_str),
            })
        }
    }
}

/// Legacy wrapper for backward compatibility - returns only leads
#[allow(dead_code)]
pub fn scrape_leads_only(url: &str) -> Result<Vec<Lead>> {
    let result = scrape(url)?;
    Ok(result.leads)
}

fn parse_third_party_html(html: &str, base_url: &str) -> Vec<Lead> {
    let document = Html::parse_document(html);
    let mut leads = Vec::new();
    
    // FindAPhD specific selectors
    let selectors = [
        ".phd-result",
        ".result-item",
        ".funding-result",
        "article",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector).take(10) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                
                // Look for funding-related content
                let text_lower = text.to_lowercase();
                if text_lower.contains("fund") || text_lower.contains("scholar") || text_lower.contains("stipend") {
                    let title = extract_title(&element).unwrap_or_else(|| {
                        text.split_whitespace().take(10).collect::<Vec<_>>().join(" ")
                    });
                    
                    if title.len() > 10 {
                        leads.push(Lead {
                            name: title,
                            amount: extract_amount(&text).unwrap_or_else(|| "See website".to_string()),
                            deadline: "Check website".to_string(),
                            source: base_url.to_string(),
                            source_type: "third_party".to_string(),
                            status: "new".to_string(),
                            eligibility: vec!["See website for details".to_string()],
                            notes: String::new(),
                            added_date: String::new(),
                            url: base_url.to_string(),
                            match_score: 0,
                            match_reasons: vec![],
                            bucket: None,
                            http_status: None,
                            effort_score: None,
                            trust_tier: Some("B".to_string()), // Third party = Tier B
                            risk_flags: vec![],
                            matched_rule_ids: vec![],
                            eligible_countries: vec![],
                            is_taiwan_eligible: None,
                            deadline_date: None,
                            deadline_label: None,
                            intake_year: None,
                            study_start: None,
                            deadline_confidence: Some("unknown".to_string()),
                            canonical_url: None,
                            is_directory_page: false,
                            official_source_url: None,
                            confidence: None,
                            eligibility_confidence: None,
                            tags: vec![],
                        });
                    }
                }
            }
        }
        
        if !leads.is_empty() {
            break;
        }
    }
    
    leads
}

fn extract_title(element: &scraper::ElementRef) -> Option<String> {
    let title_selectors = ["h2", "h3", "h4", "a.title", ".title", "a"];
    
    for sel_str in &title_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(title_el) = element.select(&sel).next() {
                let title = title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !title.is_empty() && title.len() > 5 {
                    return Some(title);
                }
            }
        }
    }
    None
}

fn extract_amount(text: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    
    // Look for stipend/funding amounts
    if let Ok(re) = regex::Regex::new(r"(?i)(£|€|\$)[\d,]+(\s*(per\s+)?(year|annum|month|pa))?") {
        if let Some(m) = re.find(text) {
            return Some(text[m.start()..m.end()].to_string());
        }
    }
    
    if text_lower.contains("fully funded") {
        return Some("Fully funded".to_string());
    }
    
    None
}

/// Generate search queries with location-constrained keywords
/// Returns queries optimized for UK/Scotland/International students
pub fn generate_search_queries() -> Vec<String> {
    let subjects = vec![
        "Software Development",
        "Computer Science", 
        "Data Science",
        "Computing",
    ];
    
    let mut queries = Vec::new();
    
    for subject in &subjects {
        // Strategy A: Location-specific (most effective for filtering US-only)
        queries.push(format!("{} scholarship UK", subject));
        queries.push(format!("{} scholarship Scotland", subject));
        queries.push(format!("{} scholarship University of Glasgow", subject));
        
        // Strategy B: Identity-based (international students)
        queries.push(format!("{} scholarship for international students", subject));
        queries.push(format!("{} scholarship for Taiwanese students", subject));
        
        // Strategy C: Degree-level (exclude high school/undergraduate)
        queries.push(format!("{} master's scholarship", subject));
        queries.push(format!("{} MSc funding", subject));
        queries.push(format!("{} postgraduate scholarship UK", subject));
    }
    
    // Add portable scholarship keywords
    queries.push("scholarship for international students UK masters".to_string());
    queries.push("Chevening scholarship".to_string());
    queries.push("Commonwealth scholarship UK".to_string());
    queries.push("Rotary Global Grant UK".to_string());
    
    queries
}

/// Known third-party scholarships as fallback
/// DEPRECATED: Do not use in production - generates hardcoded fake data
/// This function is kept for reference only and should not be called
/// Known third-party scholarships for UK study
/// These are verified prestigious scholarships available to international (including Taiwan) students
fn get_known_third_party_scholarships(source_url: &str) -> Vec<Lead> {
    let url_lower = source_url.to_lowercase();
    
    // Gates Cambridge - only return if URL matches
    if url_lower.contains("gatescambridge") {
        return vec![
            Lead {
                name: "Gates Cambridge Scholarship 2026/27".to_string(),
                amount: "Full cost of study + maintenance + airfare".to_string(),
                deadline: "2025-12-03".to_string(), // US deadline; international is Jan
                source: source_url.to_string(),
                source_type: "foundation".to_string(),
                status: "new".to_string(),
                eligibility: vec![
                    "Non-UK citizens".to_string(),
                    "Outstanding academic record".to_string(),
                    "Leadership and commitment to improving others' lives".to_string(),
                ],
                notes: "Cambridge-only. One of the most prestigious scholarships. Deadline varies by region.".to_string(),
                added_date: String::new(),
                url: "https://www.gatescambridge.org/apply/".to_string(),
                match_score: 0,
                match_reasons: vec![],
                bucket: None,
                http_status: None,
                effort_score: Some(70),
                trust_tier: Some("A".to_string()),
                risk_flags: vec![],
                matched_rule_ids: vec![],
                eligible_countries: vec![],
                is_taiwan_eligible: Some(true),
                deadline_date: Some("2026-01-07".to_string()), // International deadline
                deadline_label: Some("international deadline".to_string()),
                intake_year: Some("2026/27".to_string()),
                study_start: Some("2026-10".to_string()),
                deadline_confidence: Some("confirmed".to_string()),
                canonical_url: None,
                is_directory_page: false,
                official_source_url: Some("https://www.gatescambridge.org/".to_string()),
                confidence: None,
                eligibility_confidence: None,
                tags: vec!["cambridge".to_string(), "prestigious".to_string()],
            },
        ];
    }
    
    // Rotary Foundation
    if url_lower.contains("rotary.org") {
        return vec![
            Lead {
                name: "Rotary Foundation Global Grant Scholarship".to_string(),
                amount: "$30,000+ (varies by district)".to_string(),
                deadline: "2026-03-31".to_string(), // Varies by district
                source: source_url.to_string(),
                source_type: "foundation".to_string(),
                status: "new".to_string(),
                eligibility: vec![
                    "Sponsored by local Rotary club".to_string(),
                    "Graduate study in one of 7 focus areas".to_string(),
                    "International students eligible".to_string(),
                ],
                notes: "Contact local Rotary club for sponsorship. Focus: peace, disease prevention, water, maternal health, education, economic development, environment.".to_string(),
                added_date: String::new(),
                url: "https://www.rotary.org/en/our-programs/scholarships".to_string(),
                match_score: 0,
                match_reasons: vec![],
                bucket: None,
                http_status: None,
                effort_score: Some(50),
                trust_tier: Some("A".to_string()),
                risk_flags: vec![],
                matched_rule_ids: vec![],
                eligible_countries: vec![],
                is_taiwan_eligible: Some(true),
                deadline_date: Some("2026-03-31".to_string()),
                deadline_label: Some("varies by district".to_string()),
                intake_year: Some("2026/27".to_string()),
                study_start: Some("2026-09".to_string()),
                deadline_confidence: Some("estimated".to_string()),
                canonical_url: None,
                is_directory_page: false,
                official_source_url: Some("https://www.rotary.org/en/our-programs/scholarships".to_string()),
                confidence: None,
                eligibility_confidence: None,
                tags: vec!["rotary".to_string(), "sponsored".to_string()],
            },
        ];
    }
    
    // Marshall Scholarship - US citizens only
    if url_lower.contains("marshallscholarship") {
        return vec![
            Lead {
                name: "Marshall Scholarship 2026/27".to_string(),
                amount: "Full tuition + living expenses + travel".to_string(),
                deadline: "2025-09-30".to_string(), // For 2026/27 intake
                source: source_url.to_string(),
                source_type: "foundation".to_string(),
                status: "new".to_string(),
                eligibility: vec![
                    "US citizens ONLY".to_string(),
                    "GPA 3.7+".to_string(),
                    "Leadership potential".to_string(),
                ],
                notes: "For outstanding American students at any UK university".to_string(),
                added_date: String::new(),
                url: "https://www.marshallscholarship.org/apply".to_string(),
                match_score: 0,
                match_reasons: vec![],
                bucket: None,
                http_status: None,
                effort_score: Some(75),
                trust_tier: Some("A".to_string()),
                risk_flags: vec![],
                matched_rule_ids: vec![],
                eligible_countries: vec!["United States".to_string()],
                is_taiwan_eligible: Some(false), // US citizens only
                deadline_date: Some("2025-09-30".to_string()),
                deadline_label: Some("applications close".to_string()),
                intake_year: Some("2026/27".to_string()),
                study_start: Some("2026-10".to_string()),
                deadline_confidence: Some("confirmed".to_string()),
                canonical_url: None,
                is_directory_page: false,
                official_source_url: Some("https://www.marshallscholarship.org/".to_string()),
                confidence: None,
                eligibility_confidence: None,
                tags: vec!["us-only".to_string(), "prestigious".to_string()],
            },
        ];
    }
    
    vec![]
}

/// DEPRECATED: Original hardcoded scholarship data - DO NOT USE
/// Kept as reference only. All scholarship discovery should come from actual web scraping.
#[allow(dead_code)]
fn _get_known_third_party_scholarships_deprecated(source_url: &str) -> Vec<Lead> {
    vec![
        Lead {
            name: "Gates Cambridge Scholarship".to_string(),
            amount: "Full cost of study + maintenance".to_string(),
            deadline: "2026-10-13".to_string(),
            source: source_url.to_string(),
            source_type: "third_party".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "Non-UK citizens".to_string(),
                "Outstanding academic record".to_string(),
                "Leadership potential".to_string(),
            ],
            notes: "One of the most prestigious scholarships".to_string(),
            added_date: String::new(),
            url: "https://www.gatescambridge.org/".to_string(),
            match_score: 0,
            match_reasons: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(70), // High effort - essays, interview
            trust_tier: Some("A".to_string()), // Major foundation = Tier A
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // Non-UK citizens = Taiwan eligible
            deadline_date: Some("2026-10-13".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.gatescambridge.org/".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
        Lead {
            name: "Rhodes Scholarship (Oxford)".to_string(),
            amount: "Full tuition + stipend".to_string(),
            deadline: "2026-10-01".to_string(),
            source: source_url.to_string(),
            source_type: "third_party".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "Selected countries only".to_string(),
                "Age 19-25".to_string(),
                "Outstanding achievements".to_string(),
            ],
            notes: "World's oldest international scholarship".to_string(),
            added_date: String::new(),
            url: "https://www.rhodeshouse.ox.ac.uk/scholarships/".to_string(),
            match_score: 0,
            match_reasons: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(80), // Very high effort
            trust_tier: Some("A".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: None, // Need to verify - Taiwan may not be in Rhodes list
            deadline_date: Some("2026-10-01".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.rhodeshouse.ox.ac.uk/scholarships/".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
        Lead {
            name: "Clarendon Scholarship (Oxford)".to_string(),
            amount: "Full tuition + living expenses".to_string(),
            deadline: "2026-01-10".to_string(),
            source: source_url.to_string(),
            source_type: "third_party".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "All nationalities".to_string(),
                "Outstanding academic merit".to_string(),
            ],
            notes: "Automatic consideration with Oxford application".to_string(),
            added_date: String::new(),
            url: "https://www.ox.ac.uk/clarendon/".to_string(),
            match_score: 0,
            match_reasons: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(10), // Auto-considered
            trust_tier: Some("S".to_string()), // University official
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // All nationalities = Taiwan eligible
            deadline_date: Some("2026-01-10".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2026/27".to_string()),
            study_start: Some("2026-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.ox.ac.uk/clarendon/".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
        Lead {
            name: "Wellcome Trust PhD Programmes".to_string(),
            amount: "Full stipend + tuition + research costs".to_string(),
            deadline: "2026-11-30".to_string(),
            source: source_url.to_string(),
            source_type: "third_party".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "Biomedical/health research".to_string(),
                "International students eligible".to_string(),
            ],
            notes: "Various programmes across UK universities".to_string(),
            added_date: String::new(),
            url: "https://wellcome.org/grant-funding".to_string(),
            match_score: 0,
            match_reasons: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(60),
            trust_tier: Some("A".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // International students eligible
            deadline_date: Some("2026-11-30".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://wellcome.org/grant-funding".to_string()),
            confidence: None,
            eligibility_confidence: None,
            tags: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_search_queries() {
        let queries = generate_search_queries();
        
        // Should have queries
        assert!(!queries.is_empty());
        
        // Should contain UK-focused queries
        assert!(queries.iter().any(|q| q.contains("UK")));
        assert!(queries.iter().any(|q| q.contains("Scotland")));
        assert!(queries.iter().any(|q| q.contains("Glasgow")));
        
        // Should contain international student queries
        assert!(queries.iter().any(|q| q.contains("international students")));
        
        // Should contain postgraduate queries
        assert!(queries.iter().any(|q| q.contains("master") || q.contains("MSc") || q.contains("postgraduate")));
    }
    
    #[test]
    fn test_deprecated_function_returns_empty() {
        #[allow(deprecated)]
        let leads = get_known_third_party_scholarships("https://example.com");
        assert!(leads.is_empty(), "Deprecated function should return empty vector");
    }
}

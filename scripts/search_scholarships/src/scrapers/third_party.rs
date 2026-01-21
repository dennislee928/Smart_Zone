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
                let leads = parse_third_party_html(&html, url);
                
                if leads.is_empty() {
                    println!("  No scholarships found from HTML (empty result)");
                }
                
                Ok(ScrapeResult {
                    leads,
                    status: SourceStatus::Ok,
                    http_code: Some(status_code),
                    error_message: None,
                })
            } else {
                // HTTP error - return empty leads, don't fallback to fake data
                let status = match status_code {
                    404 => SourceStatus::NotFound,
                    403 => SourceStatus::Forbidden,
                    429 => SourceStatus::RateLimited,
                    500..=599 => SourceStatus::ServerError,
                    _ => SourceStatus::Unknown,
                };
                
                println!("  HTTP {} - {}", status_code, status);
                
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

/// Known third-party scholarships as fallback
fn get_known_third_party_scholarships(source_url: &str) -> Vec<Lead> {
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
        },
    ]
}

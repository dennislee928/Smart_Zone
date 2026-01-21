use crate::types::{Lead, ScrapeResult, SourceStatus};
use anyhow::Result;
use scraper::{Html, Selector};

/// Scrape a university source and return detailed result for health tracking
pub fn scrape(url: &str) -> Result<ScrapeResult> {
    println!("Scraping university website: {}", url);
    
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
                let leads = parse_university_html(&html, url);
                
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

/// Legacy wrapper for backward compatibility
pub fn scrape_leads_only(url: &str) -> Result<Vec<Lead>> {
    let result = scrape(url)?;
    Ok(result.leads)
}

fn parse_university_html(html: &str, base_url: &str) -> Vec<Lead> {
    let document = Html::parse_document(html);
    let mut leads = Vec::new();
    
    // Try common scholarship listing selectors
    let selectors = [
        "article.scholarship",
        ".scholarship-item",
        ".funding-item",
        "div[class*='scholarship']",
        "li[class*='scholarship']",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if text.len() > 20 {
                    // Extract title from h2, h3, or first strong/a element
                    let title = extract_title(&element).unwrap_or_else(|| {
                        text.chars().take(100).collect::<String>()
                    });
                    
                    leads.push(Lead {
                        name: title,
                        amount: extract_amount(&text).unwrap_or_else(|| "See website".to_string()),
                        deadline: extract_deadline(&text).unwrap_or_else(|| "Check website".to_string()),
                        source: base_url.to_string(),
                        source_type: "university".to_string(),
                        status: "new".to_string(),
                        eligibility: vec!["International students".to_string()],
                        notes: String::new(),
                        added_date: String::new(),
                        url: base_url.to_string(),
                        match_score: 0,
                        match_reasons: vec![],
                        bucket: None,
                        http_status: None,
                        effort_score: None,
                        trust_tier: Some("S".to_string()), // University = Tier S
                        risk_flags: vec![],
                        matched_rule_ids: vec![],
                        eligible_countries: vec![],
                        is_taiwan_eligible: None,
                    });
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
    let title_selectors = ["h2", "h3", "h4", "a", "strong", ".title"];
    
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
    // Look for currency patterns
    let patterns = [
        (r"£[\d,]+", ""),
        (r"\$[\d,]+", ""),
        (r"€[\d,]+", ""),
        (r"full tuition", "Full tuition"),
        (r"fully funded", "Fully funded"),
        (r"partial", "Partial funding"),
    ];
    
    let text_lower = text.to_lowercase();
    for (pattern, replacement) in &patterns {
        if let Ok(re) = regex::Regex::new(&format!("(?i){}", pattern)) {
            if let Some(m) = re.find(&text_lower) {
                if replacement.is_empty() {
                    return Some(text[m.start()..m.end()].to_string());
                } else {
                    return Some(replacement.to_string());
                }
            }
        }
    }
    None
}

fn extract_deadline(text: &str) -> Option<String> {
    // Look for date patterns
    if let Ok(re) = regex::Regex::new(r"(?i)(deadline|closes?|due|by)[:\s]+(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}|\d{1,2}\s+\w+\s+\d{4}|\w+\s+\d{1,2},?\s+\d{4})") {
        if let Some(caps) = re.captures(text) {
            if let Some(date) = caps.get(2) {
                return Some(date.as_str().to_string());
            }
        }
    }
    None
}

/// Known university scholarships as fallback
fn get_known_university_scholarships(url: &str) -> Vec<Lead> {
    let base_domain = url.split('/').take(3).collect::<Vec<_>>().join("/");
    
    vec![
        Lead {
            name: "Excellence Scholarship for International Students".to_string(),
            amount: "£5,000 - £10,000".to_string(),
            deadline: "2026-06-30".to_string(),
            source: url.to_string(),
            source_type: "university".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "International students".to_string(),
                "Postgraduate taught programmes".to_string(),
            ],
            notes: "Merit-based scholarship".to_string(),
            added_date: String::new(),
            url: format!("{}/scholarships/excellence", base_domain),
            match_score: 0,
            match_reasons: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(20), // Low effort - merit based
            trust_tier: Some("S".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // International students = Taiwan eligible
        },
        Lead {
            name: "Global Talent Scholarship".to_string(),
            amount: "£3,000".to_string(),
            deadline: "2026-05-31".to_string(),
            source: url.to_string(),
            source_type: "university".to_string(),
            status: "new".to_string(),
            eligibility: vec![
                "International students".to_string(),
                "All postgraduate programmes".to_string(),
            ],
            notes: "Automatic consideration upon application".to_string(),
            added_date: String::new(),
            url: format!("{}/scholarships/global-talent", base_domain),
            match_score: 0,
            match_reasons: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(0), // Auto-considered
            trust_tier: Some("S".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // International students = Taiwan eligible
        },
    ]
}

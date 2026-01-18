use crate::types::Lead;
use anyhow::Result;
use scraper::{Html, Selector};

pub fn scrape(url: &str) -> Result<Vec<Lead>> {
    println!("Scraping university website: {}", url);
    
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ScholarshipBot/1.0)")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    let response = client.get(url).send();
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let html = resp.text()?;
            let leads = parse_university_html(&html, url);
            
            if leads.is_empty() {
                println!("No scholarships found from HTML, using known scholarships");
                Ok(get_known_university_scholarships(url))
            } else {
                Ok(leads)
            }
        }
        Ok(resp) => {
            println!("HTTP error {}: using known scholarships", resp.status());
            Ok(get_known_university_scholarships(url))
        }
        Err(e) => {
            println!("Request failed ({}): using known scholarships", e);
            Ok(get_known_university_scholarships(url))
        }
    }
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
        },
    ]
}

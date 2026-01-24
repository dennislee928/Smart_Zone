//! Extraction Fallbacks Module
//!
//! Provides fallback extraction methods:
//! - JSON-LD / schema.org parsing
//! - Regex fallback for deadline/award with locale-aware parsing

use crate::types::{Lead, ExtractionEvidence};
use scraper::{Html, Selector};
use regex::Regex;
use serde_json::Value;

/// Extract scholarship data using multiple fallback methods
pub fn extract_with_fallbacks(html: &str, url: &str, lead: &mut Lead) {
    // Method 1: Try JSON-LD structured data
    if let Some(json_ld_data) = extract_json_ld(html) {
        extract_from_json_ld(&json_ld_data, url, lead);
    }
    
    // Method 2: Try schema.org microdata
    extract_from_schema_org(html, url, lead);
    
    // Method 3: Regex fallback for deadline/award
    extract_deadline_regex(html, url, lead);
    extract_award_regex(html, url, lead);
}

/// Extract JSON-LD structured data from HTML
fn extract_json_ld(html: &str) -> Option<Value> {
    let document = Html::parse_document(html);
    
    if let Ok(selector) = Selector::parse("script[type='application/ld+json']") {
        for element in document.select(&selector) {
            if let Some(text) = element.text().next() {
                if let Ok(json) = serde_json::from_str::<Value>(text) {
                    return Some(json);
                }
            }
        }
    }
    
    None
}

/// Extract data from JSON-LD
fn extract_from_json_ld(json: &Value, url: &str, lead: &mut Lead) {
    // Look for Scholarship schema
    if let Some(scholarship) = find_scholarship_in_json_ld(json) {
        if let Some(name) = scholarship.get("name").and_then(|v| v.as_str()) {
            if lead.name.is_empty() {
                lead.name = name.to_string();
                add_evidence(lead, "name", name, None, url, "json-ld");
            }
        }
        
        if let Some(amount) = scholarship.get("value").or_else(|| scholarship.get("amount"))
            .and_then(|v| v.as_str()) {
            if lead.amount.is_empty() || lead.amount == "See website" {
                lead.amount = amount.to_string();
                add_evidence(lead, "amount", amount, None, url, "json-ld");
            }
        }
        
        if let Some(deadline) = scholarship.get("applicationDeadline")
            .or_else(|| scholarship.get("deadline"))
            .and_then(|v| v.as_str()) {
            if lead.deadline.is_empty() || lead.deadline == "Check website" {
                lead.deadline = deadline.to_string();
                add_evidence(lead, "deadline", deadline, None, url, "json-ld");
            }
        }
    }
}

/// Find Scholarship schema in JSON-LD
fn find_scholarship_in_json_ld(json: &Value) -> Option<&Value> {
    match json {
        Value::Object(map) => {
            if let Some(type_val) = map.get("@type") {
                if type_val.as_str() == Some("Scholarship") || 
                   type_val.as_str() == Some("FinancialProduct") {
                    return Some(json);
                }
            }
            // Recursively search
            for value in map.values() {
                if let Some(found) = find_scholarship_in_json_ld(value) {
                    return Some(found);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(found) = find_scholarship_in_json_ld(item) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }
    None
}

/// Extract data from schema.org microdata
fn extract_from_schema_org(html: &str, url: &str, lead: &mut Lead) {
    let document = Html::parse_document(html);
    
    // Look for itemscope with itemtype="http://schema.org/Scholarship"
    if let Ok(selector) = Selector::parse("[itemscope][itemtype*='Scholarship']") {
        for element in document.select(&selector) {
            // Extract name
            if let Ok(name_sel) = Selector::parse("[itemprop='name']") {
                if let Some(name_elem) = element.select(&name_sel).next() {
                    if let Some(name) = name_elem.text().next() {
                        if lead.name.is_empty() {
                            lead.name = name.trim().to_string();
                            add_evidence(lead, "name", name.trim(), Some("[itemprop='name']".to_string()), url, "schema.org");
                        }
                    }
                }
            }
            
            // Extract amount
            if let Ok(amount_sel) = Selector::parse("[itemprop='value'], [itemprop='amount']") {
                if let Some(amount_elem) = element.select(&amount_sel).next() {
                    if let Some(amount) = amount_elem.text().next() {
                        if lead.amount.is_empty() || lead.amount == "See website" {
                            lead.amount = amount.trim().to_string();
                            add_evidence(lead, "amount", amount.trim(), Some("[itemprop='value']".to_string()), url, "schema.org");
                        }
                    }
                }
            }
            
            // Extract deadline
            if let Ok(deadline_sel) = Selector::parse("[itemprop='applicationDeadline'], [itemprop='deadline']") {
                if let Some(deadline_elem) = element.select(&deadline_sel).next() {
                    if let Some(deadline) = deadline_elem.text().next() {
                        if lead.deadline.is_empty() || lead.deadline == "Check website" {
                            lead.deadline = deadline.trim().to_string();
                            add_evidence(lead, "deadline", deadline.trim(), Some("[itemprop='applicationDeadline']".to_string()), url, "schema.org");
                        }
                    }
                }
            }
        }
    }
}

/// Extract deadline using regex with locale-aware parsing
fn extract_deadline_regex(html: &str, url: &str, lead: &mut Lead) {
    if !lead.deadline.is_empty() && lead.deadline != "Check website" && lead.deadline != "TBD" {
        return; // Already has deadline
    }
    
    // Locale-aware deadline patterns
    let patterns = vec![
        // UK format: DD/MM/YYYY or DD-MM-YYYY
        (r"(?i)(?:deadline|closing\s+date|apply\s+by)[:\s]+(\d{1,2})[/\-](\d{1,2})[/\-](\d{4})", "DD/MM/YYYY"),
        // US format: MM/DD/YYYY
        (r"(?i)(?:deadline|closing\s+date|apply\s+by)[:\s]+(\d{1,2})/(\d{1,2})/(\d{4})", "MM/DD/YYYY"),
        // ISO format: YYYY-MM-DD
        (r"(?i)(?:deadline|closing\s+date|apply\s+by)[:\s]+(\d{4})-(\d{2})-(\d{2})", "YYYY-MM-DD"),
        // Text format: "15 January 2026"
        (r"(?i)(?:deadline|closing\s+date|apply\s+by)[:\s]+(\d{1,2})\s+(january|february|march|april|may|june|july|august|september|october|november|december)\s+(\d{4})", "DD Month YYYY"),
    ];
    
    for (pattern, format_desc) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(html) {
                if let Some(matched) = caps.get(0) {
                    let deadline_str = matched.as_str();
                    if lead.deadline.is_empty() || lead.deadline == "Check website" {
                        lead.deadline = deadline_str.to_string();
                        let method = format!("regex-{}", format_desc);
                        add_evidence(lead, "deadline", deadline_str, Some(pattern.to_string()), url, &method);
                        break;
                    }
                }
            }
        }
    }
}

/// Extract award amount using regex with locale-aware parsing
fn extract_award_regex(html: &str, url: &str, lead: &mut Lead) {
    if !lead.amount.is_empty() && lead.amount != "See website" {
        return; // Already has amount
    }
    
    // Locale-aware amount patterns
    let patterns = vec![
        // UK format: £10,000 or £10,000.00
        (r"(?:£|GBP|pounds?)\s*([\d,]+(?:\.\d{2})?)", "GBP"),
        // US format: $10,000 or $10,000.00
        (r"(?:\$|USD|dollars?)\s*([\d,]+(?:\.\d{2})?)", "USD"),
        // EUR format: €10,000 or 10,000 EUR
        (r"(?:€|EUR|euros?)\s*([\d,]+(?:\.\d{2})?)|([\d,]+(?:\.\d{2})?)\s*(?:EUR|euros?)", "EUR"),
        // Generic: "10,000" or "10,000.00"
        (r"(?:award|value|amount|funding)[:\s]+([\d,]+(?:\.\d{2})?)", "Generic"),
    ];
    
    for (pattern, currency) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(html) {
                // Get first non-empty capture group
                for i in 1..caps.len() {
                    if let Some(matched) = caps.get(i) {
                        let amount_str = matched.as_str();
                        if !amount_str.is_empty() {
                            let formatted = format!("{}{}", 
                                if currency == "GBP" { "£" } else if currency == "USD" { "$" } else if currency == "EUR" { "€" } else { "" },
                                amount_str
                            );
                            lead.amount = formatted.clone();
                            let method = format!("regex-{}", currency);
                            add_evidence(lead, "amount", &formatted, Some(pattern.to_string()), url, &method);
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// Add extraction evidence to lead
fn add_evidence(lead: &mut Lead, attribute: &str, snippet: &str, selector: Option<String>, url: &str, method: &str) {
    lead.extraction_evidence.push(ExtractionEvidence {
        attribute: attribute.to_string(),
        snippet: snippet.to_string(),
        selector,
        url: url.to_string(),
        method: method.to_string(),
    });
}

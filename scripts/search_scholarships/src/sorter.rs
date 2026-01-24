use crate::types::Lead;
use chrono::{NaiveDate, Utc};

/// Calculate ROI score based on scholarship amount
pub fn calculate_roi_score(lead: &Lead) -> f64 {
    parse_amount(&lead.amount)
}

/// Calculate urgency score based on deadline proximity
pub fn calculate_urgency_score(lead: &Lead) -> i32 {
    if let Ok(deadline) = parse_deadline(&lead.deadline) {
        let now = Utc::now().date_naive();
        let days_until = (deadline - now).num_days();
        
        if days_until < 0 {
            // Past deadline
            return -100;
        } else if days_until <= 30 {
            // Very urgent (D-30 or less)
            return 100;
        } else if days_until <= 60 {
            // Urgent (D-60)
            return 50;
        } else if days_until <= 90 {
            // Somewhat urgent (D-90)
            return 25;
        } else if days_until <= 180 {
            // Normal (D-180)
            return 10;
        } else {
            // Not urgent
            return 0;
        }
    }
    0
}

/// Calculate source reliability score
pub fn calculate_source_reliability_score(lead: &Lead) -> i32 {
    match lead.source_type.as_str() {
        "university" => 50,
        "government" => 40,
        "ngo" => 30,
        "enterprise" => 20,
        "web3" => 10,
        "third_party" => 0,
        _ => 0,
    }
}

/// Calculate days until deadline (for display)
#[allow(dead_code)]
pub fn days_until_deadline(lead: &Lead) -> Option<i64> {
    if let Ok(deadline) = parse_deadline(&lead.deadline) {
        let now = Utc::now().date_naive();
        let days = (deadline - now).num_days();
        return Some(days);
    }
    None
}

/// Calculate comprehensive score for sorting
pub fn calculate_comprehensive_score(lead: &Lead) -> f64 {
    let match_score = lead.match_score as f64;
    let roi_score = calculate_roi_score(lead);
    let urgency_score = calculate_urgency_score(lead) as f64;
    let source_score = calculate_source_reliability_score(lead) as f64;
    
    // Normalize ROI score (assume max £50,000 = 100 points)
    let normalized_roi = (roi_score / 50000.0) * 100.0;
    
    // Comprehensive score: weighted combination
    // match_score (already weighted) + normalized_roi + urgency + source_reliability
    match_score + normalized_roi + urgency_score + source_score
}

/// Sort leads by comprehensive score, then urgency, then source reliability
pub fn sort_leads(leads: &mut [Lead]) {
    leads.sort_by(|a, b| {
        let score_a = calculate_comprehensive_score(a);
        let score_b = calculate_comprehensive_score(b);
        
        // Primary: comprehensive score (descending)
        match score_b.partial_cmp(&score_a) {
            Some(std::cmp::Ordering::Equal) => {
                // Secondary: urgency score (descending)
                let urgency_a = calculate_urgency_score(a);
                let urgency_b = calculate_urgency_score(b);
                match urgency_b.cmp(&urgency_a) {
                    std::cmp::Ordering::Equal => {
                        // Tertiary: source reliability (descending)
                        let source_a = calculate_source_reliability_score(a);
                        let source_b = calculate_source_reliability_score(b);
                        source_b.cmp(&source_a)
                    }
                    other => other,
                }
            }
            Some(other) => other,
            None => std::cmp::Ordering::Equal,
        }
    });
}

/// Parse amount string to numeric value (in GBP equivalent)
/// Supports: £5,000, $10,000, 5000 GBP, £5,000 - £10,000, etc.
fn parse_amount(amount_str: &str) -> f64 {
    // Remove whitespace and convert to lowercase
    let cleaned = amount_str.replace(" ", "").replace(",", "").to_lowercase();
    
    // Try to extract number ranges (e.g., "£5,000-£10,000" or "5000-10000")
    // Note: cleaned string is ASCII after replace operations, but use safe method
    if let Some(dash_pos) = cleaned.find('-') {
        let left: String = cleaned.chars().take(dash_pos).collect();
        let right: String = cleaned.chars().skip(dash_pos + 1).collect();
        
        let left_val = extract_number(&left);
        let right_val = extract_number(&right);
        
        if left_val > 0.0 && right_val > 0.0 {
            // Return average for ranges
            return (left_val + right_val) / 2.0;
        }
    }
    
    // Single value
    extract_number(&cleaned)
}

/// Extract numeric value from string, handling currency symbols
fn extract_number(text: &str) -> f64 {
    // Remove currency symbols and extract number
    let cleaned = text
        .replace("£", "")
        .replace("$", "")
        .replace("€", "")
        .replace("¥", "")
        .replace("gbp", "")
        .replace("usd", "")
        .replace("eur", "")
        .replace("jpy", "")
        .replace("cny", "");
    
    // Try to parse as float
    if let Ok(val) = cleaned.parse::<f64>() {
        // Convert to GBP equivalent (rough conversion)
        if text.contains("$") || text.contains("usd") {
            return val * 0.79; // USD to GBP (approximate)
        } else if text.contains("€") || text.contains("eur") {
            return val * 0.86; // EUR to GBP (approximate)
        } else if text.contains("¥") || text.contains("jpy") {
            return val * 0.0053; // JPY to GBP (approximate)
        } else if text.contains("cny") {
            return val * 0.11; // CNY to GBP (approximate)
        }
        return val;
    }
    
    0.0
}

/// Parse deadline string to NaiveDate with validation
/// Parse deadline string with validation
/// Rejects invalid dates like "68-58-58" by validating year (2020-2100), month (1-12), and day validity
fn parse_deadline(deadline: &str) -> Result<NaiveDate, ()> {
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
            // Validate date: check year, month, and day validity
            let year = date.year();
            let month = date.month();
            let day = date.day();
            
            // Validate year range (2020-2100)
            if year < 2020 || year > 2100 {
                continue; // Try next format
            }
            
            // Validate month range (1-12)
            if month < 1 || month > 12 {
                continue; // Try next format
            }
            
            // Validate day - from_ymd_opt would have failed if invalid, but double-check
            if NaiveDate::from_ymd_opt(year, month, day).is_some() {
                return Ok(date);
            }
        }
    }
    
    // Try to extract year-month-day from string using regex
    // This handles cases like "68-58-58" which should be rejected
    if let Ok(re) = regex::Regex::new(r"(\d{1,4})-(\d{1,2})-(\d{1,2})") {
        if let Some(caps) = re.captures(deadline) {
            let year: i32 = caps[1].parse().map_err(|_| ())?;
            let month: u32 = caps[2].parse().map_err(|_| ())?;
            let day: u32 = caps[3].parse().map_err(|_| ())?;
            
            // Validate year range (2020-2100)
            if year < 2020 || year > 2100 {
                return Err(());
            }
            
            // Validate month range (1-12)
            if month < 1 || month > 12 {
                return Err(());
            }
            
            // Validate day - from_ymd_opt will return None for invalid dates (e.g., Feb 30)
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Ok(date);
            }
        }
    }
    
    Err(())
}

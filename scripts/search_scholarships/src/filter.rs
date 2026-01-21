use crate::types::{Lead, Criteria, Profile};
use chrono::NaiveDate;
use std::collections::HashSet;

// ============================================
// Known Country Lists for Scholarship Programs
// ============================================

/// Commonwealth countries eligible for CSC scholarships (low and middle income)
const COMMONWEALTH_ELIGIBLE: &[&str] = &[
    "bangladesh", "belize", "botswana", "cameroon", "dominica", "eswatini", "fiji",
    "gambia", "ghana", "grenada", "guyana", "india", "jamaica", "kenya", "kiribati",
    "lesotho", "malawi", "malaysia", "maldives", "mauritius", "mozambique", "namibia",
    "nauru", "nigeria", "pakistan", "papua new guinea", "rwanda", "saint lucia",
    "saint vincent and the grenadines", "samoa", "seychelles", "sierra leone",
    "solomon islands", "south africa", "sri lanka", "tanzania", "tonga", "trinidad and tobago",
    "tuvalu", "uganda", "vanuatu", "zambia",
];

/// GREAT Scholarships 2026 eligible countries (varies by university)
const GREAT_SCHOLARSHIPS_COUNTRIES: &[&str] = &[
    "bangladesh", "china", "egypt", "ghana", "greece", "india", "indonesia",
    "kenya", "malaysia", "mauritius", "mexico", "nepal", "nigeria", "pakistan",
    "philippines", "spain", "sri lanka", "thailand", "turkey", "vietnam",
];

/// Chevening eligible countries (Taiwan IS eligible for Chevening)
const CHEVENING_ELIGIBLE: &[&str] = &[
    "taiwan", "china", "hong kong", "india", "pakistan", "bangladesh", "sri lanka",
    "nepal", "malaysia", "singapore", "indonesia", "philippines", "thailand", "vietnam",
    "japan", "south korea", "mongolia", "myanmar", "cambodia", "laos",
    // ... many more countries - Chevening covers 160+ countries
];

// ============================================
// Country Eligibility Parsing
// ============================================

/// Parse eligible countries from scholarship text and determine Taiwan eligibility
pub fn parse_eligible_countries(text: &str) -> (Vec<String>, Option<bool>) {
    let text_lower = text.to_lowercase();
    let mut eligible_countries: Vec<String> = Vec::new();
    let mut is_taiwan_eligible: Option<bool> = None;
    
    // 1. Check for explicit "Taiwan" mention
    if text_lower.contains("taiwan") || text_lower.contains("taiwanese") {
        is_taiwan_eligible = Some(true);
        eligible_countries.push("Taiwan".to_string());
    }
    
    // 2. Check for Commonwealth Scholarships (Taiwan NOT eligible)
    if text_lower.contains("commonwealth") && 
       (text_lower.contains("scholarship") || text_lower.contains("master")) {
        // Commonwealth scholarships are only for Commonwealth countries
        eligible_countries = COMMONWEALTH_ELIGIBLE.iter().map(|s| s.to_string()).collect();
        if is_taiwan_eligible.is_none() {
            is_taiwan_eligible = Some(false);
        }
        return (eligible_countries, is_taiwan_eligible);
    }
    
    // 3. Check for GREAT Scholarships with specific country lists
    if text_lower.contains("great scholarship") {
        // Check if specific countries are mentioned
        let great_countries: Vec<String> = GREAT_SCHOLARSHIPS_COUNTRIES.iter()
            .filter(|c| text_lower.contains(*c))
            .map(|s| s.to_string())
            .collect();
        
        if !great_countries.is_empty() {
            eligible_countries = great_countries;
            // Taiwan is NOT in GREAT Scholarships list
            if is_taiwan_eligible.is_none() {
                is_taiwan_eligible = Some(false);
            }
            return (eligible_countries, is_taiwan_eligible);
        }
    }
    
    // 4. Check for Chevening (Taiwan IS eligible)
    if text_lower.contains("chevening") {
        eligible_countries = vec!["Taiwan".to_string()]; // Simplified - Taiwan is eligible
        is_taiwan_eligible = Some(true);
        return (eligible_countries, is_taiwan_eligible);
    }
    
    // 5. Check for explicit country lists pattern: "eligible countries: X, Y, Z"
    if let Some(countries) = extract_country_list(&text_lower) {
        eligible_countries = countries.clone();
        let taiwan_in_list = countries.iter()
            .any(|c| c.to_lowercase().contains("taiwan"));
        if is_taiwan_eligible.is_none() {
            is_taiwan_eligible = Some(taiwan_in_list);
        }
        return (eligible_countries, is_taiwan_eligible);
    }
    
    // 6. Check for "international students" / "all nationalities" (Taiwan likely eligible)
    if text_lower.contains("international student") || 
       text_lower.contains("all nationalities") ||
       text_lower.contains("open to all") ||
       text_lower.contains("overseas student") {
        if is_taiwan_eligible.is_none() {
            is_taiwan_eligible = Some(true);
        }
    }
    
    // 7. Check for explicit exclusions
    let exclusion_patterns = [
        ("uk citizens only", false),
        ("british citizens only", false),
        ("eu citizens only", false),
        ("domestic students only", false),
        ("home students only", false),
        ("us citizens only", false),
    ];
    
    for (pattern, eligible) in &exclusion_patterns {
        if text_lower.contains(pattern) {
            is_taiwan_eligible = Some(*eligible);
            break;
        }
    }
    
    (eligible_countries, is_taiwan_eligible)
}

/// Extract country list from text patterns like "eligible countries: X, Y, Z"
fn extract_country_list(text: &str) -> Option<Vec<String>> {
    // Pattern: "eligible countries" or "open to students from" followed by country names
    let patterns = [
        r"(?i)eligible\s+countries?[:\s]+([^.]+)",
        r"(?i)open\s+to\s+students?\s+from[:\s]+([^.]+)",
        r"(?i)available\s+to\s+(?:students?\s+from\s+)?([^.]+(?:,\s*[^.]+)+)",
        r"(?i)nationals?\s+of[:\s]+([^.]+)",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(countries_str) = caps.get(1) {
                    let countries: Vec<String> = countries_str.as_str()
                        .split(&[',', ';', '/', '&'][..])
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty() && s.len() > 2)
                        .collect();
                    
                    if !countries.is_empty() {
                        return Some(countries);
                    }
                }
            }
        }
    }
    
    None
}

/// Update lead with country eligibility information
pub fn update_country_eligibility(lead: &mut Lead) {
    let text = format!("{} {} {} {}", 
        lead.name, 
        lead.notes, 
        lead.eligibility.join(" "),
        lead.url
    );
    
    let (countries, is_eligible) = parse_eligible_countries(&text);
    
    if !countries.is_empty() {
        lead.eligible_countries = countries;
    }
    
    if is_eligible.is_some() {
        lead.is_taiwan_eligible = is_eligible;
    }
}

/// Basic criteria matching (keywords)
pub fn matches_criteria(lead: &Lead, criteria: &Criteria) -> bool {
    let text = format!("{} {} {}", lead.name, lead.notes, lead.eligibility.join(" ")).to_lowercase();
    
    // Check excluded keywords
    for keyword in &criteria.criteria.excluded_keywords {
        if text.contains(&keyword.to_lowercase()) {
            return false;
        }
    }
    
    // Check required criteria (at least one must match)
    if !criteria.criteria.required.is_empty() {
        let matches_required = criteria.criteria.required.iter()
            .any(|req| text.contains(&req.to_lowercase()));
        if !matches_required {
            return false;
        }
    }
    
    true
}

/// Advanced profile-based filtering and scoring
pub fn filter_by_profile(lead: &mut Lead, profile: &Profile) -> bool {
    let text = format!("{} {} {} {}", 
        lead.name, 
        lead.notes, 
        lead.eligibility.join(" "),
        lead.url
    ).to_lowercase();
    
    let mut score: i32 = 0;
    let mut reasons: Vec<String> = Vec::new();
    let mut disqualified = false;
    let mut disqualify_reasons: Vec<String> = Vec::new();
    
    // === DISQUALIFICATION CHECKS ===
    
    // 1. Check nationality restrictions
    let nationality_lower = profile.nationality.to_lowercase();
    let restricted_nationalities = [
        ("us citizens only", "US citizens only"),
        ("american citizens", "US citizens only"),
        ("uk citizens only", "UK citizens only"),
        ("british citizens", "UK citizens only"),
        ("eu citizens only", "EU citizens only"),
        ("domestic students only", "Domestic students only"),
        ("home students only", "Home students only"),
    ];
    
    for (pattern, reason) in &restricted_nationalities {
        if text.contains(pattern) {
            disqualified = true;
            disqualify_reasons.push(format!("❌ {}", reason));
        }
    }
    
    // 2. Check programme level restrictions
    if text.contains("phd only") || text.contains("doctoral only") {
        disqualified = true;
        disqualify_reasons.push("❌ PhD only".to_string());
    }
    if text.contains("undergraduate only") || text.contains("bachelor only") {
        disqualified = true;
        disqualify_reasons.push("❌ Undergraduate only".to_string());
    }
    
    // 3. Check deadline - must be after min_deadline
    if let Some(min_deadline) = &profile.min_deadline {
        if let Ok(min_date) = NaiveDate::parse_from_str(min_deadline, "%Y-%m-%d") {
            if let Ok(lead_deadline) = parse_deadline(&lead.deadline) {
                if lead_deadline < min_date {
                    disqualified = true;
                    disqualify_reasons.push(format!("❌ Deadline {} is too early", lead.deadline));
                }
            }
        }
    }
    
    // 4. Check GPA requirements (if detectable)
    if let Some(max_gpa) = profile.max_gpa_requirement {
        if let Some(required_gpa) = extract_gpa_requirement(&text) {
            if required_gpa > max_gpa {
                disqualified = true;
                disqualify_reasons.push(format!("❌ Requires GPA {:.1}+", required_gpa));
            }
        }
    }
    
    // If disqualified, return false
    if disqualified {
        lead.match_reasons = disqualify_reasons;
        lead.match_score = -100;
        return false;
    }
    
    // === POSITIVE SCORING ===
    
    // 1. Target university match (+50)
    let target_uni = profile.target_university.to_lowercase();
    if text.contains(&target_uni) || text.contains("glasgow") {
        score += 50;
        reasons.push("🎯 Target university (Glasgow)".to_string());
    }
    
    // 2. Target country match (+20)
    let target_country = profile.target_country.to_lowercase();
    if text.contains(&target_country) || text.contains("uk") || text.contains("united kingdom") || text.contains("britain") {
        score += 20;
        reasons.push("🇬🇧 UK scholarship".to_string());
    }
    
    // 3. Nationality eligible (+30)
    if text.contains(&nationality_lower) || text.contains("taiwan") {
        score += 30;
        reasons.push("🇹🇼 Taiwan eligible".to_string());
    }
    
    // 4. International students welcome (+15)
    if text.contains("international") || text.contains("overseas") || text.contains("all nationalities") {
        score += 15;
        reasons.push("🌍 International students".to_string());
    }
    
    // 5. Programme level match (+20)
    let level = profile.programme_level.to_lowercase();
    if text.contains(&level) || text.contains("postgraduate") || text.contains("taught") {
        score += 20;
        reasons.push("📚 Master's level".to_string());
    }
    
    // 6. Full funding bonus (+25)
    if text.contains("full tuition") || text.contains("fully funded") || text.contains("full cost") {
        score += 25;
        reasons.push("💰 Full funding".to_string());
    }
    
    // 7. No GPA requirement or low requirement (+10)
    if !text.contains("gpa") && !text.contains("grade point") {
        score += 10;
        reasons.push("✅ No GPA requirement stated".to_string());
    }
    
    // 8. Merit-based (good for high GPA) (+15)
    if text.contains("merit") || text.contains("academic excellence") || text.contains("outstanding") {
        // Check if user has good GPA in any degree
        let has_good_gpa = profile.education.iter().any(|e| e.gpa >= 3.5);
        if has_good_gpa {
            score += 15;
            reasons.push("⭐ Merit-based (GPA 3.96)".to_string());
        }
    }
    
    // 9. Deadline timing bonus
    if let Ok(deadline) = parse_deadline(&lead.deadline) {
        let programme_start = NaiveDate::parse_from_str(&profile.programme_start, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 9, 14).unwrap());
        
        let days_before = (programme_start - deadline).num_days();
        if days_before > 30 && days_before < 365 {
            score += 10;
            reasons.push(format!("📅 Good timing ({})", lead.deadline));
        }
    }
    
    // Update lead with score and reasons
    lead.match_score = score;
    lead.match_reasons = if reasons.is_empty() {
        vec!["General scholarship".to_string()]
    } else {
        reasons
    };
    
    // Return true if score > 0 (qualified)
    score > 0
}

/// Parse various deadline formats
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
            return Ok(date);
        }
    }
    
    // Try to extract year-month-day from string
    if let Ok(re) = regex::Regex::new(r"(\d{4})-(\d{2})-(\d{2})") {
        if let Some(caps) = re.captures(deadline) {
            let year: i32 = caps[1].parse().unwrap_or(2026);
            let month: u32 = caps[2].parse().unwrap_or(1);
            let day: u32 = caps[3].parse().unwrap_or(1);
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return Ok(date);
            }
        }
    }
    
    Err(())
}

/// Extract GPA requirement from text
fn extract_gpa_requirement(text: &str) -> Option<f64> {
    if let Ok(re) = regex::Regex::new(r"(?i)gpa\s*(?:of\s*)?(\d+\.?\d*)\s*(?:\+|or\s*above|minimum)?") {
        if let Some(caps) = re.captures(text) {
            if let Ok(gpa) = caps[1].parse::<f64>() {
                return Some(gpa);
            }
        }
    }
    None
}

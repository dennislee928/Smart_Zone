use crate::types::{Lead, Criteria, Profile};
use chrono::NaiveDate;

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

// ============================================
// Structured Date Parsing
// ============================================

/// Date type classification
#[derive(Debug, Clone, PartialEq)]
pub enum DateType {
    ApplicationDeadline,  // "applications close", "deadline", "closing date"
    StudyStart,           // "study starts", "programme begins", "start date"
    ResultsAnnounced,     // "results announced", "notification date"
    OfferDeadline,        // "offer deadline", "accept by"
    Unknown,
}

/// Parsed date with context
#[derive(Debug, Clone)]
pub struct ParsedDate {
    pub date: String,           // ISO format YYYY-MM-DD
    pub date_type: DateType,
    pub label: String,          // Original label text
    pub confidence: String,     // "confirmed", "inferred", "unknown"
}

/// Parse structured dates from scholarship text
pub fn parse_structured_dates(text: &str) -> Vec<ParsedDate> {
    let text_lower = text.to_lowercase();
    let mut dates = Vec::new();
    
    // Patterns for application deadlines (highest priority)
    let deadline_patterns = [
        (r"(?i)application[s]?\s+(?:deadline|close[s]?|due)[:\s]+(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})", "applications close"),
        (r"(?i)(?:deadline|closing\s+date)[:\s]+(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})", "deadline"),
        (r"(?i)(?:deadline|closing\s+date)[:\s]+(\d{1,2}\s+\w+\s+\d{4})", "deadline"),
        (r"(?i)(?:apply\s+by|submit\s+by)[:\s]+(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})", "apply by"),
        (r"(?i)(?:apply\s+by|submit\s+by)[:\s]+(\d{1,2}\s+\w+\s+\d{4})", "apply by"),
        (r"(?i)closes?[:\s]+(\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4})", "closes"),
        (r"(?i)closes?[:\s]+(\d{1,2}\s+\w+\s+\d{4})", "closes"),
    ];
    
    for (pattern, label) in &deadline_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for caps in re.captures_iter(&text_lower) {
                if let Some(date_match) = caps.get(1) {
                    if let Some(iso_date) = normalize_date(date_match.as_str()) {
                        dates.push(ParsedDate {
                            date: iso_date,
                            date_type: DateType::ApplicationDeadline,
                            label: label.to_string(),
                            confidence: "confirmed".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // Patterns for study start dates
    let study_start_patterns = [
        (r"(?i)(?:study|programme|course)\s+(?:starts?|begins?|commences?)[:\s]+(\w+\s+\d{4})", "study starts"),
        (r"(?i)(?:start(?:ing)?|begin(?:ning)?)\s+(?:in\s+)?(\w+\s+\d{4})", "starting"),
        (r"(?i)(?:september|october)\s+(\d{4})\s+(?:intake|entry|start)", "intake"),
    ];
    
    for (pattern, label) in &study_start_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for caps in re.captures_iter(&text_lower) {
                if let Some(date_match) = caps.get(1) {
                    if let Some(iso_date) = normalize_date(date_match.as_str()) {
                        dates.push(ParsedDate {
                            date: iso_date,
                            date_type: DateType::StudyStart,
                            label: label.to_string(),
                            confidence: "inferred".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // Extract intake year patterns like "2026/27" or "2026-27"
    if let Ok(re) = regex::Regex::new(r"(?i)(\d{4})[/\-](\d{2})\s*(?:intake|session|academic\s*year)?") {
        for caps in re.captures_iter(&text_lower) {
            if let (Some(year1), Some(_year2)) = (caps.get(1), caps.get(2)) {
                let year = year1.as_str();
                // Infer September start for academic year
                dates.push(ParsedDate {
                    date: format!("{}-09-01", year),
                    date_type: DateType::StudyStart,
                    label: "academic year".to_string(),
                    confidence: "inferred".to_string(),
                });
            }
        }
    }
    
    dates
}

/// Normalize various date formats to ISO YYYY-MM-DD
fn normalize_date(date_str: &str) -> Option<String> {
    let date_str = date_str.trim();
    
    // Try common formats
    let formats = [
        "%Y-%m-%d",
        "%d/%m/%Y",
        "%m/%d/%Y",
        "%d-%m-%Y",
        "%d %B %Y",
        "%d %b %Y",
        "%B %d, %Y",
        "%b %d, %Y",
        "%B %Y",      // Month Year only
        "%b %Y",
    ];
    
    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, fmt) {
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }
    
    // Try to extract year-month-day from string
    if let Ok(re) = regex::Regex::new(r"(\d{4})-(\d{2})-(\d{2})") {
        if let Some(caps) = re.captures(date_str) {
            return Some(format!("{}-{}-{}", &caps[1], &caps[2], &caps[3]));
        }
    }
    
    // Try month year format (e.g., "September 2026")
    let months = [
        ("january", "01"), ("february", "02"), ("march", "03"), ("april", "04"),
        ("may", "05"), ("june", "06"), ("july", "07"), ("august", "08"),
        ("september", "09"), ("october", "10"), ("november", "11"), ("december", "12"),
        ("jan", "01"), ("feb", "02"), ("mar", "03"), ("apr", "04"),
        ("jun", "06"), ("jul", "07"), ("aug", "08"), ("sep", "09"),
        ("oct", "10"), ("nov", "11"), ("dec", "12"),
    ];
    
    let date_lower = date_str.to_lowercase();
    for (month_name, month_num) in &months {
        if date_lower.contains(month_name) {
            if let Ok(re) = regex::Regex::new(r"(\d{4})") {
                if let Some(caps) = re.captures(&date_lower) {
                    return Some(format!("{}-{}-01", &caps[1], month_num));
                }
            }
        }
    }
    
    None
}

/// Update lead with structured date information
pub fn update_structured_dates(lead: &mut Lead) {
    let text = format!("{} {} {} {}", 
        lead.name, 
        lead.notes, 
        lead.eligibility.join(" "),
        lead.deadline
    );
    
    let parsed_dates = parse_structured_dates(&text);
    
    // Find the best application deadline
    for parsed in &parsed_dates {
        if parsed.date_type == DateType::ApplicationDeadline {
            lead.deadline_date = Some(parsed.date.clone());
            lead.deadline_label = Some(parsed.label.clone());
            lead.deadline_confidence = Some(parsed.confidence.clone());
            break;
        }
    }
    
    // Find study start date
    for parsed in &parsed_dates {
        if parsed.date_type == DateType::StudyStart {
            lead.study_start = Some(parsed.date.clone());
            // Extract intake year
            if let Some(year) = parsed.date.split('-').next() {
                let next_year: i32 = year.parse().unwrap_or(2026) + 1;
                lead.intake_year = Some(format!("{}/{}", year, next_year % 100));
            }
            break;
        }
    }
    
    // If we still don't have a deadline_date but have a deadline string, try to parse it
    if lead.deadline_date.is_none() && !lead.deadline.is_empty() {
        if let Some(iso_date) = normalize_date(&lead.deadline) {
            lead.deadline_date = Some(iso_date);
            lead.deadline_confidence = Some("inferred".to_string());
        } else if lead.deadline.to_lowercase().contains("check") || 
                  lead.deadline.to_lowercase().contains("tbd") ||
                  lead.deadline.to_lowercase().contains("see website") {
            lead.deadline_confidence = Some("unknown".to_string());
        }
    }
}

// ============================================
// Deduplication Logic
// ============================================

/// Directory page patterns - these are landing pages, not specific scholarships
const DIRECTORY_PATTERNS: &[&str] = &[
    "/scholarships/$",
    "/scholarships/search",
    "/scholarships/find",
    "/funding/$",
    "/funding/search",
    "/bursaries/$",
    "/financial-support/$",
    "/student-finance/$",
];

/// Generate canonical URL for deduplication
pub fn generate_canonical_url(url: &str) -> String {
    let mut canonical = url.to_lowercase();
    
    // Remove trailing slashes
    while canonical.ends_with('/') {
        canonical.pop();
    }
    
    // Remove common tracking parameters
    if let Some(idx) = canonical.find('?') {
        canonical = canonical[..idx].to_string();
    }
    
    // Remove fragment
    if let Some(idx) = canonical.find('#') {
        canonical = canonical[..idx].to_string();
    }
    
    // Normalize www prefix
    canonical = canonical.replace("://www.", "://");
    
    // Remove protocol for comparison
    canonical = canonical.replace("https://", "").replace("http://", "");
    
    canonical
}

/// Check if a URL is a directory/landing page rather than a specific scholarship
pub fn is_directory_page(url: &str, name: &str) -> bool {
    let url_lower = url.to_lowercase();
    let name_lower = name.to_lowercase();
    
    // Check URL patterns
    for pattern in DIRECTORY_PATTERNS {
        if pattern.ends_with('$') {
            // Exact match at end
            let pattern_clean = &pattern[..pattern.len()-1];
            if url_lower.ends_with(pattern_clean) {
                return true;
            }
        } else if url_lower.contains(pattern) {
            return true;
        }
    }
    
    // Check if name is generic
    let generic_names = [
        "scholarships",
        "find a scholarship",
        "scholarship search",
        "funding opportunities",
        "bursaries",
        "financial support",
        "scholarships and bursaries",
        "scholarships listing",
        "scholarship database",
    ];
    
    for generic in &generic_names {
        if name_lower == *generic || name_lower.starts_with(generic) {
            return true;
        }
    }
    
    false
}

/// Check if a lead has sufficient detail to be a valid scholarship entry
pub fn has_sufficient_detail(lead: &Lead) -> bool {
    // Must have a meaningful name (not generic)
    let name_lower = lead.name.to_lowercase();
    if name_lower.len() < 10 {
        return false;
    }
    
    // Generic names are not sufficient
    let generic_patterns = [
        "scholarships",
        "find a scholarship",
        "apply for",
        "search",
        "browse",
    ];
    
    for pattern in &generic_patterns {
        if name_lower == *pattern {
            return false;
        }
    }
    
    // Should have either:
    // 1. A specific amount (not "See website")
    // 2. A specific deadline (not "Check website")
    // 3. Specific eligibility criteria
    
    let has_amount = !lead.amount.is_empty() && 
                     !lead.amount.to_lowercase().contains("see website") &&
                     !lead.amount.to_lowercase().contains("check website");
    
    let has_deadline = !lead.deadline.is_empty() && 
                       !lead.deadline.to_lowercase().contains("check website") &&
                       !lead.deadline.to_lowercase().contains("tbd");
    
    let has_eligibility = !lead.eligibility.is_empty() && 
                          !lead.eligibility.iter().any(|e| 
                              e.to_lowercase().contains("see website"));
    
    // At least one specific detail required
    has_amount || has_deadline || has_eligibility
}

/// Update lead with deduplication information
pub fn update_dedup_info(lead: &mut Lead) {
    // Generate canonical URL
    lead.canonical_url = Some(generate_canonical_url(&lead.url));
    
    // Check if this is a directory page
    lead.is_directory_page = is_directory_page(&lead.url, &lead.name);
}

/// Generate deduplication key for a lead
pub fn generate_dedup_key(lead: &Lead) -> String {
    let canonical = lead.canonical_url.as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&lead.url);
    
    // Use canonical URL + normalized name as key
    let name_normalized = lead.name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    
    format!("{}|{}", canonical, name_normalized)
}

// ============================================
// Trust Tier & Source Priority Logic
// ============================================

use crate::types::TrustTier;

/// Known official scholarship domains (Tier S)
const TIER_S_DOMAINS: &[&str] = &[
    // UK Universities
    "gla.ac.uk", "glasgow.ac.uk", "ox.ac.uk", "cam.ac.uk", "imperial.ac.uk",
    "ucl.ac.uk", "lse.ac.uk", "kcl.ac.uk", "ed.ac.uk", "manchester.ac.uk",
    "bristol.ac.uk", "warwick.ac.uk", "leeds.ac.uk", "birmingham.ac.uk",
    "sheffield.ac.uk", "nottingham.ac.uk", "southampton.ac.uk", "york.ac.uk",
    "durham.ac.uk", "exeter.ac.uk", "bath.ac.uk", "st-andrews.ac.uk",
    "liverpool.ac.uk", "cardiff.ac.uk", "qub.ac.uk", "abdn.ac.uk",
    // Government
    "gov.uk", "ukri.org", "cscuk.fcdo.gov.uk",
];

/// Known major foundation domains (Tier A)
const TIER_A_DOMAINS: &[&str] = &[
    "gatescambridge.org", "rhodeshouse.ox.ac.uk", "chevening.org",
    "marshallscholarship.org", "wellcome.org", "leverhulme.ac.uk",
    "carnegie-trust.org", "wolfson.org.uk",
];

/// Known verified aggregator domains (Tier B)
const TIER_B_DOMAINS: &[&str] = &[
    "britishcouncil.org", "study-uk.britishcouncil.org", "findaphd.com",
    "scholarshipportal.com", "prospects.ac.uk", "postgraduatesearch.com",
];

/// Determine trust tier from URL
pub fn determine_trust_tier(url: &str) -> TrustTier {
    let url_lower = url.to_lowercase();
    
    // Check Tier S (official sources)
    for domain in TIER_S_DOMAINS {
        if url_lower.contains(domain) {
            return TrustTier::S;
        }
    }
    
    // Check Tier A (major foundations)
    for domain in TIER_A_DOMAINS {
        if url_lower.contains(domain) {
            return TrustTier::A;
        }
    }
    
    // Check Tier B (verified aggregators)
    for domain in TIER_B_DOMAINS {
        if url_lower.contains(domain) {
            return TrustTier::B;
        }
    }
    
    // Default to Tier C (unverified)
    TrustTier::C
}

/// Try to find official source URL for a scholarship found via aggregator
pub fn find_official_source(lead: &Lead) -> Option<String> {
    let name_lower = lead.name.to_lowercase();
    
    // Known scholarship -> official URL mappings
    let known_mappings = [
        ("chevening", "https://www.chevening.org/scholarships/"),
        ("commonwealth", "https://cscuk.fcdo.gov.uk/scholarships/"),
        ("gates cambridge", "https://www.gatescambridge.org/"),
        ("rhodes", "https://www.rhodeshouse.ox.ac.uk/scholarships/"),
        ("marshall", "https://www.marshallscholarship.org/"),
        ("clarendon", "https://www.ox.ac.uk/clarendon/"),
    ];
    
    for (pattern, official_url) in &known_mappings {
        if name_lower.contains(pattern) {
            return Some(official_url.to_string());
        }
    }
    
    // Try to infer university official URL from name
    let university_patterns = [
        ("glasgow", "https://www.gla.ac.uk/scholarships/"),
        ("oxford", "https://www.ox.ac.uk/admissions/graduate/fees-and-funding/"),
        ("cambridge", "https://www.cam.ac.uk/study/postgraduate/funding"),
        ("imperial", "https://www.imperial.ac.uk/study/pg/fees-and-funding/"),
        ("ucl", "https://www.ucl.ac.uk/scholarships/"),
        ("edinburgh", "https://www.ed.ac.uk/student-funding/postgraduate"),
        ("manchester", "https://www.manchester.ac.uk/study/postgraduate/fees-and-funding/"),
    ];
    
    for (pattern, official_url) in &university_patterns {
        if name_lower.contains(pattern) {
            return Some(official_url.to_string());
        }
    }
    
    None
}

/// Update lead with trust tier and official source information
pub fn update_trust_info(lead: &mut Lead) {
    // Determine trust tier from URL
    let tier = determine_trust_tier(&lead.url);
    lead.trust_tier = Some(tier.to_string());
    
    // If from aggregator (Tier B or C), try to find official source
    if tier == TrustTier::B || tier == TrustTier::C {
        if let Some(official_url) = find_official_source(lead) {
            lead.official_source_url = Some(official_url);
        }
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

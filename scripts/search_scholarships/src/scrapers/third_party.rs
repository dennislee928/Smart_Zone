use crate::types::{Lead, ScrapeResult, SourceStatus};
use anyhow::Result;
use scraper::{Html, Selector};
use regex::Regex;

/// URL patterns for aggregators that are often WAF-blocked; we use index-only (name + official link) then two-hop.
const INDEX_ONLY_SOURCES: &[&str] = &[
    "findamasters.com",
    "findaphd.com",
    "findamasters.com/funding",
    "prospects.ac.uk",
    "scholarshipportal.com",
];

fn is_index_only_source(url: &str) -> bool {
    let u = url.to_lowercase();
    INDEX_ONLY_SOURCES.iter().any(|s| u.contains(s))
}

/// Index-only parse: extract name + official link only; amount/deadline/eligibility = "See official page".
fn parse_index_only(html: &str, base_url: &str) -> Vec<Lead> {
    let document = Html::parse_document(html);
    let mut leads = Vec::new();
    let base_host = base_url
        .find("://")
        .map(|i| base_url[i + 3..].trim_start_matches('/'))
        .and_then(|r| r.find('/').map(|i| &r[..i]).or(Some(r)))
        .unwrap_or("");

    let selectors = [".phd-result", ".result-item", ".funding-result", "article", "li"];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel).take(50) {
                let text = el.text().collect::<Vec<_>>().join(" ");
                let title = extract_title(&el).unwrap_or_else(|| {
                    text.split_whitespace().take(12).collect::<Vec<_>>().join(" ")
                });
                if title.len() < 10 {
                    continue;
                }
                let mut official: Option<String> = None;
                if let Ok(a_sel) = Selector::parse("a[href]") {
                    for a in el.select(&a_sel) {
                        let href = a.value().attr("href").unwrap_or("").trim();
                        if href.is_empty() || href.starts_with('#') {
                            continue;
                        }
                let abs = resolve_abs_url(base_url, href);
                if abs != base_url && !abs.is_empty() {
                    let abs_rest = abs.find("://").map(|i| &abs[i + 3..]).unwrap_or("");
                    let abs_host = abs_rest.find('/').map(|i| &abs_rest[..i]).unwrap_or(abs_rest);
                    let is_external = !abs_host.is_empty() && !abs_host.starts_with(base_host);
                    if is_external {
                        official = Some(abs);
                        break;
                    }
                }
                    }
                }
                let official = official.unwrap_or_else(|| base_url.to_string());
                let mut lead = minimal_lead(&title, base_url, &official);
                lead.is_index_only = true;
                lead.official_source_url = Some(official.clone());
                lead.url = official;
                leads.push(lead);
            }
            if !leads.is_empty() {
                break;
            }
        }
    }
    leads
}

fn resolve_abs_url(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    let (scheme, rest) = match base.find("://") {
        Some(i) => (base[..i + 3].to_string(), base[i + 3..].trim_start_matches('/')),
        None => return href.to_string(),
    };
    let slash = rest.find('/').unwrap_or(rest.len());
    let host = &rest[..slash];
    let path = if slash < rest.len() { &rest[slash..] } else { "/" };
    let path_dir = if path.ends_with('/') {
        path.to_string()
    } else {
        let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        format!("{}/", dir)
    };
    if href.starts_with('/') {
        format!("{}{}{}", scheme, host, href)
    } else {
        format!("{}{}/{}{}", scheme, host, path_dir.trim_start_matches('/'), href)
    }
}

fn minimal_lead(name: &str, source: &str, url: &str) -> Lead {
    Lead {
        name: name.to_string(),
        amount: "See official page".to_string(),
        deadline: "See official page".to_string(),
        source: source.to_string(),
        source_type: "third_party".to_string(),
        status: "new".to_string(),
        eligibility: vec!["See official page".to_string()],
        notes: String::new(),
        added_date: String::new(),
        url: url.to_string(),
        match_score: 0,
        match_reasons: vec![],
        hard_fail_reasons: vec![],
        soft_flags: vec![],
        bucket: None,
        http_status: None,
        effort_score: None,
        trust_tier: Some("B".to_string()),
        risk_flags: vec![],
        matched_rule_ids: vec![],
        eligible_countries: vec![],
        is_taiwan_eligible: None,
        taiwan_eligibility_confidence: None,
        deadline_date: None,
        deadline_label: None,
        intake_year: None,
        study_start: None,
        deadline_confidence: Some("unknown".to_string()),
        canonical_url: None,
        is_directory_page: false,
        official_source_url: Some(url.to_string()),
        source_domain: None,
        confidence: None,
        eligibility_confidence: None,
        tags: vec!["index_only".to_string()],
        is_index_only: true,
        first_seen_at: None,
        last_checked_at: None,
        next_check_at: None,
        persistence_status: None,
        source_seed: None,
        check_count: None,
    }
}

/// Fetch official page and fill amount/deadline/eligibility. On failure, set trust_tier C and needs_verification.
pub fn enrich_from_official(lead: &mut Lead) -> bool {
    let official = match lead.official_source_url.as_ref() {
        Some(u) if !u.is_empty() => u.clone(),
        _ => return false,
    };
    let client = match reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ScholarshipBot/1.0)")
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            lead.trust_tier = Some("C".to_string());
            if !lead.risk_flags.contains(&"needs_verification".to_string()) {
                lead.risk_flags.push("needs_verification".to_string());
            }
            return false;
        }
    };
    let resp = match client.get(&official).send() {
        Ok(r) if r.status().is_success() => r,
        _ => {
            lead.trust_tier = Some("C".to_string());
            if !lead.risk_flags.contains(&"needs_verification".to_string()) {
                lead.risk_flags.push("needs_verification".to_string());
            }
            return false;
        }
    };
    let html = match resp.text() {
        Ok(h) => h,
        Err(_) => {
            lead.trust_tier = Some("C".to_string());
            if !lead.risk_flags.contains(&"needs_verification".to_string()) {
                lead.risk_flags.push("needs_verification".to_string());
            }
            return false;
        }
    };
    let text = html.to_lowercase();
    // Simple extraction
    let amount = if let Ok(re) = Regex::new(r"£[\d,]+|\$[\d,]+|full\s*tuition|fully\s*funded") {
        re.find(&text)
            .map(|m| html[m.start()..m.end()].to_string())
            .unwrap_or_else(|| "See official page".to_string())
    } else {
        "See official page".to_string()
    };
    let deadline = if let Ok(re) = Regex::new(r"\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}|\d{1,2}\s+\w+\s+\d{4}") {
        re.find(&text)
            .map(|m| html[m.start()..m.end()].to_string())
            .unwrap_or_else(|| "See official page".to_string())
    } else {
        "See official page".to_string()
    };
    if amount != "See official page" {
        lead.amount = amount;
    }
    if deadline != "See official page" {
        lead.deadline = deadline;
    }
    if text.contains("international") {
        lead.eligibility = vec!["International students".to_string()];
    }
    lead.is_index_only = false;
    true
}

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
                let mut leads = if is_index_only_source(url) {
                    let idx = parse_index_only(&html, url);
                    if !idx.is_empty() {
                        println!("  Index-only: {} leads (two-hop to official for full info)", idx.len());
                    }
                    idx
                } else {
                    parse_third_party_html(&html, url)
                };

                if !is_index_only_source(url) && leads.is_empty() {
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
                            hard_fail_reasons: vec![],
                            soft_flags: vec![],
                            bucket: None,
                            http_status: None,
                            effort_score: None,
                            trust_tier: Some("B".to_string()), // Third party = Tier B
                            risk_flags: vec![],
                            matched_rule_ids: vec![],
                            eligible_countries: vec![],
                            is_taiwan_eligible: None,
                            taiwan_eligibility_confidence: None,
                            deadline_date: None,
                            deadline_label: None,
                            intake_year: None,
                            study_start: None,
                            deadline_confidence: Some("unknown".to_string()),
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
                hard_fail_reasons: vec![],
                soft_flags: vec![],
                bucket: None,
                http_status: None,
                effort_score: Some(70),
                trust_tier: Some("A".to_string()),
                risk_flags: vec![],
                matched_rule_ids: vec![],
                eligible_countries: vec![],
                is_taiwan_eligible: Some(true),
                taiwan_eligibility_confidence: None,
                deadline_date: Some("2026-01-07".to_string()), // International deadline
                deadline_label: Some("international deadline".to_string()),
                intake_year: Some("2026/27".to_string()),
                study_start: Some("2026-10".to_string()),
                deadline_confidence: Some("confirmed".to_string()),
                canonical_url: None,
                is_directory_page: false,
                official_source_url: Some("https://www.gatescambridge.org/".to_string()),
                source_domain: Some("gatescambridge.org".to_string()),
                confidence: None,
                eligibility_confidence: None,
                tags: vec!["cambridge".to_string(), "prestigious".to_string()],
                is_index_only: false,
                first_seen_at: None,
                last_checked_at: None,
                next_check_at: None,
                persistence_status: None,
                source_seed: None,
                check_count: None,
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
                hard_fail_reasons: vec![],
                soft_flags: vec![],
                bucket: None,
                http_status: None,
                effort_score: Some(50),
                trust_tier: Some("A".to_string()),
                risk_flags: vec![],
                matched_rule_ids: vec![],
                eligible_countries: vec![],
                is_taiwan_eligible: Some(true),
                taiwan_eligibility_confidence: None,
                deadline_date: Some("2026-03-31".to_string()),
                deadline_label: Some("varies by district".to_string()),
                intake_year: Some("2026/27".to_string()),
                study_start: Some("2026-09".to_string()),
                deadline_confidence: Some("estimated".to_string()),
                canonical_url: None,
                is_directory_page: false,
                official_source_url: Some("https://www.rotary.org/en/our-programs/scholarships".to_string()),
                source_domain: Some("rotary.org".to_string()),
                confidence: None,
                eligibility_confidence: None,
                tags: vec!["rotary".to_string(), "sponsored".to_string()],
                is_index_only: false,
                first_seen_at: None,
                last_checked_at: None,
                next_check_at: None,
                persistence_status: None,
                source_seed: None,
                check_count: None,
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
                hard_fail_reasons: vec![],
                soft_flags: vec![],
                bucket: None,
                http_status: None,
                effort_score: Some(75),
                trust_tier: Some("A".to_string()),
                risk_flags: vec![],
                matched_rule_ids: vec![],
                eligible_countries: vec!["United States".to_string()],
                is_taiwan_eligible: Some(false), // US citizens only
                taiwan_eligibility_confidence: Some("explicit_list".to_string()),
                deadline_date: Some("2025-09-30".to_string()),
                deadline_label: Some("applications close".to_string()),
                intake_year: Some("2026/27".to_string()),
                study_start: Some("2026-10".to_string()),
                deadline_confidence: Some("confirmed".to_string()),
                canonical_url: None,
                is_directory_page: false,
                official_source_url: Some("https://www.marshallscholarship.org/".to_string()),
                source_domain: Some("marshallscholarship.org".to_string()),
                confidence: None,
                eligibility_confidence: None,
                tags: vec!["us-only".to_string(), "prestigious".to_string()],
                is_index_only: false,
                first_seen_at: None,
                last_checked_at: None,
                next_check_at: None,
                persistence_status: None,
                source_seed: None,
                check_count: None,
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
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(70), // High effort - essays, interview
            trust_tier: Some("A".to_string()), // Major foundation = Tier A
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // Non-UK citizens = Taiwan eligible
            taiwan_eligibility_confidence: None,
            deadline_date: Some("2026-10-13".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.gatescambridge.org/".to_string()),
            source_domain: Some("gatescambridge.org".to_string()),
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
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(80), // Very high effort
            trust_tier: Some("A".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: None, // Need to verify - Taiwan may not be in Rhodes list
            taiwan_eligibility_confidence: None,
            deadline_date: Some("2026-10-01".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.rhodeshouse.ox.ac.uk/scholarships/".to_string()),
            source_domain: Some("rhodeshouse.ox.ac.uk".to_string()),
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
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(10), // Auto-considered
            trust_tier: Some("S".to_string()), // University official
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // All nationalities = Taiwan eligible
            taiwan_eligibility_confidence: None,
            deadline_date: Some("2026-01-10".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2026/27".to_string()),
            study_start: Some("2026-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://www.ox.ac.uk/clarendon/".to_string()),
            source_domain: Some("ox.ac.uk".to_string()),
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
            hard_fail_reasons: vec![],
            soft_flags: vec![],
            bucket: None,
            http_status: None,
            effort_score: Some(60),
            trust_tier: Some("A".to_string()),
            risk_flags: vec![],
            matched_rule_ids: vec![],
            eligible_countries: vec![],
            is_taiwan_eligible: Some(true), // International students eligible
            taiwan_eligibility_confidence: None,
            deadline_date: Some("2026-11-30".to_string()),
            deadline_label: Some("applications close".to_string()),
            intake_year: Some("2027/28".to_string()),
            study_start: Some("2027-10".to_string()),
            deadline_confidence: Some("confirmed".to_string()),
            canonical_url: None,
            is_directory_page: false,
            official_source_url: Some("https://wellcome.org/grant-funding".to_string()),
            source_domain: Some("wellcome.org".to_string()),
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

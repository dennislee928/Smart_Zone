//! JS-Heavy Detection Module
//!
//! Determines if a page requires browser rendering (Selenium/Playwright)
//! based on HTML content analysis and extraction results.

use crate::types::Lead;
use scraper::{Html, Selector};
use regex::Regex;

/// Browser detection result
#[derive(Debug, Clone)]
pub struct BrowserDetectionResult {
    pub needs_browser: bool,
    pub reason: BrowserReason,
    pub confidence: f32,
    pub detected_api_endpoints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserReason {
    ContentTooShort,
    SpaDetected,
    EnableJavascriptMessage,
    ExtractionFailedWithApi,
    ManualOverride,
    None,
}

impl Default for BrowserDetectionResult {
    fn default() -> Self {
        Self {
            needs_browser: false,
            reason: BrowserReason::None,
            confidence: 0.0,
            detected_api_endpoints: vec![],
        }
    }
}

/// Check if a page needs browser rendering
/// 
/// Returns true if any of these conditions are met:
/// 1. HTML content too short (< 5KB body or < 500 chars text)
/// 2. SPA framework detected (Next.js, Nuxt, React, Angular) + no extracted fields
/// 3. "Enable JavaScript" message detected
/// 4. Extraction failed but API endpoints detected in HTML
pub fn needs_browser(html: &str, url: &str, extraction_result: Option<&Lead>) -> BrowserDetectionResult {
    // Rule 1: HTML content too short
    if let Some(result) = check_content_too_short(html) {
        return result;
    }
    
    // Rule 2: SPA framework detected
    if let Some(result) = check_spa_detected(html, extraction_result) {
        return result;
    }
    
    // Rule 3: "Enable JavaScript" message
    if let Some(result) = check_enable_javascript(html) {
        return result;
    }
    
    // Rule 4: Extraction failed but API endpoints detected
    if let Some(result) = check_extraction_failed_with_api(html, extraction_result) {
        return result;
    }
    
    BrowserDetectionResult::default()
}

/// Rule 1: Check if HTML content is too short
fn check_content_too_short(html: &str) -> Option<BrowserDetectionResult> {
    let document = Html::parse_document(html);
    
    // Extract body content (excluding whitespace and comments)
    let body_text = if let Ok(body_sel) = Selector::parse("body") {
        document.select(&body_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
    } else {
        String::new()
    };
    
    // Remove whitespace and calculate actual content size
    let body_size = body_text.len();
    let text_content: String = body_text.chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let text_length = text_content.len();
    
    // Check: body < 5KB or text < 500 chars
    if body_size < 5120 || text_length < 500 {
        return Some(BrowserDetectionResult {
            needs_browser: true,
            reason: BrowserReason::ContentTooShort,
            confidence: if body_size < 2048 { 0.9 } else { 0.7 },
            detected_api_endpoints: vec![],
        });
    }
    
    None
}

/// Rule 2: Check for SPA framework indicators
fn check_spa_detected(html: &str, extraction_result: Option<&Lead>) -> Option<BrowserDetectionResult> {
    let html_lower = html.to_lowercase();
    
    // SPA framework indicators
    let spa_indicators = vec![
        ("__next_data__", "Next.js"),
        ("window.__nuxt__", "Nuxt.js"),
        ("data-reactroot", "React SSR"),
        ("app-root", "Angular"),
        ("app-root-element", "Angular"),
    ];
    
    let mut detected_frameworks = Vec::new();
    for (indicator, framework) in &spa_indicators {
        if html_lower.contains(indicator) {
            detected_frameworks.push(framework.to_string());
        }
    }
    
    // Check for empty root div (common SPA pattern)
    if let Ok(root_sel) = Selector::parse("#root, [id='root'], [id='app'], [id='app-root']") {
        let document = Html::parse_document(html);
        for element in document.select(&root_sel) {
            let text = element.text().collect::<String>();
            if text.trim().is_empty() || text.trim().len() < 50 {
                detected_frameworks.push("Empty root div".to_string());
                break;
            }
        }
    }
    
    // If SPA detected, check if we have extracted fields
    if !detected_frameworks.is_empty() {
        let has_extracted_fields = extraction_result.map(|lead| {
            !lead.name.is_empty() && 
            (!lead.amount.is_empty() || !lead.deadline.is_empty())
        }).unwrap_or(false);
        
        if !has_extracted_fields {
            return Some(BrowserDetectionResult {
                needs_browser: true,
                reason: BrowserReason::SpaDetected,
                confidence: 0.85,
                detected_api_endpoints: detect_api_endpoints_in_html(html),
            });
        }
    }
    
    None
}

/// Rule 3: Check for "Enable JavaScript" messages
fn check_enable_javascript(html: &str) -> Option<BrowserDetectionResult> {
    let html_lower = html.to_lowercase();
    
    let javascript_messages = vec![
        "enable javascript",
        "please enable javascript",
        "javascript is disabled",
        "you need to enable javascript",
        "javascript required",
    ];
    
    for msg in &javascript_messages {
        if html_lower.contains(msg) {
            return Some(BrowserDetectionResult {
                needs_browser: true,
                reason: BrowserReason::EnableJavascriptMessage,
                confidence: 0.95,
                detected_api_endpoints: vec![],
            });
        }
    }
    
    // Check for large noscript content
    if let Ok(noscript_sel) = Selector::parse("noscript") {
        let document = Html::parse_document(html);
        for element in document.select(&noscript_sel) {
            let noscript_text = element.text().collect::<String>();
            if noscript_text.len() > 200 {
                return Some(BrowserDetectionResult {
                    needs_browser: true,
                    reason: BrowserReason::EnableJavascriptMessage,
                    confidence: 0.8,
                    detected_api_endpoints: vec![],
                });
            }
        }
    }
    
    None
}

/// Rule 4: Extraction failed but API endpoints detected
fn check_extraction_failed_with_api(html: &str, extraction_result: Option<&Lead>) -> Option<BrowserDetectionResult> {
    // Check if extraction failed or insufficient
    let extraction_failed = extraction_result.map(|lead| {
        lead.name.is_empty() || 
        (lead.amount.is_empty() || lead.amount == "See website") &&
        (lead.deadline.is_empty() || lead.deadline == "Check website")
    }).unwrap_or(true);
    
    if !extraction_failed {
        return None;
    }
    
    // Detect API endpoints in HTML
    let api_endpoints = detect_api_endpoints_in_html(html);
    
    if !api_endpoints.is_empty() {
        return Some(BrowserDetectionResult {
            needs_browser: true,
            reason: BrowserReason::ExtractionFailedWithApi,
            confidence: 0.75,
            detected_api_endpoints: api_endpoints,
        });
    }
    
    None
}

/// Detect API endpoints in HTML content
fn detect_api_endpoints_in_html(html: &str) -> Vec<String> {
    let mut endpoints = Vec::new();
    let html_lower = html.to_lowercase();
    
    // Pattern 1: API path patterns
    let api_patterns = vec![
        r#"/api/[^"'\s]+"#,
        r#"/graphql[^"'\s]*"#,
        r#"/rest/[^"'\s]+"#,
        r#"/v\d+/[^"'\s]+"#,
    ];
    
    for pattern in &api_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(&html_lower) {
                if let Some(matched) = cap.get(0) {
                    let endpoint = matched.as_str().to_string();
                    if !endpoints.contains(&endpoint) {
                        endpoints.push(endpoint);
                    }
                }
            }
        }
    }
    
    // Pattern 2: JavaScript fetch/axios calls
    let js_patterns = vec![
        r#"fetch\(['"]([^'"]+)['"]"#,
        r#"axios\.get\(['"]([^'"]+)['"]"#,
        r#"\$\.ajax\([^,]*url:\s*['"]([^'"]+)['"]"#,
    ];
    
    for pattern in &js_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(html) {
                if let Some(matched) = cap.get(1) {
                    let endpoint = matched.as_str().to_string();
                    if endpoint.contains("/api/") || endpoint.contains("/graphql") {
                        if !endpoints.contains(&endpoint) {
                            endpoints.push(endpoint);
                        }
                    }
                }
            }
        }
    }
    
    // Pattern 3: JSON structures that look like API responses
    if html_lower.contains(r#"{"scholarships":"#) || 
       html_lower.contains(r#"{"scholarship":"#) ||
       html_lower.contains(r#""scholarships":["#) {
        // Try to extract base URL and construct API endpoint
        if let Some(base_url) = extract_base_url_from_html(html) {
            let possible_endpoints = vec![
                format!("{}/api/scholarships", base_url),
                format!("{}/api/scholarship", base_url),
                format!("{}/graphql", base_url),
            ];
            endpoints.extend(possible_endpoints);
        }
    }
    
    endpoints
}

/// Extract base URL from HTML (for constructing API endpoints)
fn extract_base_url_from_html(html: &str) -> Option<String> {
    // Look for base tag
    if let Ok(base_sel) = Selector::parse("base[href]") {
        let document = Html::parse_document(html);
        if let Some(base) = document.select(&base_sel).next() {
            if let Some(href) = base.value().attr("href") {
                return Some(href.to_string());
            }
        }
    }
    
    // Look for canonical URL
    if let Ok(canonical_sel) = Selector::parse("link[rel='canonical']") {
        let document = Html::parse_document(html);
        if let Some(link) = document.select(&canonical_sel).next() {
            if let Some(href) = link.value().attr("href") {
                // Extract base URL (scheme + host)
                if let Some(pos) = href.find("://") {
                    let rest = &href[pos + 3..];
                    if let Some(path_pos) = rest.find('/') {
                        return Some(format!("{}://{}", &href[..pos + 3], &rest[..path_pos]));
                    }
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_too_short() {
        let short_html = "<html><body><div>Short</div></body></html>";
        let result = check_content_too_short(short_html);
        assert!(result.is_some());
        assert!(result.unwrap().needs_browser);
    }
    
    #[test]
    fn test_spa_detected() {
        let nextjs_html = r#"
            <html>
            <body>
                <script>window.__NEXT_DATA__ = {}</script>
                <div id="root"></div>
            </body>
            </html>
        "#;
        let result = check_spa_detected(nextjs_html, None);
        assert!(result.is_some());
        assert_eq!(result.unwrap().reason, BrowserReason::SpaDetected);
    }
    
    #[test]
    fn test_enable_javascript() {
        let js_msg_html = "<html><body>Please enable JavaScript to view this page</body></html>";
        let result = check_enable_javascript(js_msg_html);
        assert!(result.is_some());
        assert_eq!(result.unwrap().reason, BrowserReason::EnableJavascriptMessage);
    }
    
    #[test]
    fn test_api_endpoint_detection() {
        let html_with_api = r#"
            <script>
                fetch('/api/scholarships/123');
                axios.get('/api/scholarship/456');
            </script>
        "#;
        let endpoints = detect_api_endpoints_in_html(html_with_api);
        assert!(!endpoints.is_empty());
        assert!(endpoints.iter().any(|e| e.contains("/api/")));
    }
}

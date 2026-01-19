use crate::types::Lead;
use anyhow::{Result, Context};
use thirtyfour::prelude::*;
use std::time::Duration;

const WEBDRIVER_URL: &str = "http://localhost:9515";
const PAGE_LOAD_TIMEOUT: u64 = 30;
const IMPLICIT_WAIT_TIMEOUT: u64 = 10;
const JS_RENDER_WAIT: u64 = 2;

pub async fn scrape_with_selenium(url: &str) -> Result<Vec<Lead>> {
    println!("Scraping with Selenium: {}", url);
    
    // 設定 Chrome 選項
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_option(
        "args",
        vec![
            "--headless=new",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--disable-gpu",
            "--window-size=1920,1080",
            "--disable-blink-features=AutomationControlled",
        ],
    )?;
    
    // 連接 WebDriver
    let driver = WebDriver::new(WEBDRIVER_URL, caps)
        .await
        .context("Failed to connect to ChromeDriver")?;
    
    // 確保瀏覽器在錯誤時也會關閉
    let result = async {
        // 設定超時（thirtyfour 0.31 使用 set_timeouts 方法）
        // 注意：可能需要使用不同的 API，這裡先簡化為不設定超時
        // 依賴隱式等待和固定延遲來處理頁面載入
        
        // 導航到目標頁面
        driver
            .goto(url)
            .await
            .context("Failed to navigate to URL")?;
        
        // 等待頁面載入（使用 driver 的等待方法）
        driver
            .query(By::Tag("body"))
            .first()
            .await
            .context("Failed to find body element")?;
        
        // 等待 JavaScript 渲染完成
        tokio::time::sleep(Duration::from_secs(JS_RENDER_WAIT)).await;
        
        // 嘗試等待常見的獎學金列表元素（可選）
        // 如果頁面有特定的載入指示器，可以在這裡等待它消失
        
        // 獲取頁面 HTML
        let page_source = driver
            .source()
            .await
            .context("Failed to get page source")?;
        
        // 使用現有的 HTML 解析邏輯
        let leads = parse_dynamic_html(&page_source, url);
        
        Ok(leads)
    }
    .await;
    
    // 確保瀏覽器關閉
    if let Err(e) = driver.quit().await {
        eprintln!("Warning: Failed to quit browser: {}", e);
    }
    
    result
}

fn parse_dynamic_html(html: &str, base_url: &str) -> Vec<Lead> {
    use scraper::{Html, Selector};
    
    let document = Html::parse_document(html);
    let mut leads = Vec::new();
    
    // 使用與現有爬蟲相同的解析邏輯
    // 嘗試常見的獎學金列表選擇器
    let selectors = [
        "article.scholarship",
        ".scholarship-item",
        ".funding-item",
        "div[class*='scholarship']",
        "li[class*='scholarship']",
        ".result-item",
        ".phd-result",
        "article",
        "[data-scholarship]",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if text.len() > 20 {
                    // 提取標題
                    let title = extract_title(&element).unwrap_or_else(|| {
                        text.chars().take(100).collect::<String>()
                    });
                    
                    // 提取金額
                    let amount = extract_amount(&text).unwrap_or_else(|| "See website".to_string());
                    
                    // 提取截止日期
                    let deadline = extract_deadline(&text).unwrap_or_else(|| "Check website".to_string());
                    
                    // 提取 URL（如果有連結）
                    let url = extract_url(&element, base_url);
                    
                    leads.push(Lead {
                        name: title,
                        amount,
                        deadline,
                        source: base_url.to_string(),
                        source_type: "third_party".to_string(),
                        status: "new".to_string(),
                        eligibility: vec!["International students".to_string()],
                        notes: String::new(),
                        added_date: String::new(),
                        url,
                        match_score: 0,
                        match_reasons: vec![],
                        bucket: None,
                        http_status: None,
                        effort_score: None,
                        trust_tier: Some("B".to_string()),
                        risk_flags: vec![],
                        matched_rule_ids: vec![],
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
    use scraper::Selector;
    
    // 嘗試從 h1, h2, h3 提取
    for tag in &["h1", "h2", "h3", "h4"] {
        if let Ok(selector) = Selector::parse(tag) {
            if let Some(title_elem) = element.select(&selector).next() {
                let title = title_elem.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    return Some(title);
                }
            }
        }
    }
    
    // 嘗試從 strong 或 a 標籤提取
    for tag in &["strong", "a"] {
        if let Ok(selector) = Selector::parse(tag) {
            if let Some(title_elem) = element.select(&selector).next() {
                let title = title_elem.text().collect::<String>().trim().to_string();
                if !title.is_empty() && title.len() > 5 {
                    return Some(title);
                }
            }
        }
    }
    
    None
}

fn extract_amount(text: &str) -> Option<String> {
    use regex::Regex;
    
    // 匹配常見的金額格式
    let patterns = [
        r"£\s*[\d,]+",
        r"\$\s*[\d,]+",
        r"€\s*[\d,]+",
        r"[\d,]+\s*(?:GBP|USD|EUR)",
        r"full\s+tuition",
        r"fully\s+funded",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.find(text) {
                return Some(cap.as_str().to_string());
            }
        }
    }
    
    None
}

fn extract_deadline(text: &str) -> Option<String> {
    use regex::Regex;
    use chrono::NaiveDate;
    
    // 匹配日期格式
    let patterns = [
        r"\d{1,2}[/-]\d{1,2}[/-]\d{2,4}",
        r"\d{4}[/-]\d{1,2}[/-]\d{1,2}",
        r"(?:deadline|closes?|due)\s*:?\s*(\d{1,2}\s+\w+\s+\d{4})",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(text) {
                let date_str = cap.get(1).map(|m| m.as_str()).unwrap_or(cap.get(0).unwrap().as_str());
                // 嘗試解析日期以驗證格式
                if NaiveDate::parse_from_str(date_str, "%d/%m/%Y").is_ok()
                    || NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
                {
                    return Some(date_str.to_string());
                }
            }
        }
    }
    
    None
}

fn extract_url(element: &scraper::ElementRef, base_url: &str) -> String {
    use scraper::Selector;
    
    // 嘗試從 a 標籤提取 href
    if let Ok(selector) = Selector::parse("a[href]") {
        if let Some(link) = element.select(&selector).next() {
            if let Some(href) = link.value().attr("href") {
                // 處理絕對 URL
                if href.starts_with("http://") || href.starts_with("https://") {
                    return href.to_string();
                }
                // 處理相對 URL（簡化版本）
                else if href.starts_with('/') {
                    // 從 base_url 提取協議和域名
                    if let Some(scheme_end) = base_url.find("://") {
                        if let Some(host_start) = base_url[scheme_end + 3..].find('/') {
                            let host = &base_url[..scheme_end + 3 + host_start];
                            return format!("{}{}", host, href);
                        }
                    }
                }
                // 處理相對路徑
                else {
                    let base = base_url.trim_end_matches('/');
                    return format!("{}/{}", base, href);
                }
            }
        }
    }
    
    base_url.to_string()
}

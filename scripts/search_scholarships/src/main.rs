//! ScholarshipOps Search & Triage System
//! 
//! Complete pipeline for scholarship discovery and qualification

mod scrapers;
mod filter;
mod storage;
mod notify;
mod types;
mod sorter;
mod rules;
mod link_health;
mod triage;
mod effort;
mod source_health;
mod normalize;
mod discovery;
mod url_state;
mod extraction_fallbacks;
mod js_detector;
mod browser_queue;
mod api_discovery;

pub use types::*;

use anyhow::{Result, Context};
use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<()> {
    let root = std::env::var("ROOT").unwrap_or_else(|_| ".".to_string());
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    
    println!("=== ScholarshipOps Search & Triage ===");
    println!("Timestamp: {}", now);
    println!();
    
    // ==========================================
    // Stage 0: Load Configuration
    // ==========================================
    println!("Stage 0: Loading configuration...");
    
    let criteria = storage::load_criteria(&root)?;
    let sources = storage::load_sources(&root)?;
    
    // Load source filter config from environment or use defaults
    let source_filter = load_source_filter_config();
    println!("  Source filter: include={:?}, exclude={:?}, max_failures={}",
        source_filter.include_types,
        source_filter.exclude_types,
        source_filter.max_consecutive_failures
    );
    
    // Load source health tracking
    let mut health_file = source_health::load_health(&root)?;
    println!("  Source health records: {}", health_file.sources.len());
    
    // Filter enabled sources based on type and health
    let enabled_sources: Vec<_> = sources.sources.iter()
        .filter(|s| s.enabled)
        .collect();
    
    // Load rules (optional - continue without if missing)
    let rules_config = match rules::load_rules(&root) {
        Ok(r) => {
            println!("  Loaded {} hard reject, {} soft downgrade, {} positive rules",
                r.hard_reject_rules.len(),
                r.soft_downgrade_rules.len(),
                r.positive_scoring_rules.len()
            );
            Some(r)
        }
        Err(e) => {
            println!("  Warning: Could not load rules.yaml: {}", e);
            None
        }
    };
    
    // Load existing leads
    let mut leads_file = storage::load_leads(&root)?;
    println!("  Existing leads in database: {}", leads_file.leads.len());
    
    // Use name + url as unique key to prevent duplicates
    let existing_keys: HashSet<String> = leads_file.leads.iter()
        .map(|l| format!("{}|{}", l.name.to_lowercase().trim(), l.url.to_lowercase().trim()))
        .collect();
    
    // Track seen leads in this run
    let mut seen_keys: HashSet<String> = HashSet::new();
    
    println!();
    
    // ==========================================
    // Stage 0.5: Discovery Stage (重構)
    // ==========================================
    println!("Stage 0.5: Discovering URLs from discovery_seed sources...");
    let discovery_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to create HTTP client for discovery")?;
    
    // 分離 sources 為三種類型
    let detail_sources: Vec<_> = enabled_sources.iter()
        .filter(|s| s.mode.as_deref() == Some("detail") || s.mode.is_none())
        .collect();
        
    let index_sources: Vec<_> = enabled_sources.iter()
        .filter(|s| s.mode.as_deref() == Some("index"))
        .collect();
        
    let discovery_seeds: Vec<_> = enabled_sources.iter()
        .filter(|s| s.mode.as_deref() == Some("discovery_seed"))
        .collect();
    
    println!("  Source types: {} detail, {} index, {} discovery_seed", 
        detail_sources.len(), index_sources.len(), discovery_seeds.len());
    
    // 處理 discovery_seed sources
    let mut discovered_candidates: Vec<discovery::CandidateUrl> = Vec::new();
    for seed in &discovery_seeds {
        println!("  Processing discovery seed: {}", seed.name);
        match discovery::discover_from_seed(&discovery_client, seed).await {
            Ok(candidates) => {
                println!("    Found {} candidates", candidates.len());
                discovered_candidates.extend(candidates);
            }
            Err(e) => {
                println!("    Error: {}", e);
            }
        }
    }
    
    // 保存 candidates
    if !discovered_candidates.is_empty() {
        if let Err(e) = storage::save_candidates(&root, &discovered_candidates) {
            println!("  Warning: Failed to save candidates: {}", e);
        } else {
            println!("  Saved {} candidates to candidate_urls.jsonl", discovered_candidates.len());
        }
    }
    
    // ==========================================
    // Stage 0.6: Candidate Normalization + Heavy Validation
    // ==========================================
    println!("Stage 0.6: Normalizing and validating candidates...");
    
    let discovered_count = discovered_candidates.len();
    let mut validated_candidates: Vec<discovery::CandidateUrl> = Vec::new();
    let validation_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create validation client")?;
    
    for mut candidate in discovered_candidates {
        // URL 正規化
        candidate.url = normalize::canonicalize_candidate_url(&candidate.url);
        
        // Confidence 過濾（初步）
        if candidate.confidence < 0.6 {
            continue;
        }
        
        // 重量級驗證
        match discovery::validate_candidate_heavy(&validation_client, &mut candidate).await {
            Ok(is_valid) => {
                if is_valid {
                    validated_candidates.push(candidate);
                } else {
                    println!("  Rejected: {} (reason: {})", candidate.url, candidate.reason);
                }
            }
            Err(e) => {
                println!("  Validation error for {}: {}", candidate.url, e);
                // 驗證失敗的 candidates 仍保留，但標記為需要手動檢查
                candidate.tags.push("validation_failed".to_string());
                validated_candidates.push(candidate);
            }
        }
    }
    
    // 保存驗證後的 candidates
    if !validated_candidates.is_empty() {
        if let Err(e) = storage::save_candidates(&root, &validated_candidates) {
            println!("  Warning: Failed to save validated candidates: {}", e);
        } else {
            println!("  Validated {} candidates (from {} discovered)", 
                validated_candidates.len(), discovered_count);
        }
    }
    
    // 保存驗證後的 candidates
    if !validated_candidates.is_empty() {
        if let Err(e) = storage::save_candidates(&root, &validated_candidates) {
            println!("  Warning: Failed to save validated candidates: {}", e);
        } else {
            println!("  Validated {} candidates (from {} discovered)", 
                validated_candidates.len(), discovered_count);
        }
    }
    
    println!();
    
    // ==========================================
    // Stage 1: Scrape Sources (with health tracking)
    // ==========================================
    let mut skipped_sources: Vec<(String, String)> = Vec::new();
    
    // 載入驗證後的 candidates
    let candidate_urls = storage::load_candidates(&root).unwrap_or_default();
    println!("  Loaded {} validated candidates from candidate_urls.jsonl", candidate_urls.len());
    
    let mut sources_to_scrape: Vec<_> = enabled_sources.iter()
        .filter(|s| {
            // 排除 discovery_seed sources（它們已經處理過了）
            if s.mode.as_deref() == Some("discovery_seed") {
                return false;
            }
            
            if let Some(reason) = source_health::should_skip_source(s, &health_file, &source_filter) {
                skipped_sources.push((s.name.clone(), reason));
                false
            } else {
                true
            }
        })
        .collect();
    
    // Sort sources by priority: priority 1 first, then by None (default)
    sources_to_scrape.sort_by(|a, b| {
        let pri_a = a.priority.unwrap_or(255);  // Default to lowest priority
        let pri_b = b.priority.unwrap_or(255);
        pri_a.cmp(&pri_b)
    });
    
    // Count priority sources for logging
    let priority_count = sources_to_scrape.iter().filter(|s| s.priority == Some(1)).count();
    
    println!("Stage 1: Scraping {} sources ({} priority, {} skipped)...", 
        sources_to_scrape.len(), priority_count, skipped_sources.len());
    
    if !skipped_sources.is_empty() {
        println!("  Skipped sources:");
        for (name, reason) in &skipped_sources {
            println!("    - {}: {}", name, reason);
        }
    }
    
    let mut all_leads: Vec<Lead> = Vec::new();
    let mut filtered_out: Vec<(String, Vec<String>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut source_stats = SourceStats::default();
    
    for source in sources_to_scrape {
        println!("  Scraping: {} ({})", source.name, source.url);
        
        // Check if source has fallback strategies
        let mut fallback_leads: Vec<Lead> = Vec::new();
        
        if let Some(health) = health_file.sources.iter().find(|h| h.url == source.url) {
            if !health.fallback_strategies.is_empty() {
                println!("    Source has fallback strategies: {:?}", health.fallback_strategies);
                
                let base_url = if let Some(pos) = source.url.find("://") {
                    let rest = &source.url[pos + 3..];
                    if let Some(path_pos) = rest.find('/') {
                        format!("{}://{}", &source.url[..pos + 3], &rest[..path_pos])
                    } else {
                        source.url.clone()
                    }
                } else {
                    source.url.clone()
                };
                
                let discovery_config = discovery::DiscoveryConfig::default();
                let discovery_client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .context("Failed to create HTTP client for fallback")?;
                
                for strategy in &health.fallback_strategies {
                    match strategy.as_str() {
                        "sitemap" => {
                            println!("      Trying sitemap fallback...");
                            let common_sitemaps = vec![
                                format!("{}/sitemap.xml", base_url),
                                format!("{}/sitemap_index.xml", base_url),
                            ];
                            
                            for sitemap_url in common_sitemaps {
                                if let Ok(sitemap_candidates) = discovery::parse_sitemap(&discovery_client, &sitemap_url, &discovery_config).await {
                                    println!("        Found {} URLs from sitemap", sitemap_candidates.len());
                                    
                                    // Filter and create leads from sitemap URLs
                                    for candidate in sitemap_candidates {
                                        let url_lower = candidate.url.to_lowercase();
                                        if url_lower.contains("scholarship") || 
                                           url_lower.contains("funding") || 
                                           url_lower.contains("bursary") || 
                                           url_lower.contains("award") {
                                            // Create discovery lead
                                            fallback_leads.push(Lead {
                                                name: format!("Discovered from sitemap: {}", candidate.url),
                                                amount: String::new(),
                                                deadline: String::new(),
                                                source: source.url.clone(),
                                                source_type: source.source_type.clone(),
                                                status: "discovered".to_string(),
                                                eligibility: vec![],
                                                notes: format!("Discovered via sitemap fallback from {}", source.name),
                                                added_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                                                url: candidate.url.clone(),
                                                match_score: 0,
                                                match_reasons: vec![],
                                                hard_fail_reasons: vec![],
                                                soft_flags: vec![],
                                                bucket: None,
                                                http_status: None,
                                                effort_score: None,
                                                trust_tier: Some("B".to_string()),
                                                risk_flags: vec!["sitemap_fallback".to_string()],
                                                matched_rule_ids: vec![],
                                                eligible_countries: vec![],
                                                is_taiwan_eligible: None,
                                                taiwan_eligibility_confidence: None,
                                                deadline_date: None,
                                                deadline_label: None,
                                                intake_year: None,
                                                study_start: None,
                                                deadline_confidence: None,
                                                canonical_url: None,
                                                is_directory_page: false,
                                                official_source_url: Some(candidate.url.clone()),
                                                source_domain: None,
                                                confidence: Some(0.7),
                                                eligibility_confidence: None,
                                                tags: vec!["sitemap_discovery".to_string(), "needs_scraping".to_string()],
                                                is_index_only: true,
                                                first_seen_at: Some(chrono::Utc::now().to_rfc3339()),
                                                last_checked_at: Some(chrono::Utc::now().to_rfc3339()),
                                                next_check_at: None,
                                                persistence_status: Some("new".to_string()),
                                                source_seed: Some(source.url.clone()),
                                                check_count: Some(0),
                                                extraction_evidence: vec![],
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        "rss" => {
                            println!("      Trying RSS fallback...");
                            if let Ok(feed_urls) = discovery::discover_feeds_public(&discovery_client, &base_url).await {
                                println!("        Found {} RSS feeds", feed_urls.len());
                                // RSS feed parsing would be implemented here
                                // For now, we'll rely on sitemap fallback
                            }
                        }
                        "head_request" => {
                            println!("      Trying HEAD request fallback...");
                            // Try HEAD request first, then GET if successful
                            if let Ok(resp) = discovery_client.head(&source.url).send().await {
                                if resp.status().is_success() {
                                    // HEAD succeeded, try GET with longer timeout
                                    let get_client = reqwest::Client::builder()
                                        .timeout(std::time::Duration::from_secs(30))
                                        .build()
                                        .context("Failed to create HTTP client for HEAD->GET")?;
                                    
                                    if let Ok(get_resp) = get_client.get(&source.url).send().await {
                                        if get_resp.status().is_success() {
                                            println!("        HEAD->GET succeeded, will retry scrape...");
                                            // The normal scrape will proceed below
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        match scrapers::scrape_source(source).await {
            Ok(result) => {
                // Update health tracking
                source_health::update_health(
                    &mut health_file, 
                    source, 
                    &result, 
                    source_filter.max_consecutive_failures
                );
                
                // Track stats
                if result.status == SourceStatus::Ok || !fallback_leads.is_empty() {
                    if result.status == SourceStatus::Ok {
                        source_stats.success += 1;
                        println!("    Found {} raw leads", result.leads.len());
                    }
                } else {
                    source_stats.failed += 1;
                    println!("    Failed: {}", result.status);
                    if let Some(ref err) = result.error_message {
                        errors.push(format!("{}: {}", source.name, err));
                    }
                    continue; // Skip processing leads from failed sources
                }
                
                let mut added = 0;
                let mut skipped_directory = 0;
                let mut skipped_insufficient = 0;
                
                // Process both normal leads and fallback leads
                let mut all_result_leads = result.leads;
                if !fallback_leads.is_empty() {
                    println!("    Adding {} leads from fallback strategies", fallback_leads.len());
                    all_result_leads.extend(fallback_leads);
                }
                
                for mut lead in all_result_leads {
                    // Update deduplication info first
                    filter::update_dedup_info(&mut lead);
                    
                    // Skip directory/landing pages
                    if lead.is_directory_page {
                        skipped_directory += 1;
                        continue;
                    }
                    
                    // Skip leads without sufficient detail
                    if !filter::has_sufficient_detail(&lead) {
                        skipped_insufficient += 1;
                        continue;
                    }
                    
                    // Funding intent gate: check if lead has funding-related keywords
                    if !filter::has_funding_intent(&lead) {
                        filtered_out.push((lead.name.clone(), vec!["No funding intent".to_string()]));
                        continue;
                    }
                    
                    // Create unique key using improved dedup logic
                    let key = filter::generate_dedup_key(&lead);
                    
                    // Skip duplicates
                    if existing_keys.contains(&key) || seen_keys.contains(&key) {
                        continue;
                    }
                    seen_keys.insert(key);
                    
                    // Basic keyword filtering
                    if !filter::matches_criteria(&lead, &criteria) {
                        filtered_out.push((lead.name.clone(), vec!["Keyword mismatch".to_string()]));
                        continue;
                    }
                    
                    // Update country eligibility, structured dates, and trust info before profile filtering
                    filter::update_country_eligibility(&mut lead);
                    filter::update_structured_dates(&mut lead);
                    filter::update_trust_info(&mut lead);
                    
                    // Validate deadline format - clear invalid formats like "68-58-58"
                    if !lead.deadline.is_empty() && 
                       lead.deadline != "Check website" && 
                       lead.deadline != "See website" &&
                       lead.deadline != "See official page" &&
                       !lead.deadline.to_lowercase().contains("tbd") {
                        if filter::parse_deadline(&lead.deadline).is_err() {
                            // Invalid deadline format - clear it
                            lead.deadline = String::new();
                            lead.deadline_date = None;
                            lead.deadline_confidence = Some("invalid_format".to_string());
                        }
                    }
                    
                    // Handle unknown eligibility - lower trust for untrusted sources
                    filter::handle_unknown_eligibility(&mut lead);
                    
                    // Profile-based filtering
                    if let Some(ref profile) = criteria.profile {
                        if filter::filter_by_profile(&mut lead, profile) {
                            lead.status = "qualified".to_string();
                            lead.source_type = source.source_type.clone();
                            lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                            all_leads.push(lead);
                            added += 1;
                        } else {
                            filtered_out.push((lead.name.clone(), lead.match_reasons.clone()));
                        }
                    } else {
                        lead.status = "qualified".to_string();
                        lead.source_type = source.source_type.clone();
                        lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                        all_leads.push(lead);
                        added += 1;
                    }
                }
                if skipped_directory > 0 || skipped_insufficient > 0 {
                    println!("    Skipped: {} directory pages, {} insufficient detail", 
                        skipped_directory, skipped_insufficient);
                }
                println!("    Added {} qualified leads", added);
            }
            Err(e) => {
                let err_msg = format!("Failed to scrape {}: {}", source.name, e);
                println!("    Error: {}", err_msg);
                errors.push(err_msg);
                source_stats.failed += 1;
            }
        }
    }
    
    source_stats.skipped = skipped_sources.len();
    println!("  Source stats: {} success, {} failed, {} skipped", 
        source_stats.success, source_stats.failed, source_stats.skipped);
    println!("  Total qualified leads: {}", all_leads.len());
    println!();
    
    // ==========================================
    // Stage 1.1: Process Candidate URLs from Discovery Seeds
    // ==========================================
    if !candidate_urls.is_empty() {
        println!("Stage 1.1: Processing {} candidate URLs from discovery seeds...", candidate_urls.len());
        
        let mut candidate_leads_added = 0;
        for candidate in candidate_urls {
            // 只處理 confidence >= 0.6 的 candidates
            if candidate.confidence < 0.6 {
                continue;
            }
            
            // 建立臨時 Lead 物件（僅 URL，其他欄位待提取）
            let mut lead = Lead {
                name: format!("Discovered: {}", candidate.url),
                amount: String::new(),
                deadline: String::new(),
                source: candidate.source_seed.clone(),
                source_type: "discovered".to_string(),
                status: "discovered".to_string(),
                eligibility: vec![],
                notes: format!("Discovered from {} via {}", candidate.source_seed, candidate.discovered_from),
                added_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                url: candidate.url.clone(),
                match_score: 0,
                match_reasons: vec![],
                hard_fail_reasons: vec![],
                soft_flags: vec![],
                bucket: None,
                http_status: None,
                effort_score: None,
                trust_tier: Some("B".to_string()),
                risk_flags: vec!["discovered_candidate".to_string()],
                matched_rule_ids: vec![],
                eligible_countries: vec![],
                is_taiwan_eligible: None,
                taiwan_eligibility_confidence: None,
                deadline_date: None,
                deadline_label: None,
                intake_year: None,
                study_start: None,
                deadline_confidence: None,
                canonical_url: Some(normalize::normalize_url(&candidate.url)),
                is_directory_page: false,
                official_source_url: Some(candidate.url.clone()),
                source_domain: None,
                confidence: Some(candidate.confidence),
                eligibility_confidence: None,
                tags: candidate.tags.clone(),
                is_index_only: true,
                first_seen_at: Some(candidate.discovered_at.clone()),
                last_checked_at: Some(chrono::Utc::now().to_rfc3339()),
                next_check_at: None,
                persistence_status: Some("new".to_string()),
                source_seed: Some(candidate.source_seed.clone()),
                check_count: Some(0),
                extraction_evidence: vec![],
            };
            
            // 進入 detail extraction（使用現有 scrapers）
            // 根據 URL domain 決定使用哪個 scraper
            let url_lower = candidate.url.to_lowercase();
            let scraper_type = if url_lower.contains(".ac.uk") || url_lower.contains(".edu") {
                "university"
            } else if url_lower.contains(".gov.uk") || url_lower.contains(".gov") {
                "government"
            } else {
                "third_party"
            };
            
            // 建立臨時 Source 物件
            let temp_source = Source {
                name: format!("Discovered: {}", candidate.url),
                source_type: scraper_type.to_string(),
                url: candidate.url.clone(),
                enabled: true,
                scraper: scraper_type.to_string(),
                priority: Some(3),
                discovery_mode: None,
                allow_domains_outbound: None,
                mode: Some("detail".to_string()),
                max_depth: None,
                deny_patterns: None,
            };
            
            // 使用現有 scraper 提取詳細資訊
            match scrapers::scrape_source(&temp_source).await {
                Ok(result) => {
                    if !result.leads.is_empty() {
                        // 使用提取到的第一個 lead（應該只有一個）
                        let mut extracted_lead = result.leads[0].clone();
                        // 保留 discovery 相關的 metadata
                        extracted_lead.source = candidate.source_seed.clone();
                        extracted_lead.source_type = "discovered".to_string();
                        extracted_lead.source_seed = Some(candidate.source_seed.clone());
                        extracted_lead.tags.extend(candidate.tags.clone());
                        extracted_lead.official_source_url = Some(candidate.url.clone());
                        
                        // 進入正常的 lead 處理流程
                        filter::update_dedup_info(&mut extracted_lead);
                        
                        if !extracted_lead.is_directory_page &&
                           filter::has_sufficient_detail(&extracted_lead) &&
                           filter::has_funding_intent(&extracted_lead) {
                            let key = filter::generate_dedup_key(&extracted_lead);
                            if !existing_keys.contains(&key) && !seen_keys.contains(&key) {
                                seen_keys.insert(key);
                                
                                if filter::matches_criteria(&extracted_lead, &criteria) {
                                    filter::update_country_eligibility(&mut extracted_lead);
                                    filter::update_structured_dates(&mut extracted_lead);
                                    filter::update_trust_info(&mut extracted_lead);
                                    
                                    if let Some(ref profile) = criteria.profile {
                                        if filter::filter_by_profile(&mut extracted_lead, profile) {
                                            extracted_lead.status = "qualified".to_string();
                                            extracted_lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                            all_leads.push(extracted_lead);
                                            candidate_leads_added += 1;
                                        }
                                    } else {
                                        extracted_lead.status = "qualified".to_string();
                                        extracted_lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                        all_leads.push(extracted_lead);
                                        candidate_leads_added += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("    Failed to scrape candidate {}: {}", candidate.url, e);
                }
            }
        }
        
        println!("  Added {} leads from candidate URLs", candidate_leads_added);
        println!();
    }
    
    // ==========================================
    // Stage 1.5: Bulk Extraction Detection & URL Normalization/Deduplication
    // ==========================================
    println!("Stage 1.5: Detecting bulk extractions and deduplicating...");
    let before_count = all_leads.len();
    
    // Mark leads from URLs that produced too many extractions
    filter::mark_bulk_extracted_leads(&mut all_leads);
    
    // Filter out newly marked directory pages
    let bulk_extracted_count = all_leads.iter().filter(|l| l.is_directory_page).count();
    all_leads.retain(|l| !l.is_directory_page);
    
    // Apply URL normalization and deduplication
    let (deduplicated_leads, dedup_stats) = normalize::deduplicate_leads_with_stats(all_leads);
    all_leads = deduplicated_leads;
    let dedup_removed = dedup_stats.duplicates_removed;
    
    println!("  Marked {} leads as bulk extractions from directory pages", bulk_extracted_count);
    println!("  Entity-level deduplication removed {} duplicates ({} unique keys with duplicates)", 
        dedup_removed, dedup_stats.dup_count_by_key.len());
    println!("  Leads after dedup: {} (removed {})", all_leads.len(), before_count - all_leads.len());
    println!();
    
    // Store dedup stats for report generation
    let dedup_stats_for_report = dedup_stats;
    
    // ==========================================
    // Stage 1.55: JS-Heavy Detection & Browser Queue
    // ==========================================
    println!("Stage 1.55: Detecting JS-heavy pages and queuing for browser extraction...");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to create HTTP client")?;
    
    let mut browser_queue_count = 0;
    let mut browser_detection_reasons: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    
    for lead in &mut all_leads {
        // Skip if already marked for browser or if URL is invalid
        if lead.tags.contains(&"pending_browser".to_string()) || lead.url.is_empty() {
            continue;
        }
        
        // Check if lead has complete data - if yes, skip browser queue
        let has_complete_data = !lead.name.is_empty() && 
            (!lead.amount.is_empty() && lead.amount != "See website") &&
            (!lead.deadline.is_empty() && lead.deadline != "Check website" && lead.deadline != "TBD");
        
        if has_complete_data {
            // Lead has complete data from HTTP scraping, skip browser queue
            continue;
        }
        
        // Fetch HTML for JS-heavy detection
        let html_content = match client.get(&lead.url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    resp.text().await.unwrap_or_default()
                } else {
                    // HTTP request failed - mark for browser
                    let detection = js_detector::BrowserDetectionResult {
                        needs_browser: true,
                        reason: js_detector::BrowserReason::ExtractionFailedWithApi,
                        confidence: 0.8,
                        detected_api_endpoints: vec![],
                    };
                    if let Err(e) = browser_queue::write_to_browser_queue(&root, lead, &detection) {
                        eprintln!("  Warning: Failed to write {} to browser queue: {}", lead.url, e);
                    } else {
                        lead.tags.push("pending_browser".to_string());
                        browser_queue_count += 1;
                    }
                    continue;
                }
            }
            Err(_) => {
                // Network error - mark for browser
                let detection = js_detector::BrowserDetectionResult {
                    needs_browser: true,
                    reason: js_detector::BrowserReason::ExtractionFailedWithApi,
                    confidence: 0.7,
                    detected_api_endpoints: vec![],
                };
                if let Err(e) = browser_queue::write_to_browser_queue(&root, lead, &detection) {
                    eprintln!("  Warning: Failed to write {} to browser queue: {}", lead.url, e);
                } else {
                    lead.tags.push("pending_browser".to_string());
                    browser_queue_count += 1;
                }
                continue;
            }
        };
        
        // Check if page needs browser rendering (only if extraction failed or incomplete)
        let detection = js_detector::needs_browser(&html_content, &lead.url, Some(lead));
        
        if detection.needs_browser {
            // Write to browser queue
            if let Err(e) = browser_queue::write_to_browser_queue(&root, lead, &detection) {
                eprintln!("  Warning: Failed to write {} to browser queue: {}", lead.url, e);
            } else {
                lead.tags.push("pending_browser".to_string());
                browser_queue_count += 1;
                
                let reason_str = format!("{:?}", detection.reason);
                *browser_detection_reasons.entry(reason_str).or_insert(0) += 1;
            }
        }
    }
    
    println!("  Queued {} URLs for browser extraction", browser_queue_count);
    for (reason, count) in &browser_detection_reasons {
        println!("    {}: {}", reason, count);
    }
    println!();
    
    // ==========================================
    // Stage 1.6: Link Validation
    // ==========================================
    println!("Stage 1.6: Validating scholarship links...");
    let invalid_links = filter::validate_all_scholarship_links(&mut all_leads);
    println!("  Found {} leads with invalid links (pointing to homepages)", invalid_links);
    println!();

    // ==========================================
    // Stage 1.62: Merge Browser Results
    // ==========================================
    println!("Stage 1.62: Merging browser extraction results...");
    match browser_queue::read_browser_results(&root) {
        Ok(browser_results) => {
            let mut merged_count = 0;
            let mut new_from_browser = 0;
            
            for result in browser_results {
                let before_count = all_leads.len();
                browser_queue::merge_browser_result(&mut all_leads, result);
                
                if all_leads.len() > before_count {
                    new_from_browser += all_leads.len() - before_count;
                } else {
                    merged_count += 1;
                }
            }
            
            println!("  Merged {} browser results ({} updated, {} new)", 
                merged_count + new_from_browser, merged_count, new_from_browser);
            
            // Register detected API endpoints
            for result in browser_queue::read_browser_results(&root)? {
                for api_endpoint in &result.detected_api_endpoints {
                    // Extract domain from URL using simple string parsing
                    if let Some(domain) = filter::extract_domain_from_url(&api_endpoint.url) {
                        if let Err(e) = api_discovery::register_api_endpoint(&root, &domain, &api_endpoint.url) {
                            eprintln!("  Warning: Failed to register API endpoint {}: {}", api_endpoint.url, e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("  No browser results found or error reading: {}", e);
        }
    }
    println!();
    
    // ==========================================
    // Stage 1.65: Enrich index-only leads (two-hop to official)
    // ==========================================
    let index_only_count = all_leads.iter().filter(|l| l.is_index_only).count();
    if index_only_count > 0 {
        println!("Stage 1.65: Enriching {} index-only leads (two-hop to official)...", index_only_count);
        let mut enriched_n = 0;
        for lead in all_leads.iter_mut() {
            if !lead.is_index_only || lead.official_source_url.is_none() {
                continue;
            }
            if scrapers::enrich_from_official(lead) {
                enriched_n += 1;
            }
        }
        let still_index = all_leads.iter().filter(|l| l.is_index_only).count();
        println!("  Enriched {} leads; {} remain index-only (needs_verification)", enriched_n, still_index);
        println!();
    }

    // ==========================================
    // Stage 1.7: Extract source domains for trust tier determination
    // ==========================================
    println!("Stage 1.7: Extracting source domains...");
    for lead in &mut all_leads {
        if lead.source_domain.is_none() {
            lead.source_domain = filter::extract_domain_from_url(&lead.url);
        }
    }
    println!("  Extracted domains for {} leads", all_leads.len());
    println!();
    let mut dead_links = Vec::new();
    if all_leads.len() <= 50 {
        println!("Stage 2: Checking link health...");
        dead_links = link_health::check_links(&mut all_leads, 5).await;
        let dead_count = dead_links.iter()
            .filter(|r| matches!(r.status, LinkHealthStatus::NotFound | LinkHealthStatus::ServerError))
            .count();
        println!("  Checked {} URLs, {} dead/error links", dead_links.len(), dead_count);
        println!();
    } else {
        println!("Stage 2: Skipping link health check ({} leads, max 50)", all_leads.len());
        println!();
    }
    
    // ==========================================
    // Stage 3: Effort Scoring
    // ==========================================
    println!("Stage 3: Calculating effort scores...");
    effort::update_effort_scores(&mut all_leads);
    println!("  Updated effort scores for {} leads", all_leads.len());
    println!();
    
    // ==========================================
    // Stage 4: Apply Rules & Triage
    // ==========================================
    let triage_stats;
    if let Some(ref rules) = rules_config {
        println!("Stage 4: Applying rules and triage...");
        triage_stats = triage::triage_leads(&mut all_leads, rules);
        println!("  Bucket A: {} | Bucket B: {} | Bucket C: {} | Bucket X: {}",
            triage_stats.bucket_a, triage_stats.bucket_b, triage_stats.bucket_c, triage_stats.bucket_x);
    } else {
        println!("Stage 4: Skipping rules (no rules.yaml)");
        // Default triage based on score
        for lead in all_leads.iter_mut() {
            lead.bucket = Some(if lead.match_score >= 100 {
                Bucket::A
            } else if lead.match_score >= 50 {
                Bucket::B
            } else {
                Bucket::C
            });
        }
        triage_stats = triage::TriageStats {
            total: all_leads.len(),
            bucket_a: all_leads.iter().filter(|l| l.bucket == Some(Bucket::A)).count(),
            bucket_b: all_leads.iter().filter(|l| l.bucket == Some(Bucket::B)).count(),
            bucket_c: all_leads.iter().filter(|l| l.bucket == Some(Bucket::C)).count(),
            bucket_x: all_leads.iter().filter(|l| l.bucket == Some(Bucket::X)).count(),
            ..Default::default()
        };
    }
    println!();
    
    // ==========================================
    // Stage 5: Sort & Finalize
    // ==========================================
    println!("Stage 5: Sorting leads...");
    sorter::sort_leads(&mut all_leads);
    
    // Split into buckets (A, B, C, X)
    let (bucket_a, bucket_b, bucket_c, bucket_x) = triage::split_by_bucket(all_leads.clone());
    let watchlist: Vec<Lead> = all_leads.iter()
        .filter(|l| l.deadline.to_lowercase().contains("check") || l.deadline.to_lowercase().contains("tbd"))
        .cloned()
        .collect();
    
    println!("  Final: A={}, B={}, C={}, X={}, Watchlist={}",
        bucket_a.len(), bucket_b.len(), bucket_c.len(), bucket_x.len(), watchlist.len());
    println!();
    
    // ==========================================
    // Stage 6: Generate Reports
    // ==========================================
    println!("Stage 6: Generating reports...");
    
    // Create output directory
    let date_str = chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let productions_dir = PathBuf::from(&root).join("scripts").join("productions");
    let report_dir = productions_dir.join(&date_str);
    fs::create_dir_all(&report_dir)?;
    
    // Generate all reports
    let triage_md = triage::generate_triage_md(&bucket_a, &bucket_b, &bucket_c, &bucket_x, &watchlist);
    let triage_csv = triage::generate_triage_csv(&bucket_a, &bucket_b, &bucket_c, &bucket_x);
    let deadlinks_md = link_health::generate_deadlinks_report(&dead_links);
    let health_report_md = source_health::generate_health_report(&health_file);
    
    let full_report = build_full_report(&now, &all_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile, &dedup_stats_for_report);
    let summary_report = build_summary_report(&now, &bucket_a, &bucket_b, &bucket_c, &filtered_out, &errors, leads_file.leads.len(), &dedup_stats_for_report);
    let markdown_report = build_markdown_report(&now, &all_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile, &dedup_stats_for_report);
    let html_report = build_html_report(&now, &all_leads, &filtered_out, &errors, leads_file.leads.len(), &criteria.profile, &dedup_stats_for_report);
    
    // Save reports
    fs::write(report_dir.join("triage.md"), &triage_md)?;
    fs::write(report_dir.join("triage.csv"), &triage_csv)?;
    fs::write(report_dir.join("deadlinks.md"), &deadlinks_md)?;
    fs::write(report_dir.join("source_health.md"), &health_report_md)?;
    fs::write(report_dir.join("report.txt"), &full_report)?;
    fs::write(report_dir.join("report.md"), &markdown_report)?;
    fs::write(report_dir.join("report.html"), &html_report)?;
    
    // Generate rules audit if rules were loaded
    if let Some(ref rules) = rules_config {
        let audit = triage::generate_rules_audit(rules, &triage_stats);
        let audit_json = serde_json::to_string_pretty(&audit)?;
        fs::write(report_dir.join("rules.audit.json"), &audit_json)?;
    }
    
    println!("  Saved reports to: {:?}", report_dir);
    
    // Save summary for Discord
    fs::write("summary.txt", &summary_report)?;
    
    // ==========================================
    // Stage 7: Update Database & Health Tracking
    // ==========================================
    println!();
    println!("Stage 7: Updating database and health tracking...");

    // Save source health tracking
    source_health::save_health(&root, &health_file)?;
    let disabled_count = health_file.sources.iter().filter(|h| h.auto_disabled).count();
    println!("  Updated source health ({} auto-disabled)", disabled_count);

    // Persist A, B, and C (C for audit). Merge by dedup key; set first_seen_at, last_checked_at, next_check_at, persistence_status, source_seed, check_count.
    let now = chrono::Utc::now();
    let now_iso = now.to_rfc3339();
    let mut by_key: std::collections::HashMap<String, Lead> = leads_file
        .leads
        .drain(..)
        .map(|l| (filter::generate_dedup_key(&l), l))
        .collect();

    for mut lead in all_leads.iter().cloned() {
        let key = filter::generate_dedup_key(&lead);
        let is_new = !by_key.contains_key(&key);
        if is_new {
            lead.first_seen_at = Some(now_iso.clone());
            lead.source_seed = Some(lead.source.clone());
            lead.check_count = Some(1);
        } else if let Some(existing) = by_key.get(&key) {
            if lead.first_seen_at.is_none() {
                lead.first_seen_at = existing.first_seen_at.clone();
            }
            if lead.source_seed.is_none() {
                lead.source_seed = existing.source_seed.clone();
            }
            lead.check_count = Some(existing.check_count.unwrap_or(0) + 1);
        }
        lead.last_checked_at = Some(now_iso.clone());
        let days = match lead.bucket {
            Some(Bucket::A) | Some(Bucket::B) => 7,
            Some(Bucket::C) => 30,
            Some(Bucket::X) | None => 30,
        };
        lead.next_check_at = Some(
            (now + chrono::Duration::days(days as i64)).to_rfc3339(),
        );
        lead.persistence_status = Some(match lead.bucket {
            Some(Bucket::A) | Some(Bucket::B) => "ok".to_string(),
            Some(Bucket::C) => "rejected".to_string(),
            Some(Bucket::X) => "ok".to_string(),
            None => "rejected".to_string(),
        });
        by_key.insert(key, lead);
    }

    leads_file.leads = by_key.into_values().collect();
    let saved_count = leads_file.leads.len();
    storage::save_leads(&root, &leads_file)?;
    println!("  Persisted {} leads (A+B+C, merge by dedup key)", saved_count);
    
    // Send notification
    println!("  Sending notification...");
    notify::send_notifications(&summary_report)?;
    
    println!();
    println!("=== Complete ===");
    
    Ok(())
}

// ==========================================
// Source Stats & Filter Config
// ==========================================

#[derive(Default)]
struct SourceStats {
    success: usize,
    failed: usize,
    skipped: usize,
}

/// Load source filter config from environment variables
fn load_source_filter_config() -> SourceFilterConfig {
    let include_types: Vec<String> = std::env::var("SOURCE_INCLUDE_TYPES")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    let exclude_types: Vec<String> = std::env::var("SOURCE_EXCLUDE_TYPES")
        .unwrap_or_else(|_| "web3".to_string()) // Default: exclude web3
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    let max_failures: u32 = std::env::var("SOURCE_MAX_FAILURES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    
    let skip_disabled: bool = std::env::var("SOURCE_SKIP_DISABLED")
        .map(|s| s != "false" && s != "0")
        .unwrap_or(true);
    
    SourceFilterConfig {
        include_types,
        exclude_types,
        max_consecutive_failures: max_failures,
        skip_auto_disabled: skip_disabled,
    }
}

// ==========================================
// Report Generation Functions
// ==========================================

fn build_full_report(
    timestamp: &str, 
    leads: &[Lead], 
    filtered_out: &[(String, Vec<String>)],
    errors: &[String], 
    total_leads: usize,
    profile: &Option<Profile>,
    dedup_stats: &normalize::DeduplicationStats,
) -> String {
    let mut report = format!("🔍 **ScholarshipOps Search Report**\n📅 {}\n\n", timestamp);
    
    if let Some(p) = profile {
        report.push_str("👤 **Your Profile:**\n");
        report.push_str(&format!("• Nationality: {}\n", p.nationality));
        report.push_str(&format!("• Target: {} ({})\n", p.target_university, p.programme_start));
        report.push_str(&format!("• Level: {}\n", p.programme_level));
        report.push_str("\n");
    }
    
    // Group by bucket
    let bucket_a: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::A)).collect();
    let bucket_b: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::B)).collect();
    let bucket_c: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::C) || l.bucket.is_none()).collect();
    
    report.push_str(&format!("📊 **Results:** A={} | B={} | C={}\n", bucket_a.len(), bucket_b.len(), bucket_c.len()));
    report.push_str(&format!("🔄 **Deduplication:** {} duplicates removed ({} unique keys with duplicates)\n\n", 
        dedup_stats.duplicates_removed, dedup_stats.dup_count_by_key.len()));
    
    if !bucket_a.is_empty() {
        report.push_str("## 🎯 Bucket A (主攻)\n\n");
        for (i, lead) in bucket_a.iter().enumerate() {
            report.push_str(&format!("{}. **{}**\n", i + 1, lead.name));
            report.push_str(&format!("   💰 {} | ⏰ {} | Score: {}\n", lead.amount, lead.deadline, lead.match_score));
            report.push_str(&format!("   🔗 {}\n\n", lead.url));
        }
    }
    
    if !bucket_b.is_empty() {
        report.push_str("## 📋 Bucket B (備援)\n\n");
        for (i, lead) in bucket_b.iter().take(10).enumerate() {
            report.push_str(&format!("{}. {} - {} (Score: {})\n", i + 1, lead.name, lead.amount, lead.match_score));
        }
        if bucket_b.len() > 10 {
            report.push_str(&format!("... and {} more\n", bucket_b.len() - 10));
        }
        report.push_str("\n");
    }
    
    if !filtered_out.is_empty() {
        report.push_str(&format!("⏭️ **Filtered out:** {} scholarships\n\n", filtered_out.len()));
    }
    
    if !errors.is_empty() {
        report.push_str(&format!("⚠️ **Errors:** {}\n\n", errors.len()));
    }
    
    report.push_str(&format!("📁 **Total in database:** {}", total_leads + bucket_a.len() + bucket_b.len()));
    
    report
}

fn build_summary_report(
    timestamp: &str,
    bucket_a: &[Lead],
    bucket_b: &[Lead],
    bucket_c: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
    dedup_stats: &normalize::DeduplicationStats,
) -> String {
    let mut report = format!("🔍 **ScholarshipOps Triage**\n📅 {}\n\n", timestamp);
    
    report.push_str(&format!("📊 A={} | B={} | C={}\n", bucket_a.len(), bucket_b.len(), bucket_c.len()));
    report.push_str(&format!("🔄 Dedup: {} removed\n\n", dedup_stats.duplicates_removed));
    
    if !bucket_a.is_empty() {
        report.push_str("🎯 **Top Picks:**\n");
        for (i, lead) in bucket_a.iter().take(3).enumerate() {
            let name = if lead.name.chars().count() > 35 {
                format!("{}...", lead.name.chars().take(32).collect::<String>())
            } else {
                lead.name.clone()
            };
            report.push_str(&format!("{}. {} | {}\n", i + 1, name, lead.amount));
        }
        if bucket_a.len() > 3 {
            report.push_str(&format!("   +{} more in A\n", bucket_a.len() - 3));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("📁 DB: {} | ⏭️ Filtered: {} | ⚠️ Errors: {}", 
        total_leads + bucket_a.len() + bucket_b.len(), 
        filtered_out.len(), 
        errors.len()
    ));
    
    report
}

fn build_markdown_report(
    timestamp: &str,
    leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    errors: &[String],
    total_leads: usize,
    profile: &Option<Profile>,
    dedup_stats: &normalize::DeduplicationStats,
) -> String {
    let mut report = format!("# ScholarshipOps Search Report\n\n**Date:** {}\n\n", timestamp);
    
    if let Some(p) = profile {
        report.push_str("## Your Profile\n\n");
        report.push_str(&format!("- **Nationality:** {}\n", p.nationality));
        report.push_str(&format!("- **Target:** {} ({})\n", p.target_university, p.programme_start));
        report.push_str(&format!("- **Level:** {}\n", p.programme_level));
        report.push_str("\n");
    }
    
    // Group by bucket
    let bucket_a: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::A)).collect();
    let bucket_b: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::B)).collect();
    
    report.push_str("## Results\n\n");
    report.push_str(&format!("- **Bucket A (主攻):** {} scholarships\n", bucket_a.len()));
    report.push_str(&format!("- **Bucket B (備援):** {} scholarships\n", bucket_b.len()));
    report.push_str(&format!("- **Filtered out:** {} scholarships\n", filtered_out.len()));
    report.push_str(&format!("- **Duplicates removed:** {} ({} unique keys with duplicates)\n", 
        dedup_stats.duplicates_removed, dedup_stats.dup_count_by_key.len()));
    report.push_str("\n");
    
    if !bucket_a.is_empty() {
        report.push_str("### Bucket A - High Priority\n\n");
        report.push_str("| # | Name | Amount | Deadline | Score | Effort |\n");
        report.push_str("|---|------|--------|----------|-------|--------|\n");
        
        for (i, lead) in bucket_a.iter().enumerate() {
            let effort = lead.effort_score.map(|e| format!("{}/100", e)).unwrap_or("-".to_string());
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score, effort
            ));
        }
        report.push_str("\n");
    }
    
    if !bucket_b.is_empty() {
        report.push_str("### Bucket B - Medium Priority\n\n");
        report.push_str("| # | Name | Amount | Deadline | Score |\n");
        report.push_str("|---|------|--------|----------|-------|\n");
        
        for (i, lead) in bucket_b.iter().take(20).enumerate() {
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score
            ));
        }
        if bucket_b.len() > 20 {
            report.push_str(&format!("\n*... and {} more*\n", bucket_b.len() - 20));
        }
        report.push_str("\n");
    }
    
    if !errors.is_empty() {
        report.push_str("## Errors\n\n");
        for err in errors {
            report.push_str(&format!("- {}\n", err));
        }
        report.push_str("\n");
    }
    
    report.push_str(&format!("## Statistics\n\n**Total leads in database:** {}\n", 
        total_leads + bucket_a.len() + bucket_b.len()));
    
    report
}

fn build_html_report(
    timestamp: &str,
    leads: &[Lead],
    filtered_out: &[(String, Vec<String>)],
    _errors: &[String],
    total_leads: usize,
    profile: &Option<Profile>,
    dedup_stats: &normalize::DeduplicationStats,
) -> String {
    let bucket_a: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::A)).collect();
    let bucket_b: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::B)).collect();
    let bucket_c: Vec<_> = leads.iter().filter(|l| l.bucket == Some(Bucket::C) || l.bucket.is_none()).collect();
    
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ScholarshipOps Triage Report</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
       line-height: 1.6; color: #333; background: #f5f5f5; padding: 20px; }
.container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
h1 { color: #2c3e50; margin-bottom: 10px; }
h2 { color: #34495e; margin-top: 30px; margin-bottom: 15px; border-bottom: 2px solid #3498db; padding-bottom: 5px; }
.timestamp { color: #7f8c8d; font-size: 0.9em; margin-bottom: 20px; }
.stats { display: flex; gap: 20px; margin: 20px 0; flex-wrap: wrap; }
.stat-box { background: #ecf0f1; padding: 15px 25px; border-radius: 8px; text-align: center; flex: 1; min-width: 120px; }
.stat-box.a { background: #d5f5e3; border-left: 4px solid #27ae60; }
.stat-box.b { background: #fdebd0; border-left: 4px solid #f39c12; }
.stat-box.c { background: #fadbd8; border-left: 4px solid #e74c3c; }
.stat-box.filtered { background: #e8e8e8; border-left: 4px solid #95a5a6; }
.stat-number { font-size: 2em; font-weight: bold; }
table { width: 100%; border-collapse: collapse; margin: 20px 0; }
th { background: #3498db; color: white; padding: 12px; text-align: left; }
td { padding: 10px; border-bottom: 1px solid #ddd; }
tr:hover { background: #f8f9fa; }
a { color: #3498db; }
.bucket-a { background: #d5f5e3; }
.bucket-b { background: #fdebd0; }
.bucket-c { background: #fadbd8; }
.filtered-row { background: #f5f5f5; }
.reason { font-size: 0.9em; color: #7f8c8d; }
</style>
</head>
<body>
<div class="container">
"#);
    
    html.push_str(&format!("<h1>🔍 ScholarshipOps Triage Report</h1>\n"));
    html.push_str(&format!("<p class=\"timestamp\">📅 {}</p>\n", timestamp));
    
    if let Some(p) = profile {
        html.push_str("<h2>👤 Your Profile</h2>\n<ul>\n");
        html.push_str(&format!("<li><strong>Nationality:</strong> {}</li>\n", p.nationality));
        html.push_str(&format!("<li><strong>Target:</strong> {} ({})</li>\n", p.target_university, p.programme_start));
        html.push_str(&format!("<li><strong>Level:</strong> {}</li>\n", p.programme_level));
        html.push_str("</ul>\n");
    }
    
    html.push_str("<div class=\"stats\">\n");
    html.push_str(&format!("<div class=\"stat-box a\"><div class=\"stat-number\">{}</div>Bucket A</div>\n", bucket_a.len()));
    html.push_str(&format!("<div class=\"stat-box b\"><div class=\"stat-number\">{}</div>Bucket B</div>\n", bucket_b.len()));
    html.push_str(&format!("<div class=\"stat-box c\"><div class=\"stat-number\">{}</div>Bucket C</div>\n", bucket_c.len()));
    html.push_str(&format!("<div class=\"stat-box filtered\"><div class=\"stat-number\">{}</div>Filtered</div>\n", filtered_out.len()));
    html.push_str("</div>\n");
    
    if !bucket_a.is_empty() {
        html.push_str("<h2>🎯 Bucket A - High Priority</h2>\n");
        html.push_str("<table><thead><tr><th>#</th><th>Name</th><th>Amount</th><th>Deadline</th><th>Score</th><th>Effort</th><th>Link</th></tr></thead><tbody>\n");
        
        for (i, lead) in bucket_a.iter().enumerate() {
            let effort = lead.effort_score.map(|e| format!("{}/100", e)).unwrap_or("-".to_string());
            html.push_str(&format!(
                "<tr class=\"bucket-a\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><a href=\"{}\" target=\"_blank\">Link</a></td></tr>\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score, effort, lead.url
            ));
        }
        html.push_str("</tbody></table>\n");
    }
    
    if !bucket_b.is_empty() {
        html.push_str("<h2>📋 Bucket B - Medium Priority</h2>\n");
        html.push_str("<table><thead><tr><th>#</th><th>Name</th><th>Amount</th><th>Deadline</th><th>Score</th><th>Link</th></tr></thead><tbody>\n");
        
        for (i, lead) in bucket_b.iter().take(30).enumerate() {
            html.push_str(&format!(
                "<tr class=\"bucket-b\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><a href=\"{}\" target=\"_blank\">Link</a></td></tr>\n",
                i + 1, lead.name, lead.amount, lead.deadline, lead.match_score, lead.url
            ));
        }
        html.push_str("</tbody></table>\n");
        
        if bucket_b.len() > 30 {
            html.push_str(&format!("<p><em>... and {} more</em></p>\n", bucket_b.len() - 30));
        }
    }
    
    if !bucket_c.is_empty() {
        html.push_str("<h2>❌ Bucket C - Rejected</h2>\n");
        html.push_str("<table><thead><tr><th>#</th><th>Name</th><th>Amount</th><th>Deadline</th><th>Rejection Reason</th><th>Link</th></tr></thead><tbody>\n");
        
        for (i, lead) in bucket_c.iter().take(20).enumerate() {
            // Get rejection reason from matched_rule_ids or match_reasons
            let reason = if !lead.matched_rule_ids.is_empty() {
                lead.matched_rule_ids.first().cloned().unwrap_or_default()
            } else if !lead.match_reasons.is_empty() {
                lead.match_reasons.first().cloned().unwrap_or_default()
            } else {
                "Not eligible".to_string()
            };
            
            html.push_str(&format!(
                "<tr class=\"bucket-c\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"reason\">{}</td><td><a href=\"{}\" target=\"_blank\">Link</a></td></tr>\n",
                i + 1, lead.name, lead.amount, lead.deadline, reason, lead.url
            ));
        }
        html.push_str("</tbody></table>\n");
        
        if bucket_c.len() > 20 {
            html.push_str(&format!("<p><em>... and {} more rejected scholarships</em></p>\n", bucket_c.len() - 20));
        }
    }
    
    if !filtered_out.is_empty() {
        html.push_str("<h2>⏭️ Filtered Out</h2>\n");
        html.push_str("<table><thead><tr><th>#</th><th>Name</th><th>Filter Reasons</th></tr></thead><tbody>\n");
        
        for (i, (name, reasons)) in filtered_out.iter().take(20).enumerate() {
            let reasons_str = if reasons.is_empty() {
                "Keyword mismatch".to_string()
            } else {
                reasons.join(", ")
            };
            
            html.push_str(&format!(
                "<tr class=\"filtered-row\"><td>{}</td><td>{}</td><td class=\"reason\">{}</td></tr>\n",
                i + 1, name, reasons_str
            ));
        }
        html.push_str("</tbody></table>\n");
        
        if filtered_out.len() > 20 {
            html.push_str(&format!("<p><em>... and {} more filtered scholarships</em></p>\n", filtered_out.len() - 20));
        }
    }
    
    html.push_str(&format!("<h2>📊 Statistics</h2>\n"));
    html.push_str(&format!("<p><strong>Total leads in database:</strong> {}</p>\n", 
        total_leads + bucket_a.len() + bucket_b.len()));
    html.push_str(&format!("<p><strong>Duplicates removed:</strong> {} ({} unique keys with duplicates)</p>\n", 
        dedup_stats.duplicates_removed, dedup_stats.dup_count_by_key.len()));
    
    html.push_str("</div>\n</body>\n</html>");
    
    html
}

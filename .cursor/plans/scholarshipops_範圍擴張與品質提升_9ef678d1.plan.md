---
name: ""
overview: ""
todos: []
isProject: false
---

---

name: ScholarshipOps 範圍擴張與品質提升

overview: 實作三層式 Discovery 策略、修復日期解析與 dedupe、對 403/timeout 做分流、擴張來源池，並優化 GitHub Actions workflow

todos:

                                                                                                                                - id: p1-date-validation

content: 修復日期解析驗證：在 parse_deadline 函數中加入合法性驗證（年份 2020-2100，月份 1-12，日期有效），拒絕 68-58-58 等無效日期

status: in_progress

                                                                                                                                - id: p1-dedup-content-hash

content: 強化 Deduplication：在 deduplicate_leads_with_stats 中加入 content-hash 作為第二層去重，使用 url_state::UrlStateStorage::calculate_content_hash

status: pending

                                                                                                                                - id: p2-layer1-index-pages

content: Layer 1 官方清單：在 university.rs 實作 discover_from_index_pages，從索引頁提取 /scholarships/ links，整合到 scrape 函數

status: pending

                                                                                                                                - id: p2-layer2-sitemap-integration

content: Layer 2 Sitemap/RSS 整合：在 main.rs 新增 Stage 0.5 Discovery，對每個 source 嘗試 discovery::discover_urls 和 discovery::parse_sitemap

status: pending

                                                                                                                                - id: p2-layer3-browser-optimization

content: Layer 3 Browser 優化：確保 js_detector::needs_browser 只在 HTTP 抓取失敗或欄位不完整時觸發

status: pending

                                                                                                                                - id: p3-fallback-strategies

content: 403/Timeout 分流：在 SourceHealth 新增 fallback_strategies 欄位，修改 update_health 邏輯，不立即 auto-disable，而是嘗試 sitemap/rss fallback

status: pending

                                                                                                                                - id: p3-fallback-execution

content: Fallback 策略執行：在 main.rs Stage 1 中，若 source 有 fallback_strategies，執行對應的 discovery 策略

status: pending

status: pending

                                                                                                                                - id: p4-government-agencies

content: 新增 UK 政府/公共機構：在 sources.yml 新增 UKRI, Scottish Funding Council 等 5-8 個政府機構來源

status: pending

                                                                                                                                - id: p4-foundations

content: 新增基金會/產業獎學金：在 sources.yml 新增科技/工程向基金會（IEEE, Google, Microsoft 等 5-10 個）

status: pending

                                                                                                                                - id: p4-uk-universities-external-discovery

content: 新增其他 UK 大學來源（external_links_only 模式）：實作 discovery_mode: external_links_only，只提取外部資助方連結，不追校內獎學金 detail pages

status: pending

                                                                                                                                - id: p4-sources-external-domains-whitelist

content: 實作 allow_domains_outbound 白名單：在 sources.yml 新增 allow_domains_outbound 欄位，限制 crawler 只追蹤白名單外部域名

status: pending

                                                                                                                                - id: p5-matrix-strategy

content: GitHub Actions Matrix 分片：修改 search.yml，將 sources 分成 10 片，每片產生 leads.part-*.jsonl，最後 merge job 合併

status: pending

                                                                                                                                - id: p5-two-stage-workflow

content: 分兩段 Workflow（替代方案）：實作 discovery job（只抓 index/sitemap）和 fetch_extract job（處理 queue）

status: pending

                                                                                                                                - id: p0-diagnosis-schedule-branch

content: 問題診斷：確認 GitHub Actions schedule 是否在 default branch 上執行，若在 feature branch 則 schedule 不會觸發

status: pending

                                                                                                                                - id: p0-diagnosis-glasgow-js

content: 問題診斷：確認 Glasgow /scholarships/all/ 是否為 JS-templated 頁面（{{scholar.title}}），需要瀏覽器渲染

status: pending

                                                                                                                                - id: p0-diagnosis-runner-time-limit

content: 問題診斷：確認 GitHub-hosted runner 執行時間上限（通常 6 小時），規劃 matrix sharding 策略

status: pending

                                                                                                                                - id: p0-sources-glasgow-selenium

content: Sources v2：將 Glasgow /scholarships/all/ 和 /scholarships/search/ 改為 selenium scraper（JS-templated）

status: pending

                                                                                                                                - id: p0-sources-glasgow-landing

content: Sources v2：新增 Glasgow /scholarships/ landing 頁作為額外 seed

status: pending

                                                                                                                                - id: p0-sources-disable-saltire-saas

content: Sources v2：將 Saltire 和 SAAS 預設 disabled（台灣不符合資格）

status: pending

                                                                                                                                - id: p0-sources-great-selenium

content: Sources v2：將 GREAT Scholarships 和 British Council Study UK 改為 selenium scraper（避免 403）

status: pending

                                                                                                                                - id: p0-criteria-required-minimal

content: Criteria v2：將 required keywords 減到最小集合（scholarship|funding|bursary|award），其餘移到 preferred

status: pending

                                                                                                                                - id: p0-criteria-gpa-degree-aware

content: Criteria v2：GPA 判斷改為 degree-aware（undergrad vs postgrad），max_gpa_requirement 改為 4.0

status: pending

                                                                                                                                - id: p0-rules-intl-fee-status

content: Rules v2：新增 P-INTL-FEE-002 規則，匹配 "international fee status" / "overseas fee status" 語句

status: pending

                                                                                                                                - id: p0-rules-scam-allowlist

content: Rules v2：修正 S-SCAM-001，加入官方網域 allowlist（gla.ac.uk, gov.uk 等），避免誤判 hardship fund

status: pending

                                                                                                                                - id: p0-rules-nontarget-source-type

content: Rules v2：修正 E-NONTARGET-001，只套用在 university sources，不套用在 foundation/government sources

status: pending

                                                                                                                                - id: p0-workflow-mode-wide

content: Workflow v2：新增 mode=wide|focused 選項，wide 模式啟用 discovery + matrix sharding

status: pending

                                                                                                                                - id: p0-workflow-prepare-matrix

content: Workflow v2：新增 prepare job 產生 shard matrix，支援 SHARD_INDEX/SHARD_TOTAL 環境變數

status: pending

                                                                                                                                - id: p0-workflow-timeout-340min

content: Workflow v2：設定 timeout-minutes: 340，確保在 GitHub-hosted runner 限制內

status: pending

isProject: false

---

# ScholarshipOps 範圍擴張與品質提升計劃

## 問題分析

1. **來源池太小**：19 個 sources，其中 British Council Study UK、FindAPhD 因 403 被 auto-disabled
2. **解析品質問題**：日期解析有 bug（68-58-58 無效日期被接受），dedupe 報告顯示 0 但實際有重複
3. **Discovery 未整合**：已有 `discovery.rs` 模組但未整合到主流程
4. **403/timeout 處理**：直接 auto-disable，應改走 sitemap/RSS 分流

## 實作計劃

### Phase 1: 修復核心品質問題（立即修復）

#### 1.1 修復日期解析驗證

**檔案**：`scripts/search_scholarships/src/filter.rs`, `scripts/search_scholarships/src/rules.rs`, `scripts/search_scholarships/src/triage.rs`

**變更**：

- 在 `parse_deadline` 函數中加入日期合法性驗證
- 驗證規則：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 年份必須在 2020-2100 範圍內
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 月份必須在 1-12 範圍內
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 日期必須符合該月份的有效天數
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 拒絕明顯無效的日期（如 68-58-58）
- 若解析失敗或驗證失敗，返回 `None` 並記錄到 `risk_flags`（如 "invalid_deadline_format"）

**實作細節**：

```rust
fn parse_deadline(deadline: &str) -> Result<NaiveDate, ()> {
    // ... 現有解析邏輯 ...
    
    // 驗證日期合法性
    if let Some(date) = parsed_date {
        // 檢查年份範圍
        if date.year() < 2020 || date.year() > 2100 {
            return Err(());
        }
        // 檢查月份和日期是否有效（chrono 會自動處理，但額外檢查更安全）
        if date.month() < 1 || date.month() > 12 {
            return Err(());
        }
        return Ok(date);
    }
    Err(())
}
```

#### 1.2 強化 Deduplication（URL canonicalize + content-hash）

**檔案**：`scripts/search_scholarships/src/normalize.rs`

**變更**：

- 在 `deduplicate_leads_with_stats` 中加入 content-hash 作為第二層去重
- 使用 `url_state::UrlStateStorage::calculate_content_hash` 計算內容 hash
- 若 entity-level key 相同但 content-hash 不同，視為不同版本（更新現有 lead）
- 若 entity-level key 不同但 content-hash 相同，視為重複（使用更完整的 lead）

**實作細節**：

```rust
pub fn deduplicate_leads_with_stats(leads: Vec<Lead>) -> (Vec<Lead>, DeduplicationStats) {
    let mut dedup_map: HashMap<String, Lead> = HashMap::new();
    let mut content_hash_map: HashMap<String, String> = HashMap::new(); // content_hash -> entity_key
    
    for mut lead in leads {
        let entity_key = generate_entity_dedup_key(&lead);
        let content_hash = calculate_lead_content_hash(&lead);
        
        // 檢查 content-hash 是否已存在（不同 entity_key 但相同內容）
        if let Some(existing_key) = content_hash_map.get(&content_hash) {
            if existing_key != &entity_key {
                // 內容相同但 entity_key 不同，視為重複
                continue;
            }
        }
        
        // ... 現有邏輯 ...
    }
}

fn calculate_lead_content_hash(lead: &Lead) -> String {
    let content = format!("{}|{}|{}|{}", 
        lead.name, lead.amount, lead.deadline, lead.eligibility.join(","));
    url_state::UrlStateStorage::calculate_content_hash(content.as_bytes())
}
```

### Phase 2: 三層式 Discovery 整合

#### 2.1 Layer 1: 官方清單（索引頁）擴展

**檔案**：`scripts/search_scholarships/src/scrapers/university.rs`

**變更**：

- 為每個 university source 新增 `index_urls[]` 配置（在 `sources.yml` 中）
- 實作 `discover_from_index_pages` 函數：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 從索引頁 HTML 提取所有 `/scholarships/` 內的 links
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 使用 `detail_url_regex` 過濾允許深入的 URL pattern
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 限制 `max_depth`（通常 2 層）
- 整合到 `scrape` 函數：先 discovery，再 scrape detail pages

**實作細節**：

```rust
pub async fn discover_from_index_pages(
    client: &Client,
    index_urls: &[String],
    detail_url_regex: Option<&Regex>,
    max_depth: u32,
) -> Result<Vec<String>> {
    let mut discovered_urls = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    for index_url in index_urls {
        queue.push_back((index_url.clone(), 0));
    }
    
    while let Some((url, depth)) = queue.pop_front() {
        if depth > max_depth || visited.contains(&url) {
            continue;
        }
        visited.insert(url.clone());
        
        // Fetch HTML
        let html = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&html);
        
        // Extract scholarship links
        if let Ok(selector) = Selector::parse("a[href*='/scholarships/'], a[href*='/funding/']") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    let resolved_url = resolve_url(&url, href);
                    
                    // Apply detail_url_regex filter
                    if let Some(regex) = detail_url_regex {
                        if !regex.is_match(&resolved_url) {
                            continue;
                        }
                    }
                    
                    if !visited.contains(&resolved_url) {
                        discovered_urls.push(resolved_url.clone());
                        if depth < max_depth {
                            queue.push_back((resolved_url, depth + 1));
                        }
                    }
                }
            }
        }
    }
    
    Ok(discovered_urls)
}
```

#### 2.2 Layer 2: Sitemap/RSS 整合

**檔案**：`scripts/search_scholarships/src/main.rs`

**變更**：

- 在 Stage 1（Scrape Sources）之前新增 **Stage 0.5: Discovery**
- 對每個 source 嘗試：

                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                1. `discovery::discover_urls`（robots.txt -> sitemap）
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                2. `discovery::parse_sitemap`（解析 sitemap XML）
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                3. 從 sitemap 提取的 URLs 過濾（含 `scholarship|funding|bursary|award` 關鍵字）
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                4. 將候選 URLs 加入 `sources_to_scrape` 或建立新的 `discovered_sources` 列表

**實作細節**：

```rust
// Stage 0.5: Discovery (sitemap/RSS)
println!("Stage 0.5: Discovering URLs from sitemaps/RSS...");
let mut discovered_urls: Vec<discovery::CandidateUrl> = Vec::new();

for source in &enabled_sources {
    let discovery_config = discovery::DiscoveryConfig::default();
    
    // Try discovery
    if let Ok(candidates) = discovery::discover_urls(&client, source, &discovery_config).await {
        for candidate in candidates {
            // Filter by keywords
            let url_lower = candidate.url.to_lowercase();
            if url_lower.contains("scholarship") || 
               url_lower.contains("funding") || 
               url_lower.contains("bursary") || 
               url_lower.contains("award") {
                discovered_urls.push(candidate);
            }
        }
    }
    
    // Try parsing sitemap if found
    if let Ok(sitemap_urls) = discovery::parse_sitemap(&client, &format!("{}/sitemap.xml", base_url), &discovery_config).await {
        discovered_urls.extend(sitemap_urls);
    }
}

println!("  Discovered {} candidate URLs", discovered_urls.len());
```

#### 2.3 Layer 3: Browser Queue 優化

**檔案**：`scripts/search_scholarships/src/main.rs`

**變更**：

- 確保 Stage 1.55（JS-Heavy Detection）只在 HTTP 抓取失敗或頁面是 SPA 時才觸發
- 優化 `js_detector::needs_browser` 的觸發條件：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 只有在 `extraction_result` 為 `None` 或欄位不完整時才標記為需要 browser
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 若 HTTP 抓取成功且有完整欄位，跳過 browser queue

### Phase 3: 403/Timeout 分流策略

#### 3.1 修改 Source Health 邏輯

**檔案**：`scripts/search_scholarships/src/source_health.rs`

**變更**：

- 在 `SourceHealth` 結構中新增 `fallback_strategies: Vec<String>` 欄位（如 `["sitemap", "rss"]`）
- 修改 `update_health` 函數：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 若遇到 403，不立即 auto-disable，而是標記 `fallback_strategies = ["sitemap", "rss"]`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 若遇到 timeout，標記 `fallback_strategies = ["sitemap", "rss", "head_request"]`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 只有在所有 fallback 策略都失敗後才 auto-disable

**實作細節**：

```rust
pub struct SourceHealth {
    // ... 現有欄位 ...
    pub fallback_strategies: Vec<String>, // "sitemap", "rss", "head_request"
}

pub fn update_health(...) {
    // ...
    if result.status == SourceStatus::Forbidden {
        // 不立即 disable，嘗試 fallback
        h.fallback_strategies = vec!["sitemap".to_string(), "rss".to_string()];
        // 只有在 fallback 也失敗後才增加 consecutive_failures
    } else if result.status == SourceStatus::Timeout {
        h.fallback_strategies = vec!["sitemap".to_string(), "rss".to_string(), "head_request".to_string()];
    }
    // ...
}
```

#### 3.2 實作 Fallback 策略執行

**檔案**：`scripts/search_scholarships/src/main.rs`

**變更**：

- 在 Stage 1 中，若 source 有 `fallback_strategies`，嘗試執行：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - `sitemap`：使用 `discovery::parse_sitemap`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - `rss`：使用 `discovery::discover_feeds`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - `head_request`：先 HEAD 請求，再 GET

**實作細節**：

```rust
// 在 scrape source 時
if let Some(health) = health_file.sources.iter().find(|h| h.url == source.url) {
    if !health.fallback_strategies.is_empty() {
        // 嘗試 fallback 策略
        for strategy in &health.fallback_strategies {
            match strategy.as_str() {
                "sitemap" => {
                    if let Ok(urls) = discovery::parse_sitemap(&client, &format!("{}/sitemap.xml", base_url), &config).await {
                        // 使用 sitemap URLs
                    }
                }
                "rss" => {
                    if let Ok(feeds) = discovery::discover_feeds(&client, &base_url).await {
                        // 使用 RSS feeds
                    }
                }
                "head_request" => {
                    // 先 HEAD，再 GET
                }
                _ => {}
            }
        }
    }
}
```

### Phase 4: 來源擴張

#### 4.1 新增 UK 大學官方索引頁（作為外部資助方發現器）

**檔案**：`tracking/sources.yml`、`scripts/search_scholarships/src/types.rs`、`scripts/search_scholarships/src/scrapers/university.rs`

**核心概念**：加其他 UK 大學來源（不是為了拿他們校內獎學金）

你加 Edinburgh / Manchester 等「scholarships index pages」的價值，主要是把它們當 **"portable / external scholarships 的發現器（lead generator）"**，而不是把它們的校內獎學金當作你的目標。

**為什麼要加其他 UK 大學來源？**

**A) 抓到 Glasgow 頁面上沒列到的「外部資助方」**

很多大學的 funding 入口頁，會明確引導你去找政府機構、慈善信託、外部公司等資助來源，而這些才是你真正想要的「可攜、可用在 Glasgow」的錢。

以 Edinburgh 的 scholarships 入口為例，它明確寫到 scholarship providers 包含政府、research councils、charitable trusts、external bodies/private companies。

換句話說：你加其他大學的 index page，是為了讓 crawler 自動把外部資助方、外部獎學金的官方頁抓出來，進一步做 triage。

**B) UK-wide 計畫常「分散在各校頁面」而不是只有一個總表**

例如 GREAT Scholarships 是 British Council + 多校合作的 scheme，各校會有自己的 GREAT 頁面，且 eligible countries 也可能不同。

如果你只盯 Glasgow，有時會錯過「某方案其實有 Glasgow 版本，但 Glasgow 頁面沒寫得清楚／更新慢」的情況。

**C) 反向驗證與去重（提高品質、不是增加噪音）**

當同一個外部獎學金在多校網站出現，你可以利用「多處同名出現」做：

- 去重（entity dedupe）
- 提取官方主辦方（找到最權威的 funder page）
- 比對 deadline / eligibility 差異（降低誤判）

**什麼時候「不該加」？**

如果你的目標是：只要 Glasgow 校內獎學金 + 英國全國級（Chevening/CSC/Rotary…），那你確實可以不加其他學校，因為會引入大量「不可攜、非 Glasgow」噪音。

而且你已經在 rules 裡有 E-NONTARGET-001 這種 gate（非 Glasgow 且不可攜 → C）。這表示你現在的系統設計其實是偏"Glasgow-focused"，所以「加 10–15 所」必須搭配正確的 crawling 策略，否則只會浪費抓取與解析資源。

**正確做法：加，但把它們設計成「只產 leads，不產 scholarship」**

這是關鍵。你要的是 coverage 擴大，但不讓 Bucket 被非目標洗版。

**建議在 sources.yml 增加 2 個概念欄位（最小改動）**

對 Tier 3 的 UK universities 設定：

1. **`discovery_mode: external_links_only`**

            - 只抽「外部連結」：gov.uk、fcdo、chevening、cscuk、rotary、britishcouncil、charity/foundation 等
            - 不追該校內部 scholarship detail pages（或追了也直接標記為 non-portable → C，且不輸出到主要報表）

2. **`allow_domains_outbound: [...]`**

            - 白名單外部域名，避免 crawler 任意發散

這樣你就能回答你自己問的那句話：

**"為什麼要新增其他 UK 大學來源？"**

→ 因為它們是「外部資助方/可攜獎學金」的入口，而不是要收集那些學校自己的獎學金。

**變更**：

- 新增 Russell Group 大學的 scholarships index pages：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - Edinburgh: `https://www.ed.ac.uk/student-funding/postgraduate/international`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - St Andrews: `https://www.st-andrews.ac.uk/study/fees-and-funding/postgraduate/`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - Strathclyde: `https://www.strath.ac.uk/studywithus/scholarships/`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - Manchester: `https://www.manchester.ac.uk/study/international/finance/scholarships/`
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 等（共 10-15 個）
- 每個 source 配置 `discovery_mode: external_links_only` 和 `allow_domains_outbound` 白名單

**實作細節**：

```yaml
# Tier 3: UK Universities (External Links Discovery Only)
- name: "Edinburgh Scholarships Index"
  type: "university"
  url: "https://www.ed.ac.uk/student-funding/postgraduate/international"
  enabled: true
  priority: 3
  scraper: "university"
  discovery_mode: "external_links_only"  # 只提取外部連結
  allow_domains_outbound:
  - "gov.uk"
  - "fcdo.gov.uk"
  - "cscuk.fcdo.gov.uk"
  - "chevening.org"
  - "rotary.org"
  - "britishcouncil.org"
  - "ukri.org"
  - "sfc.ac.uk"
  - "charitycommission.gov.uk"
    # 不包含該校自己的域名（如 ed.ac.uk）
```

**Rust 實作變更**：

在 `types.rs` 中擴展 `Source` struct：

```rust
pub struct Source {
    // ... 現有欄位 ...
    pub discovery_mode: Option<String>,  // "external_links_only", "full", None
    pub allow_domains_outbound: Option<Vec<String>>,  // 白名單外部域名
}
```

在 `scrapers/university.rs` 中實作 `extract_external_links` 函數：

```rust
pub fn extract_external_links(
    html: &str,
    base_url: &str,
    allow_domains: &[String],
) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut external_links = Vec::new();
    
    // 提取所有連結
    if let Ok(selector) = Selector::parse("a[href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let resolved_url = resolve_url(base_url, href);
                
                // 檢查是否為外部連結且在白名單內
                if let Ok(parsed) = url::Url::parse(&resolved_url) {
                    if let Some(domain) = parsed.host_str() {
                        // 檢查是否為白名單域名
                        if allow_domains.iter().any(|allowed| domain.contains(allowed)) {
                            // 排除該校自己的域名
                            if !domain.contains(extract_domain(base_url)) {
                                external_links.push(resolved_url);
                            }
                        }
                    }
                }
            }
        }
    }
    
    external_links
}
```

在 `main.rs` 中，當 `discovery_mode == "external_links_only"` 時：

- 只提取外部連結
- 將提取的連結加入 discovery queue
- 不追蹤該校內部的 scholarship detail pages
- 或追蹤後直接標記為 `is_portable: false`，進入 Bucket C

#### 4.2 新增 UK 政府/公共機構

**檔案**：`tracking/sources.yml`

**變更**：

- UKRI (UK Research and Innovation): `https://www.ukri.org/opportunity/`
- Scottish Funding Council: `https://www.sfc.ac.uk/`
- 等（共 5-8 個）

#### 4.3 新增基金會/產業獎學金

**檔案**：`tracking/sources.yml`

**變更**：

- 科技/工程向基金會（以官方頁為主，避免第三方 403）：
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - IEEE Foundation
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - Google Scholarships
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - Microsoft Scholarships
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                - 等（共 5-10 個）

### Phase 5: GitHub Actions 優化

#### 5.1 Matrix 分片策略

**檔案**：`.github/workflows/search.yml`

**變更**：

- 將 sources 分成 N 片（例如 10 片）
- 每片 job 產生 `leads.part-{matrix.part}.jsonl`
- 最後一個 merge job 合併 + dedupe + triage

**實作細節**：

```yaml
jobs:
  discover:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        part: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    steps:
   - name: Run search (part ${{ matrix.part }})
        run: |
          ./target/release/search_scholarships --part ${{ matrix.part }} --total-parts 10
   - name: Upload leads part
        uses: actions/upload-artifact@v4
        with:
          name: leads-part-${{ matrix.part }}
          path: leads.part-${{ matrix.part }}.jsonl

  merge:
    needs: discover
    runs-on: ubuntu-latest
    steps:
   - name: Download all parts
        uses: actions/download-artifact@v4
   - name: Merge and dedupe
        run: |
          cat leads.part-*.jsonl | ./target/release/search_scholarships --merge-mode
```

#### 5.2 分兩段 Workflow（替代方案）

**檔案**：`.github/workflows/search.yml`

**變更**：

- `discovery` job：只抓 index/sitemap，產生 URL queue
- `fetch+extract` job：只處理 queue

**實作細節**：

```yaml


jobs:
  discovery:
    runs-on: ubuntu-latest
    steps:
   - name: Discover URLs
        run: ./target/release/search_scholarships --mode discovery
   - name: Upload URL queue
        uses: actions/upload-artifact@v4
        with:
          name: url-queue
          path: tracking/url_queue.jsonl

  fetch_extract:
    needs: discovery
    runs-on: ubuntu-latest
    steps:
   - name: Download URL queue
        uses: actions/download-artifact@v4
   - name: Fetch and extract
        run: ./target/release/search_scholarships --mode fetch
```

## 實作順序

1. **Phase 1**（立即修復）：日期解析驗證 + Deduplication 強化
2. **Phase 2**（核心功能）：三層式 Discovery 整合
3. **Phase 3**（穩定性）：403/Timeout 分流
4. **Phase 4**（擴張）：來源擴張
5. **Phase 5**（效能）：GitHub Actions 優化

## 預期成果

- **來源池**：從 19 個擴張到 50-80 個
- **覆蓋率**：透過 sitemap/RSS discovery，每個 source 的 URL 覆蓋率提升 3-5 倍
- **品質**：日期解析錯誤率從 ~5% 降到 <1%，dedupe 準確率提升到 >95%
- **穩定性**：403/timeout 來源不再直接失去，透過 fallback 策略維持覆蓋
- **效能**：GitHub Actions Matrix 分片後，單一 job 執行時間從 6 小時降到 30-40 分鐘

## Phase 0: 問題診斷（為什麼範圍不夠廣）

### 0.1 Schedule 可能根本沒在想要的 branch 上跑

**問題**：GitHub Actions 的 schedule 事件是以預設分支（default branch）上的 workflow 檔案為準，不是你在 feature branch 寫了 cron 就會照跑。

**影響**：如果你的 default branch 不是 feature/ScholarshipOps，那 cron 很可能沒有定期產出 productions（你看到的多半是 push / manual）。

**解決方案**：將 workflow 檔案合併到 default branch，或確認 default branch 上的 workflow 檔案包含 schedule 配置。

### 0.2 Glasgow scholarships listing 頁是 Client-side template

**問題**：`https://www.gla.ac.uk/scholarships/all/` 頁面內容包含 `{{scholar.title}}` 這種模板符號，代表它需要前端 JS 取資料再渲染。

**影響**：若你的 university scraper 偏向 requests/HTML parse，不走 Selenium/瀏覽器，就會抓到空壳 → 覆蓋率會被「入口頁」卡死。

**解決方案**：

- 快速路線：保留 Selenium（你 workflow 已經起 ChromeDriver），在 headless 模式打開頁面，等待渲染完成後把 scholarship links 全撈出來。
- 進階路線：在 Selenium 啟動時收集 network logs（performance logs）抓到 JSON endpoint，之後改用 HTTP 直抓（速度會快 10–50 倍）。

### 0.3 GitHub-hosted runner 單一 job 有執行上限

**問題**：GitHub-hosted runner 的 job execution time limit 常見是 6 小時（官方文件有列）。

**影響**：擴大範圍要靠「分治」（matrix/job fan-out），不是單 job 無限加深。

**解決方案**：實作 matrix sharding，將 sources 分成多個 shards，每個 shard 獨立執行，最後合併結果。

## Phase 6: Sources 擴展策略 v2（重點：不要只是"加更多 URL"，而是"加發現能力"）

### 6.1 Glasgow 專用：把「入口頁」改成「可枚舉 detail pages 的機制」

**檔案**：`tracking/sources.yml`

**變更**：

- 為 `gla.ac.uk` 加一個 discovery mode：`crawl_internal`
- 做法：從幾個起點頁（`/scholarships/`、`/scholarships/all/`、`/study/postgraduate/fees-funding/`...）開始，只跟進同網域且符合 `^/scholarships/` 的連結，抓到大量 scholarship detail pages 後再逐頁 triage。
- 好處：你不再依賴 `/all/` 的 JS 列表是否能解析。

**針對 `/scholarships/all/`**：

- 要嘛 Selenium 抓 DOM，要嘛逆向它的資料端點
- 快速路線：保留 Selenium（你 workflow 已經起 ChromeDriver），在 headless 模式打開頁面，等待渲染完成後把 scholarship links 全撈出來。
- 進階路線：在 Selenium 啟動時收集 network logs（performance logs）抓到 JSON endpoint，之後改用 HTTP 直抓（速度會快 10–50 倍）。

**具體修改**：

```yaml
- name: "Glasgow All Scholarships"
  type: "university"
  url: "https://www.gla.ac.uk/scholarships/all/"
  enabled: true
  priority: 1
  scraper: "selenium"   # JS-templated listing ({{scholar.title}}); needs browser-render

- name: "Glasgow Scholarship Search"
  type: "university"
  url: "https://www.gla.ac.uk/scholarships/search/"
  enabled: true
  priority: 1
  scraper: "selenium"   # search UI is also JS-heavy on UofG scholarships

- name: "Glasgow Scholarships Landing"
  type: "university"
  url: "https://www.gla.ac.uk/scholarships/"
  enabled: true
  priority: 1
  scraper: "university"
```

### 6.2 Tier 2/UK-wide：改成「官方/高可信入口」＋「國別分頁」策略

**加入 GOV.UK 的權威入口（當作可擴散的 seed）**：

- GOV.UK 有整理 "Postgraduate scholarships for international students"，並導向 Chevening / Commonwealth 等。
- 這類頁面很適合做「種子」，再把每個外連當 source candidate（但要做 trust / risk 管控）。

**GREAT Scholarships**：

- 你需要的是"global index + country filter"，不是隨便抓某國頁
- GREAT Scholarships 是分國家頁，例如 2026–27 對西班牙的頁面、對越南的頁面內容完全不同。
- 你的 sources.yaml 應該：
                                                                - 先放 GREAT 的 global / hub 入口（或 British Council 的總表頁）
                                                                - triage 時再判斷是否包含 Taiwan（多半不含，就直接 C）

**Saltire Scholarships**：

- 請直接 hard-reject（台灣不在 eligible list）
- Saltire eligibility 明確列出 eligible citizenship（Canada/India/Japan/USA/Pakistan/China HK…），沒有 Taiwan。
- 建議：sources.yaml 把 Saltire 預設 disabled，或保留但 triage 直接 C（節省計算資源）。

**具體修改**：

```yaml
- name: "GREAT Scholarships"
  type: "government"
  url: "https://study-uk.britishcouncil.org/scholarships/great-scholarships"
  enabled: true
  priority: 2
  scraper: "selenium"   # British Council pages often throttle/geo-block; browser fallback helps

- name: "British Council Study UK"
  type: "third_party"
  url: "https://study-uk.britishcouncil.org/scholarships"
  enabled: true
  priority: 2
  scraper: "selenium"   # previously observed 403 in your productions; use browser to reduce blocks

- name: "SAAS - Student Awards Agency Scotland"
  type: "government"
  url: "https://www.saas.gov.uk/"
  enabled: false  # generally home-fee aligned; Taiwan applicants typically ineligible
  priority: 2
  scraper: "government"

- name: "Scotland's Saltire Scholarships"
  type: "government"
  url: "https://www.scotland.org/study/saltire-scholarships"
  enabled: false  # citizenship limited (Canada/China(HK)/India/Japan/Pakistan/USA)
  priority: 2
  scraper: "government"
```

### 6.3 非獎學金但"錢相關"的 Glasgow funds：要分類，不要混在 scholarship

**把 MyGlasgow 的 Financial Support Fund 標記為 hardship_fund（不是 scholarship）**：

- 該基金明確說不能提供 bursary/scholarship，也不能支援"開始就讀"用途，屬於 emergency living-cost support。
- 意義：你可以留著，但在輸出上要分欄位（Scholarship vs Hardship/Emergency），避免誤判"有搞頭"。

## Phase 7: criteria.yml v2（你現在的設計很容易「把真正的獎學金在 discovery 階段就殺掉」）

### 7.1 把 required keywords 從「硬門檻」改成「加權訊號」

**問題型態**：

- 你現在 required 裡面像 UK master eligible / Open international / all nationalities 這種字串，很多官方頁不會用這種語句，會寫成 "open to overseas fee payers" 或 "international fee status"。
- 結果：會造成 false negatives（範圍變小）。

**改法（核心原則）**：

- discovery 階段：只要像 scholarship|funding|bursary|award 命中就先收下。
- triage 階段：再用 rules.yaml 做 hard reject + scoring。
- 也就是把 criteria.yml 的 required 降到 1–2 個最小集合，其餘放 preferred 當 scoring。

**具體修改**：

```yaml
criteria:
  required:
    # Minimal "funding intent" gate only (discovery should be permissive)
  - "scholarship"
  - "funding"
  preferred:
  - "international"
  - "overseas"
  - "international fee status"
  - "postgraduate taught"
  - "masters"
  - "MSc"
  - "all nationalities"
  - "no nationality restriction"
  - "full tuition"
  - "tuition fee discount"
  - "fully funded"
  - "stipend"
```

### 7.2 GPA：你現在的模型需要「分學位」判斷，不然會誤殺或誤留

**現況風險**：

- 你同時有 Undergrad GPA（2.92/4.0）與 Master GPA（3.96/4.0），而很多獎學金寫 "minimum GPA 3.25" 但沒說是 undergrad 還是 cumulative。

**改法**：

- 在 profile 增加：
                                                                - `gpa_undergrad: 2.92`
                                                                - `gpa_postgrad: 3.96`
- 在 rules 增加：
                                                                - 若頁面出現 undergraduate GPA / bachelor GPA → 用 undergrad 判斷
                                                                - 若頁面出現 current postgraduate / master GPA → 用 postgrad 判斷
                                                                - 若不明確 → 不 hard reject，降權到 B（需要人工確認）

**具體修改**：

```yaml
profile:
  education:
    gpa_undergrad: 2.92
    gpa_postgrad: 3.96
  min_deadline: "2026-02-01"
  # Keep permissive at discovery; do degree-aware GPA checks in rules/triage.
  max_gpa_requirement: 4.0
```

## Phase 8: rules.yaml v2（三個你很可能會踩的品質坑）

### 8.1 Scam 規則的「bank account」會誤傷官方 hardship fund

**現象**：

- Glasgow Financial Support Fund 申請要求 bank statements / bank account details。
- 但這不是 scam，而是正常審核。

**改法**：

- Scam 規則加 allowlist：gla.ac.uk, gov.uk, britishcouncil.*, cscuk.fcdo.gov.uk 等可信網域出現 bank 字眼時，不判 scam，改走 hardship_fund 分類。

**具體修改**：

```yaml
hard_reject_rules:
 - id: "S-SCAM-001"
    name: "Payment Required"
    stage: "trust"
    description: "Requires payment or bank details"
    when:
      any_regex:
    - "(?i)credit\\s*card"
    - "(?i)bank\\s*account"
    - "(?i)processing\\s*fee"
    - "(?i)application\\s*fee\\s*\\$"
    - "(?i)pay\\s*to\\s*apply"
      not_any_regex:
        # Allowlist official domains where bank statements are normal eligibility checks
    - "(?i)gla\\.ac\\.uk"
    - "(?i)gov\\.uk"
    - "(?i)cscuk\\.fcdo\\.gov\\.uk"
    - "(?i)chevening\\.org"
    - "(?i)rotary\\.org"
    action:
      bucket: "C"
      reason: "Scam-like language: payment or bank details requested."
```

### 8.2 Non-target university gate 太早套用會讓你「永遠找不到外部獎學金」

**改法**：

- E-NONTARGET-001 只應套用在「明確為某校專屬、且不可攜」的頁面。
- 對於 foundation / corporate / government 類來源，應改成：
                                                                - 若未提 Glasgow，但提 "any UK university" 或 "UK institutions" → 允許進 B
                                                                - 若完全只提某校且該校不在你的 target list → 才 C

**具體修改**：

```yaml
hard_reject_rules:
 - id: "E-NONTARGET-001"
    name: "Non-Target and Non-Portable"
    stage: "eligibility"
    description: "Not from Glasgow and not explicitly portable to UK universities"
    when:
      is_directory_page: false
      # Only apply this rule to university sources; do NOT apply to foundation/government sources.
      source_type:
        any_of: ["university"]
      not_any_regex:
    - "(?i)gla\\.ac\\.uk"
    - "(?i)glasgow\\.ac\\.uk"
    - "(?i)University\\s*of\\s*Glasgow"
    - "(?i)\\bGlasgow\\b.*scholarship"
    - "(?i)any\\s+university"
    - "(?i)any\\s+UK\\s+universit(y|ies)"
    - "(?i)international\\s+students?\\s+at\\s+UK\\s+universit(y|ies)"
    action:
      bucket: "C"
      reason: "Not from Glasgow and not portable to UK universities"
```

### 8.3 403/429：你現在只把它當 link health，但它其實是「覆蓋率殺手」

**改法**：

- 加一個 retry_policy（每網域）：
                                                                - 429：指數退避 + 降低併發
                                                                - 403：換 UA / 增加瀏覽器 fallback（對於 listing page 特別重要）

**新增規則**：

```yaml
positive_scoring_rules:
 - id: "P-INTL-FEE-002"
    name: "International Fee Status Wording"
    stage: "scoring"
    description: "Common UK phrasing for international eligibility"
    when:
      any_regex:
    - "(?i)international\\s+fee\\s+status"
    - "(?i)overseas\\s+fee\\s+status"
    - "(?i)international\\s+fee\\s+students?"
    action:
      score_add: 25
      reason: "Uses international/overseas fee status wording (often equivalent to 'international students')."
```

## Phase 9: Workflow v2（讓你真的"每週都產出"，而不是靠手動 push）

### 9.1 把 workflow 檔案合併到 default branch

**檔案**：`.github/workflows/scholarshipops-search.yml`

- schedule 事件以 default branch 為準。

### 9.2 用 workflow_dispatch inputs 做兩種模式

**檔案**：`.github/workflows/scholarshipops-search.yml`

- `mode: focused | wide`
                                                                - focused：只跑 Tier1 + 少量 Tier2（快、穩）
                                                                - wide：跑 Tier1 + Tier2 + 開啟 discovery（crawl depth + sitemap），用 matrix 切分到多個 jobs（每個 job 控制 <6h）

**具體修改**：

```yaml
on:
  push:
    branches:
   - feature/ScholarshipOps
  schedule:
  - cron: "10 13 * * 3"
  workflow_dispatch:
    inputs:
      mode:
        description: "focused = Tier1+Tier2 only, wide = enable discovery + broader sources"
        required: true
        default: "focused"
        type: choice
        options:
     - focused
     - wide
      shards:
        description: "Number of shards (wide mode only)"
        required: true
        default: "8"

jobs:
  prepare:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.mk.outputs.matrix }}
      shard_total: ${{ steps.mk.outputs.shard_total }}
      mode: ${{ steps.mk.outputs.mode }}
    steps:
   - id: mk
        run: |
          MODE="${{ github.event.inputs.mode || 'focused' }}"
          if [ "$MODE" = "wide" ]; then
            SHARDS="${{ github.event.inputs.shards || '8' }}"
          else
            SHARDS="1"
          fi
          # Build JSON array [0..SHARDS-1]
          ARR=$(python3 - << 'PY'
import os, json
shards=int(os.environ["SHARDS"])
print(json.dumps(list(range(shards))))
PY
          )
          echo "mode=$MODE" >> "$GITHUB_OUTPUT"
          echo "shard_total=$SHARDS" >> "$GITHUB_OUTPUT"
          echo "matrix={\"shard\":$ARR}" >> "$GITHUB_OUTPUT"
        env:
          SHARDS: ${{ github.event.inputs.shards || '8' }}

  search:
    needs: [prepare]
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix: ${{ fromJson(needs.prepare.outputs.matrix) }}
    timeout-minutes: 340  # stay under hosted-runner limits
    steps:
   - uses: actions/checkout@v4
      # ... existing steps ...
   - name: Run search
        working-directory: scripts/search_scholarships
        env:
          TZ: Asia/Taipei
          ROOT: ${{ github.workspace }}
          MODE: ${{ needs.prepare.outputs.mode }}
          SHARD_INDEX: ${{ matrix.shard }}
          SHARD_TOTAL: ${{ needs.prepare.outputs.shard_total }}
        run: cargo run --release
```

**Rust-side 最小變更**：

- 當讀取 sources.yaml 時，根據穩定 hash（例如 sha1(url) % SHARD_TOTAL == SHARD_INDEX）分割 sources。
- 在 MODE=wide 時，啟用：
                                                                - 更深的內部爬取（UofG `/scholarships/…` allow-path）
                                                                - 更廣泛的第三方索引遍歷（但限制每個網域的頁數）

## 預期成果（v2 patch 後）

- **Glasgow scholarships count** 應該大幅增加，因為 `/scholarships/all/` 現在實際會被渲染。
- **你的 crawl 不會再「安靜地不執行」schedule**（假設 workflow 檔案已合併到 default branch）。
- **Wide mode** 會將總抓取頁面增加（大約）shard count ×，而不會超過單一 job 的時間限制。
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

    - id: p5-matrix-strategy

content: GitHub Actions Matrix 分片：修改 search.yml，將 sources 分成 10 片，每片產生 leads.part-*.jsonl，最後 merge job 合併

status: pending

    - id: p5-two-stage-workflow

content: 分兩段 Workflow（替代方案）：實作 discovery job（只抓 index/sitemap）和 fetch_extract job（處理 queue）

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

#### 4.1 新增 UK 大學官方索引頁

**檔案**：`tracking/sources.yml`

**變更**：

- 新增 Russell Group 大學的 scholarships index pages：
                - Edinburgh: `https://www.ed.ac.uk/student-funding/postgraduate/international`
                - St Andrews: `https://www.st-andrews.ac.uk/study/fees-and-funding/postgraduate/`
                - Strathclyde: `https://www.strath.ac.uk/studywithus/scholarships/`
                - Manchester: `https://www.manchester.ac.uk/study/international/finance/scholarships/`
                - 等（共 10-15 個）
- 每個 source 配置 `index_urls` 和 `detail_url_regex`

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
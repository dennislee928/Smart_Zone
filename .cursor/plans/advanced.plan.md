# cursor.plan.md — Smart_Zone Scholarship Search Accuracy Upgrade

Date: 2026-01-21

Goal: Produce a high-precision, low-noise shortlist of scholarships that I can apply for NOW (and within the next 30/60 days) for University of Glasgow MSc Software Development (2026/27 intake).

---

## 0) Current State (Observed Symptoms)

- Latest production run yields:

        - Bucket A/B: 0 items

        - Many rejected items, many “deadlinks”, and misleading rule reasons.

- Root causes (confirmed in code):

1) Synthetic lead generation in scrapers (university/government) creates non-existent URLs.

2) Rules evaluation uses OR across conditions (should be AND).

3) Deadline parsing uses `lead.deadline` instead of `lead.deadline_date`.

4) Source list is massively over-broad, causing high failure rates and low signal.

5) Link health checker treats transient blocks as dead; HEAD-only checks produce false negatives.

6) Canonicalization/dedup is too weak; UTM/query variants inflate duplicates.

Definition of “apply today”:

- Open now AND application deadline is within:

        - “Apply now”: <= 30 days

        - “Prepare”: 31–90 days

        - “Watch”: > 90 days or unknown but recurring annually

---

## 1) Target Operating Model (What “Good” Looks Like)

Outputs per run:

1) `triage.md`

            - Section A: “Apply today / next 30 days”

            - Section B: “Prepare (31–90 days)”

            - Section C: “Watchlist (deadline unknown/annual cycle)”

            - Each row includes: Name, Amount, Deadline (date + timezone), Eligibility summary, Link, Confidence score, Reason tags

2) `source_health.md`

            - Status distribution by domain (200/3xx/403/429/5xx)

            - Top failing domains with recommended action (retry/backoff/js/manual)

3) `deadlinks.md`

            - Only true 404/410 after GET verification and retries (not 403/429)

Key KPI:

- Precision: >= 70% of Bucket A links are real, open, and relevant.

- Recall: Glasgow + portable scholarships are consistently discovered.

- Stability: repeated runs produce comparable results (no random placeholder drift).

---

## 2) Immediate Hotfixes (Do these first)

### 2.1 Fix Rules Engine Semantics (CRITICAL)

File: `scripts/search_scholarships/src/rules.rs`

Problem:

- Current `matches_rule()` returns true if ANY single condition matches.

- This breaks triage integrity.

Fix:

- Treat rule as conjunction (AND):

        - If a condition is specified, it must pass.

        - `any_regex` is OR within that sub-condition.

        - `not_any_regex` means “none of these regex should match”.

        - Deadline/status/money constraints must all pass if present.

Also:

- Return structured “matched_fields” so reports can explain WHICH condition triggered.

Acceptance:

- Add unit tests for:

        - Rule with both deadline + regex (must require both)

        - Rule with http_status + not_any_regex

        - A lead that matches regex but fails deadline must not trigger

---

### 2.2 Fix Deadline Parsing

Files:

- `scripts/search_scholarships/src/rules.rs`

- `scripts/search_scholarships/src/triage.rs`

Fix:

- Use `lead.deadline_date: Option<String>` as primary.

- Only fallback to parsing `lead.deadline` when date missing.

- Standardize deadline to ISO date `YYYY-MM-DD` in UTC or local timezone note.

Acceptance:

- “Apply today” section correctly populates when a lead has a deadline_date.

---

### 2.3 Stop Generating Fake Leads

Files:

- `scripts/search_scholarships/src/scrapers/university.rs`

- `scripts/search_scholarships/src/scrapers/government.rs`

- `scripts/search_scholarships/src/scrapers/third_party.rs`

Fix:

- Remove placeholder scholarship entries (guessed paths, “estimated” deadlines) OR mark them as:

        - `lead.confidence = 0.2`

        - `lead.tags += ["catalog_only"]`

        - and DO NOT place into Bucket A/B without confirming via real page fetch.

Acceptance:

- `deadlinks.md` volume drops sharply.

- Buckets contain mostly real pages extracted from HTML.

---

## 3) Rebuild the Source Strategy (High Signal, Low Noise)

### 3.1 Replace “1000+ sources” with Tiered Seeds

File: `tracking/sources.yml`

Create three tiers:

Tier 1 (must-have; Glasgow-specific):

- https://www.gla.ac.uk/scholarships/globalleadershipscholarship/

- https://www.gla.ac.uk/scholarships/search/

- https://www.gla.ac.uk/scholarships/all/

- https://frontdoor.spa.gla.ac.uk/ScholarshipApp/ (portal listing)

Tier 2 (portable external scholarships relevant to UK masters):

- Rotary Global Grant official pages (district + TRF)

- Major international scholarship bodies you already track (only if eligibility fits)

Tier 3 (optional exploration):

- Only add when Tier 1/2 are stable.

- NEVER auto-generate hundreds of unrelated universities.

Action:

- Deprecate `scripts/generate_sources_complete.py` for production use.

- Keep it as “research mode” only.

Acceptance:

- Total sources: 20–80, not 800+.

- Crawl completion rate improves and you get usable A/B items.

---

## 4) Implement Real Glasgow Scraping (No JS Guessing)

### 4.1 Scholarship Index Extraction

Implement a Glasgow-specific scraper that:

- Fetches `gla.ac.uk/scholarships/all/` and extracts scholarship detail links

- Fetches `gla.ac.uk/scholarships/search/`:

        - If it’s JS-driven, use a JS-capable strategy:

                - Option A: Selenium/Playwright only for this domain + path allowlist

                - Option B: Discover the underlying JSON endpoint via network inspection and call it directly

Store:

- Title

- Amount

- Deadline date

- Eligibility: country/fee status/level/programme

- Official “Apply” link

Acceptance:

- Run produces Glasgow scholarships with correct deadlines and eligibility text.

---

## 5) Fix Link Health (Reduce False “Deadlinks”)

File: `scripts/search_scholarships/src/link_health.rs`

Improvements:

1) Use GET fallback:

            - If HEAD is 403/405/429/5xx → try GET (lightweight, small range).

2) Retries + exponential backoff:

            - Respect `Retry-After` when status 429/503 (if present).

            - Limit retries per domain.

3) Classify statuses:

            - Dead: 404/410 after GET verification

            - Transient: 403/429/500/503 (retry policy)

            - Redirect: 301/302 (follow to canonical)

Acceptance:

- `deadlinks.md` contains mostly 404/410 only.

- `source_health.md` distinguishes “blocked” vs “dead”.

---

## 6) Canonicalization + Dedup (Stop Counting Variants)

Files:

- `scripts/search_scholarships/src/storage.rs` (or new `normalize.rs`)

- `scripts/search_scholarships/src/sorter.rs`

Normalization rules:

- Force https

- Lowercase host

- Remove tracking params: utm_*, gclid, fbclid, session ids

- Normalize trailing slash

- Follow redirects and store final URL as canonical (when safe)

Dedup key:

- canonical_url OR (title + sponsor + country + deadline_date)

Acceptance:

- Duplicate leads drop.

- Same scholarship appears once with best link.

---

## 7) Profile/Criteria Filtering That Matches Reality

File: `tracking/criteria.yml`

Current issues:

- `max_gpa_requirement` is used incorrectly (should compare requirement to USER GPA, not to an arbitrary ceiling).

- `required_keywords` is too strict and removes relevant items.

Fix model:

- Add user profile fields:

        - `user_undergrad_gpa`

        - `user_grad_gpa` (if applicable)

        - `target_university = "University of Glasgow"`

        - `target_programme_keywords = ["Software Development", "Computing Science", "Computer Science"]`

- Filter logic:

        - Only disqualify when scholarship explicitly requires undergraduate GPA > user_undergrad_gpa (and the scholarship states it clearly).

        - Otherwise, keep and set `needs_manual_check`.

Acceptance:

- Glasgow scholarships aren’t filtered out by keyword/GPA misconfig.

---
## 7.1. 修改 third_party.rs：強制加入「留學」相關關鍵字
目前的搜尋可能太過廣泛（例如只搜 "Software Development Scholarship"），這會導致搜尋引擎回傳大量美國當地的獎學金。

請將 third_party.rs 中的搜尋關鍵字生成邏輯（通常在 build_search_queries 或類似函式中）修改如下。這會強制搜尋引擎尋找包含「英國」、「國際學生」或「留學」字眼的結果。

建議的修改邏輯 (Rust 偽代碼/範例)：

Rust
// 在 third_party.rs 中找到生成 queries 的地方

pub fn generate_queries(profile: &Profile) -> Vec<String> {
    let mut queries = Vec::new();
    
    // 核心關鍵字
    let subjects = vec!["Software Development", "Computer Science", "Data Science"];
    
    for subject in subjects {
        // 策略 A: 明確指定地點 (這是排除純美國獎學金最有效的方法)
        // 這樣搜出來的美國基金會，通常都是提供 "Study Abroad" 資金的
        queries.push(format!("{} scholarship UK", subject));
        queries.push(format!("{} scholarship Scotland", subject));
        queries.push(format!("{} scholarship University of Glasgow", subject));
        
        // 策略 B: 針對身分 (強制包含國際學生)
        queries.push(format!("{} scholarship for international students", subject));
        queries.push(format!("{} scholarship for Taiwanese students", subject));
        
        // 策略 C: 針對學位 (排除大量給高中生的當地獎學金)
        queries.push(format!("{} master's scholarship", subject));
        queries.push(format!("{} MSc funding", subject));
    }
    
    // 策略 D: 負面關鍵字 (在 Query 層級就過濾掉雜訊)
    // 許多搜尋引擎支援減號排除法
    let negative_keywords = " -\"US residents only\" -\"high school\" -\"undergraduate\"";
    
    // 將負面關鍵字附加到所有查詢後
    let final_queries: Vec<String> = queries.iter()
        .map(|q| format!("{}{}", q, negative_keywords))
        .collect();

    final_queries
}
2. 優化 rules.yaml：調整「可攜帶性 (Portability)」的判斷
既然您願意接受美國基金會的資助（只要能帶去英國用），我們需要確保規則引擎不會因為「看起來像美國機構」就直接殺掉它。

請檢查並微調您的 E-RESIDENCY-001 (US Residency Gate) 規則。目前的規則有點太「一刀切」了。

建議修改 rules.yaml 中的 E-RESIDENCY-001：

將原本單純的 regex 判斷，加上一個 例外條件 (unless)。如果內文明確提到了 "UK" 或 "International"，即使它來自美國基金會，也先留下來人工確認。

YAML
  # === US Residency Gate (Revised) ===
  - id: "E-RESIDENCY-001"
    name: "US State/County Residency Required"
    stage: "eligibility"
    description: "Requires US state or county residency"
    when:
      # 同時滿足以下兩點才踢除：
      all_of:
        # 1. 看起來像是有美國居住限制
        - any_regex:
            - "(?i)resident\\s+of\\s+(the\\s+)?state\\s+of"
            - "(?i)must\\s+be\\s+a\\s+resident\\s+of"
            - "(?i)high\\s+school\\s+seniors?" # 高中生通常是當地獎學金
            - "(?i)\\bWashington\\s+State\\b"  # 針對您遇到的西雅圖案例
        
        # 2. 且「沒有」提到可以去英國或國際使用 (這是保護機制)
        - not_any_regex:
            - "(?i)study\\s*abroad"
            - "(?i)international\\s*study"
            - "(?i)tenable\\s*at\\s*any\\s*university"
            - "(?i)United\\s*Kingdom"
            - "(?i)\\bUK\\b"
            - "(?i)\\bGlasgow\\b"
    action:
      bucket: "C"
      reason: "US residency requirement detected without international portability."
___

## 8) Scoring + Bucketing (Make “Apply Today” Useful)

Add fields to `Lead`:

- `confidence: f32` (0–1)

- `deadline_confidence: f32`

- `eligibility_confidence: f32`

- `tags: Vec<String>`

Bucket rules:

- A (Apply today): deadline <= 30 days AND confidence >= 0.7

- B (Prepare): 31–90 days OR confidence 0.5–0.7

- Watchlist: deadline unknown/annual or confidence < 0.5

Acceptance:

- You always get a concise “do now” list when opportunities exist.

---

## 9) Tests + Fixtures (Prevent Regressions)

Add fixtures:

- Saved HTML for Glasgow Global Leadership Scholarship page

- Saved HTML for scholarships/all or portal listing

Tests:

- rules engine AND semantics

- deadline parsing

- canonicalization

- link health classification

Command:

- `cargo test -p search_scholarships`

Acceptance:

- CI-level confidence that the pipeline remains accurate.

---

## 10) “What can I apply today?” Generator

Add output section in `triage.md`:

- `## Apply Today (Top 10)`

        - Only A bucket

        - Sorted by (deadline soonest, highest confidence)

Also export:

- `apply_today.csv` for quick filtering on phone.

Acceptance:

- One glance tells you what to submit immediately.

---

## Execution Order (Recommended)

Day 1:

1) Fix rules AND semantics + tests

2) Fix deadline_date usage

3) Remove fake leads (or mark as catalog_only)

4) Narrow sources.yml to Tier 1 (Glasgow)

Day 2:

5) Implement Glasgow index extraction (all/search/portal)

6) Link health: GET fallback + retry/backoff

7) Canonicalization + dedup

Day 3:

8) Scoring + bucketing + apply_today.csv

9) Add fixtures for Glasgow pages and regression tests

---

## Definition of Done

- Bucket A contains at least 1–5 real scholarships when they exist.

- 80%+ of A links open correctly in browser.

- Deadlinks list is mostly true 404/410.

- Re-running produces consistent results with stable reasons.
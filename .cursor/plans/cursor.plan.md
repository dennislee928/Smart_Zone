# cursor.plan.md — ScholarshipOps GitHub Action 清單「快速篩選規則」(超詳細版)

## 1) 目的與成果定義 (Goal / Outcome)

### 1.1 目的

把 GitHub Action 產出的 `report.md`（欄位：Name/Amount/Deadline/Score/Match/ROI/Urgency/Source/Days Until）轉成：

- **A：主攻（高機率+高回報+時程相容）**

- **B：備援（可申請但不優先/或需要補資料驗證）**

- **C：淘汰（資格不符、時程不符、可信度不足、或成本過高）**

並且對每一筆給出「可機器判斷」且可追溯的理由。

### 1.2 成果輸出 (Deliverables)

必備輸出（每次 workflow run 都產生）：

1. `triage.md`：A/B/C 分桶清單（附理由與下一步）

2. `triage.csv`：可排序/可丟回 spreadsheet 的資料表

3. `deadlinks.md`：死鏈/跳轉/403 清單（供修正來源/規則）

4. `rules.audit.json`：本次套用的規則版本、命中規則、分數明細（可除錯）

可選輸出：

- `triage.diff.md`：與上次 run 的變更（新增/移除/狀態變更）

- `watchlist.md`：尚未開放/截止未公告/需等待「開放日」的獎學金追蹤表

---

## 2) 輸入資料與使用者 Profile (Inputs / Profile)

### 2.1 輸入

- `report.md`：ScholarshipOps Search Report（GitHub Action 產出）

    - 欄位：`Name | Amount | Deadline | Comprehensive Score | Match | ROI | Urgency | Source | Days Until`

### 2.2 使用者 Profile（可參數化）

用 `profile.yaml` 管理，避免規則寫死；Cursor 改這個就能換人用。

**profile.yaml (範例)**

- target_intake: "2026/27"

- expected_start_month: "2026-09"

- degree_level: "postgraduate_taught"   # PGT / master taught

- country_of_citizenship: ["Taiwan"]

- country_of_residence: ["Taiwan"]

- target_university: ["University of Glasgow", "Glasgow"]

- target_program_keywords:

    - "Software Development"

    - "Computer Science"

    - "Computing"

    - "Software Engineering"

- hard_exclusions:

    - "undergraduate_only"

    - "phd_only"

    - "home_fee_status_only"

    - "care_leaver_only"

    - "refugee_only"

- preferred_currencies: ["GBP", "USD", "EUR"]

- min_award_gbp_equiv_for_A: 2000

- max_application_effort_hours_for_A: 6

- latest_acceptable_deadline: "2026-08-31"  # 入學前可合理完成/出結果的上限（可調）

---

## 3) 整體流程設計 (Pipeline)

### 3.1 Stage 0 — Parse & Normalize（資料解析與標準化）

**目標**：把 `report.md` 的每一列變成結構化物件 `ScholarshipItem`，並補齊可計算欄位。

- Parse Markdown table → list[ScholarshipItem]

- Normalize:

    - amount：解析幣別/最小最大值（`£3,000 - £5,000`、`Up to £10,000`、`Full tuition`）

    - deadline：轉 ISO date（缺失→`null` + `deadline_status=unknown`）

    - days_until：若 report 已給，做一致性校驗；不一致以 deadline 重算

    - source_url：基本 canonicalize（去 UTM、去結尾斜線統一）

**輸出**：`items.normalized.json`

---

### 3.2 Stage 1 — Link Health & Canonical Source（連結健康檢查）

**目標**：先把「不存在/不可訪問/疑似模板」的條目降權或淘汰，避免後面浪費算力。

對每個 `source_url`：

- HTTP 狀態：

    - 200/206：OK

    - 301/302：追蹤 1–3 次後 canonicalize

    - 403/429：標記 `rate_limited_or_blocked`

    - 404/410：`dead_link`

    - 5xx：`server_error`

- Content sanity（輕量）：

    - 檢查頁面是否包含 scholarship 常見關鍵字（例如 "scholarship", "funding", "tuition", "eligible", "deadline"）

    - 若內容極短/只剩導覽/或是 generic landing page → `thin_or_generic_page`

**規則建議**

- `dead_link` → 直接 C（除非 Name 是你已知的校內獎學金且 URL 可能改版，可進 watchlist）

- `rate_limited_or_blocked` → B（待重試）+ 建議加 backoff

- `thin_or_generic_page` → B（需要人工或更深層抓取）

**輸出**：`items.linkchecked.json`, `deadlinks.md`

---

### 3.3 Stage 2 — Source Trust（來源可信度評分）

**目標**：把「官方校方/政府/權威機構」跟「聚合站/內容農場/可疑站」分開，避免被 SEO 垃圾污染。

#### 3.3.1 Domain 分級（可配置）

在 `trust.domains.yaml` 定義：

- Tier S：學校官方域名（如 `.ac.uk`, `.edu`）、政府（`.gov`）、知名國際組織官方域

- Tier A：大型基金會/慈善機構官方域名、主要媒體的 funding page

- Tier B：可信聚合（可用但需回到原始來源核實）

- Tier C：不明聚合/個人博客/內容站（預設淘汰或強降權）

#### 3.3.2 Scam / Dark Pattern 偵測（關鍵字）

命中任一條，直接 C 或至少 `risk_flag=true`：

- 要求「先付款」才能申請/拿名單/保留名額

- 要你提供信用卡/銀行帳號「先扣款」或「驗證身份」

- “guaranteed”, “money back”, “exclusive list”, “act now” 類型文案

- URL 與品牌不一致（例如看似某大學獎學金但域名不是該校）

- 下載可疑可執行檔（`.exe`, `.dmg`）作為申請必需

**輸出**：`items.trust.json`（欄位：trust_tier, trust_score, risk_flags[]）

---

### 3.4 Stage 3 — Eligibility Gate（硬資格門檻：一票否決）

**目標**：先做「硬淘汰」，再做「軟排序」。

#### 3.4.1 學位層級 Gate（你的目標是 PGT/Master taught）

直接 C：

- `undergraduate only`

- `PhD only / doctoral only`

- `postdoc`、`faculty`、`staff only`

- `foundation year`、`pre-sessional` only

#### 3.4.2 Fee Status Gate（英國獎學金常見硬門檻）

直接 C（或至少強降權到 B/C）：

- `Home fee status only`

- `UK fee status required`

- `RUK only`

- `EU/home only`（視條文，若明確排除 international 就淘汰）

- `must be ordinarily resident in UK`（通常等同排除）

> 重要：Fee status 文字表述很多變體，請用同義詞字典 + regex。

#### 3.4.3 身分/族群 Gate（與你無關的定向補助）

直接 C（除非你確定符合且願意走該路線）：

- `care leaver`

- `asylum seeker / refugee / sanctuary`

- `armed forces covenant / veteran / service leaver`

- `local council residents only`

- `domiciled in Scotland/England/Wales`（若限定 domicile）

#### 3.4.4 國籍/居住地 Gate

直接 C：

- 限定國籍不包含 Taiwan（或你居住地不符）

- “available only to citizens of …” 且不包含你

不確定 → B（需要抓取 eligible countries 清單或 FAQ）

**輸出**：`items.eligibility.json`（欄位：hard_fail_reasons[]）

---

### 3.5 Stage 4 — Timeline Gate（時程相容性）

**目標**：把「看起來很好但時間點不可能」的淘汰，特別是入學年與出結果時間不匹配。

定義幾個時間狀態：

- `deadline_passed`：deadline < today → C

- `deadline_missing`：無 deadline → B（進 watchlist 或要求補抓）

- `deadline_far_future`：> expected_start_month + 30d → 多數情況 C（除非是入學後才可申請的 bursary）

- `deadline_close`：< 21 days → Urgency 加權，但若 effort 高則可能直接 B（來不及）

- `results_after_start`：若頁面可抓到 “award announced in …” 且晚於開學 → B/C

**你這個案子的預設**

- 你是 2026/09 入學：建議 A 類 deadline 上限先抓 `2026-08-31`（可調）

---

### 3.6 Stage 5 — Effort & Friction（申請成本估算）

**目標**：同樣 £5,000，有的 30 分鐘搞定、有的要三封推薦信+作品集+面試；要把「成本」量化進排序。

建立 `effort_score`（0–100，越高越累）：

- 0–20：auto-considered（自動審），或只要填一個短表

- 20–40：1 篇 essay（<= 500 words）+ 基本文件

- 40–60：2–3 篇 essay、推薦信、作品集、或需要面試

- 60–80：考試/測驗、提案書、需校內提名、或多階段審查

- 80–100：需要 membership、需付費活動、或需在地機構背書

對 `effort_score` 加上加權因子：

- 有模板可重用（例如你已寫好的 leadership/impact essay）→ effort 減 10

- 需要影片/作品集（你還沒 ready）→ effort 加 10–20

---

### 3.7 Stage 6 — Expected Value Scoring（期望值模型）

**目標**：把「金額」與「成功率」與「成本」合成單一排序。

#### 3.7.1 建議分數構成

- `award_value_score`：依 GBP 等值（Full tuition 直接給高分）

- `probability_score`：用 proxy（來源可信+資格明確+過往錄取數/名額+你的 match）

- `timeline_score`：越接近且合理越高

- `effort_penalty`：effort 越高扣越多

- `risk_penalty`：scam/不明條文扣分

**總分**

`final_score = award_value_score * 0.35 + probability_score * 0.35 + timeline_score * 0.2 - effort_penalty * 0.2 - risk_penalty * 0.4`

> 注意：這不是學術模型，是「工程可運作」的 triage 模型；權重要用你的實際申請結果回饋校正。

---

## 4) 規則庫 (Rules Library) — 可直接實作的「可配置」設計

### 4.1 規則資料結構

用 YAML 定義規則，讓你不改 code 也能調整。

**rules.yaml (概念)**

- id: "E-FEE-001"

type: "hard_reject"

stage: "eligibility"

when:

any_regex:

            - "(?i)home fee status"

            - "(?i)UK fee status"

            - "(?i)RUK"

            - "(?i)home students? only"

action:

bucket: "C"

reason: "Requires Home/UK fee status; not applicable to international/Taiwan applicants."

- id: "E-LEVEL-UG-001"

type: "hard_reject"

stage: "eligibility"

when:

any_regex:

            - "(?i)undergraduate (only|students?)"

            - "(?i)bachelor('?s)? (only|students?)"

action:

bucket: "C"

reason: "Undergraduate-only scholarship."

- id: "T-DEAD-PAST-001"

type: "hard_reject"

stage: "timeline"

when:

deadline:

lt_today: true

action:

bucket: "C"

reason: "Deadline passed."

- id: "S-SCAM-001"

type: "hard_reject"

stage: "trust"

when:

any_regex:

            - "(?i)credit card"

            - "(?i)bank account"

            - "(?i)processing fee"

            - "(?i)guaranteed"

            - "(?i)money back"

action:

bucket: "C"

reason: "Scam-like language / payment or bank details requested."

### 4.2 Keyword Dictionaries（同義詞字典）

建立 `dicts/*.txt`（中英混合）提高命中率。

#### 4.2.1 Fee status（排除）

- "home fee status"

- "UK fee status"

- "RUK"

- "home students only"

- "domiciled in"

- "ordinarily resident in the UK"

- "settled status"

- "UK residents only"

#### 4.2.2 Degree level（排除）

- "undergraduate"

- "bachelor"

- "foundation year"

- "doctoral"

- "PhD"

- "postdoctoral"

- "research fellowship"

#### 4.2.3 Special identity（排除）

- "care leaver"

- "sanctuary scholarship"

- "refugee"

- "asylum"

- "armed forces"

- "veteran"

- "local authority"

#### 4.2.4 Include signals（加分）

- "international students"

- "postgraduate taught"

- "master's programme"

- "self-funded"

- "tuition fee discount"

- "fee waiver"

- "University of Glasgow"（或你的 target 校名）

- "School of Computing"

- "Computer Science"

- "Software"

---

## 5) 分桶邏輯（A/B/C）— 明確且可解釋

### 5.1 C：淘汰 (Hard fail)

任何一項成立：

- dead_link 且不可修復

- 命中 hard_reject（資格/費籍/身分/學位層級）

- 明顯詐騙/要付款/要銀行資料

- deadline 已過

- 只限非你國籍/非 international

### 5.2 B：備援 (Needs verification / medium priority)

成立任一項：

- deadline missing / “opens later” → watchlist

- 403/429 被擋 → 重試與降頻

- eligibility 模糊（需要抓 “eligible countries” 清單）

- effort 高但獎金也高（先備用）

### 5.3 A：主攻 (High priority)

同時滿足：

- trust_tier ∈ {S, A}

- 無 hard_fail

- deadline 在你的可行窗口內

- award >= min_award_gbp_equiv_for_A（或 Full tuition）

- effort_score <= max_application_effort_hours_for_A（或可重用既有材料）

- 明確適用 international / 或明確包含 Taiwan

---

## 6) GitHub Actions 實作建議（工程化）

### 6.1 Workflow 建議

- schedule：每週 1–2 次（避免被 429）

- job 分段：

1) parse_report

2) link_check (with rate limit, retry)

3) fetch_light_content (only for Tier S/A)

4) apply_rules + scoring

5) render outputs + upload artifact

### 6.2 Rate limiting / politeness

- 併發數限制（例如同時最多 5）

- 對同域名設限（domain-level throttle）

- 遇到 429：指數退避（1m, 3m, 10m）

- cache：ETag/Last-Modified 省流量

### 6.3 Debuggability

- 每條 item 保留：

    - matched_rules[]

    - hard_fail_reasons[]

    - trust_tier, risk_flags[]

    - score_breakdown（award/probability/timeline/effort/risk）

- 讓你可以對「為什麼被淘汰」一眼看懂

---

## 7) 測試案例（必要，否則規則會越改越亂）

建立 `tests/fixtures.json`，覆蓋至少：

1) "Home fee status only" → C

2) "International students" + "postgraduate taught" → A/B（視 deadline/金額）

3) 404 dead link → C

4) 429 blocked → B + retry

5) "processing fee" / "credit card" → C

6) deadline missing → B/watchlist

7) "PhD only" → C

8) 國籍限定不包含 Taiwan → C

每次改 rules 都跑 unit tests，確保不回歸。

---

## 8) Cursor 任務清單（逐步落地）

### 8.1 第 1 迭代（可在 1 次 Sprint 完成）

- [ ] 實作 parser：`report.md` → items.json

- [ ] 實作 link health check → deadlinks.md

- [ ] 實作 rules engine（yaml 驅動）→ triage.csv/triage.md

- [ ] 先只用「文字規則」（不抓全文）完成 A/B/C 分桶

### 8.2 第 2 迭代（提升準確度）

- [ ] 對 Tier S/A 做 light fetch（抓出 eligible/deadline/level）

- [ ] 將 “eligible countries/territories change each year” 類型改成 watchlist 規則（避免誤判）

- [ ] 新增 domain whitelist/blacklist

- [ ] 新增 `triage.diff.md`（追蹤狀態變更）

### 8.3 第 3 迭代（逼近半自動申請管理）

- [ ] 對 A 類自動產出「申請 checklist」（文件/字數/推薦信/是否面試）

- [ ] 產出 `calendar.ics`（deadline/提醒）

- [ ] 對每個 A 類生成 `application_brief.md`（可直接丟進你的資料室/Obsidian）

---

## 9) 你應該先設定的預設閾值（避免卡住）

- A 類最低獎金：£2,000（或 tuition discount >= 10%）

- A 類最高 effort：<= 6 小時（超過先進 B）

- deadline window：<= 2026-08-31（之後多數不利於 2026/09 入學前完成）

- trust：非 Tier S/A 的一律先進 B（除非可回推到官方原始來源）

---

## 10) 一句話版本（寫在 triage.md 開頭）

「先用硬門檻（費籍/學位層級/身分/國籍/死鏈/詐騙）砍掉 60–80%，再用時程+期望值模型排序剩下的 20–40%，把 A 類控制在 5–15 筆，才是可持續的申請管線。」
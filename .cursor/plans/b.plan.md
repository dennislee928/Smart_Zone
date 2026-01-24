The run effectively produced one “lead” in the database and it’s Chevening 2027/28 with a deadline 2026-11-03. 

report

The rules audit shows only 3 items were processed, and 2 were rejected by “Taiwan Not Eligible”. 

rules.audit

This combination usually means one (or more) of the following is happening:

Your source list is too small / too narrow, so only a handful of pages are being crawled.

The crawler is skipping/losing most sources upstream (timeouts, JS-only pages, 403/429), so they never become “items processed.”

Your filter gates are overly aggressive early in the pipeline, so leads are eliminated before they reach triage.

Your “target intake” rule is missing, so irrelevant-cycle scholarships (Chevening 2027/28) slip through while relevant Glasgow 2026/27 items never appear. (Chevening for 2027/28 is not actionable for starting Glasgow in Sep 2026.) 

report

How to expand high-quality scholarship resources (without exploding noise)
Principle 1: Separate “Discovery Sources” from “Canonical Sources”

Discovery sources are databases/index pages (FindAMasters, Study UK, Prospects, etc.). They are useful to find names and links, but not authoritative.

Canonical sources are the official pages where you extract deadline/eligibility/value.

Your pipeline should:

Crawl discovery → extract candidate scholarship detail URLs

Crawl canonical → extract structured fields + score/triage

This avoids ranking directory pages as if they were scholarships.

Tiered expansion strategy (high signal first)
Tier 0 — Always-on “must crawl” (Glasgow + your exact target)

Add these first. They should dominate your recall and they are stable:

University of Glasgow scholarship index (all scholarships)

University of Glasgow Global Leadership Scholarship (2026/27)

If Tier 0 doesn’t yield multiple candidates, do not expand—fix the scraper first.

Tier 1 — Official UK-level scholarship gateways (portable + authoritative)

These tend to link to official schemes and participating universities:

Study UK (British Council) scholarships & funding hub

GOV.UK postgraduate funding overview (not scholarships, but helps for “what exists” classification)

These are “discovery + guidance” sources. Use them to seed canonical pages.

Tier 2 — High-quality discovery databases (use as link generators, not truth)

Use these to expand breadth without adding thousands of random domains:

FindAMasters funding guides / scholarship listings

Prospects postgraduate funding guide

PostgraduateStudentships.co.uk (directory)

LSE careers page explicitly recommends specific databases (good “meta” validation)

Implementation detail: discovery pages should produce “outbound scholarship URLs” that you then canonicalize and crawl.

Tier 3 — Shortlisted UK universities (scholarship index pages only)

Instead of “all UK universities,” pick 10–20 highly international institutions and crawl only:

/scholarships/, /funding/, /student-funding/ index pages

plus their international postgraduate scholarship pages

Example approach:

Start with Russell Group institutions, but only add index pages (not guessed paths).

For each uni, you want one seed URL that lists scholarships, then extract detail links.

(You can generate the list programmatically, but keep the first iteration manual and curated.)

The single biggest upgrade: “Source Pack” templates

Create source packs in tracking/sources.yml like:

pack: glasgow_core

pack: uk_official

pack: discovery_databases

pack: uk_uni_shortlist

Then your pipeline can run in modes:

mode=precision: glasgow_core + uk_official

mode=balanced: + discovery_databases

mode=explore: + uk_uni_shortlist (capped, throttled)


___
You are not “resource-poor”; you are **being filtered/collapsing upstream**.

From what you shared:

* `sources.yml` already has a reasonable Tier-1/Tier-2 set. 
* `rules.yaml` contains **two hard-reject rules that will aggressively wipe out most candidates** unless your extractor populates fields perfectly. 

That is why you end up with **A=0 / B=1 / C=2**: you are not expanding into “high-quality scholarships”; you are **not converting pages → valid lead objects**, and then your rules hard-reject “unknowns”.

Below is a concrete plan: fix the two rule pitfalls, then expand *high-quality* sources mainly by **mining the Glasgow pages you already have** (this yields 10–100 high-signal scholarship detail URLs without adding noisy domains).

---

## 1) Why your buckets are tiny (root causes in your config)

### A. `E-COUNTRY-001` hard-rejects “unknown” as “not eligible”

```yaml
- id: "E-COUNTRY-001"
  when:
    is_taiwan_eligible: false
  action: bucket "C"
```

This assumes your pipeline sets `is_taiwan_eligible` correctly.

In practice, most scholarships **do not list eligible countries**, or your parser fails to extract the list. If your default value is `false`, you will hard-reject almost everything. 

**Fix:** Make `is_taiwan_eligible` tri-state: `true | false | null`. Only hard-reject when it’s explicitly `false`.

### B. `E-NONTARGET-001` can wipe out discovery-mode sources unintentionally

```yaml
- id: "E-NONTARGET-001"
  when:
    not_any_regex: [glasgow patterns, "any UK university", ...]
  action: bucket "C"
```

If this rule runs on *raw page text* (or a lead assembled from a page that doesn’t mention “Glasgow” in the body), it will classify “non-target” too early and kill leads before they are fully extracted. 

**Fix:** Apply “non-target” only when you have a **final scholarship detail lead** (e.g., `lead.type == scholarship_detail`), not on index/discovery pages.

---

## 2) Rule changes that will immediately increase usable output (without adding noise)

### Change 1 — Make “Taiwan not eligible” a *hard reject only when explicit*

Replace `E-COUNTRY-001` with two rules:

**Hard reject only if you have an eligible list and Taiwan is absent**

```yaml
- id: "E-COUNTRY-EXPLICIT-001"
  name: "Taiwan Explicitly Not Eligible"
  stage: "eligibility"
  when:
    eligible_countries_present: true
    is_taiwan_eligible: false
  action:
    bucket: "C"
    reason: "Eligible countries listed and Taiwan is not included."
```

**Soft downgrade if eligibility is unknown**

```yaml
- id: "B-COUNTRY-UNKNOWN-001"
  name: "Country Eligibility Unknown"
  stage: "eligibility"
  when:
    eligible_countries_present: false
  action:
    bucket: "B"
    add_to_watchlist: true
    reason: "Eligible countries not stated/extracted; needs manual verification."
```

This single change typically converts “almost everything rejected” into “Bucket B watchlist”, which is what you want for exploration. 

---

### Change 2 — Gate `E-NONTARGET-001` so it doesn’t kill discovery/index pages

Add a condition like `lead_kind: scholarship_detail` (or equivalent field), and only then apply non-target filtering:

```yaml
- id: "E-NONTARGET-001"
  name: "Non-Target and Non-Portable"
  stage: "eligibility"
  when:
    lead_kind: "scholarship_detail"
    not_any_regex:
      - "(?i)gla\\.ac\\.uk"
      - "(?i)University\\s*of\\s*Glasgow"
      - "(?i)any\\s*(UK|British)\\s*university"
      - "(?i)tenable\\s+at\\s+any\\s+UK"
      - "(?i)\\bChevening\\b"
      - "(?i)\\bRotary\\b"
      - "(?i)\\bFulbright\\b"
  action:
    bucket: "C"
    reason: "Not Glasgow and not portable to UK universities."
```

If you can’t add `lead_kind`, then implement the same effect in code: **skip E-NONTARGET for sources with `type=university_index` or `type=discovery`**. 

---

### Change 3 — Lower Bucket A threshold temporarily while you stabilize extraction

Your `A` requires `min_final_score: 100` plus trust tier and effort constraints. 
While you’re still fixing extraction, temporarily reduce:

```yaml
bucket_thresholds:
  A:
    min_final_score: 80
    min_trust_tier: "C"
    max_effort_score: 75
```

Once your pipeline is healthy, raise it again.

---

## 3) How to expand **high-quality scholarship resources** (the right way)

### The highest-quality expansion is not “add more domains”

It is: **expand depth on the Glasgow domain + official hubs, then extract scholarship detail pages**.

You already have these Tier-1 seeds enabled: 

* `https://www.gla.ac.uk/scholarships/all/`
* `https://www.gla.ac.uk/scholarships/search/`
* `https://www.gla.ac.uk/scholarships/globalleadershipscholarship/`
* plus the portal

**What’s missing is a “link frontier” step:**

1. Fetch the index page
2. Extract all internal links that match `/scholarships/`
3. Enqueue them as **scholarship_detail** pages
4. Parse each detail page for deadline/amount/eligibility
5. Score + triage

This yields **dozens of real scholarships** with near-zero noise, without adding new sources.

---

## 4) Curated source expansion that stays high-quality (if you still want more breadth)

Your Tier-2 already includes British Council Study UK, FindAMasters funding, Prospects, Saltire, SAAS. 

Two expansions that remain high signal:

### A. Add *portable* scholarship owners (authoritative pages)

These are “single source, many opportunities” without becoming noisy:

* Large professional bodies (discipline-linked scholarships)
* Major education providers (IELTS/IDP awards)
* Large philanthropic funds with clear UK applicability

Add only **official pages**, not random listicles.

### B. Add a short UK-university shortlist as **disabled-by-default** (Tier-3)

You already have Edinburgh/St Andrews/Strathclyde disabled. That’s correct. 
When Tier-1/Tier-2 is stable, add 10–15 universities *as index pages only*, and keep them disabled until needed.

---

## 5) Practical checklist to get A/B counts up fast

1. **Implement tri-state Taiwan eligibility** (`eligible_countries_present`, `is_taiwan_eligible=null`).
2. **Stop applying non-target rule on discovery/index pages**.
3. **Add Glasgow link frontier crawl** (depth 1–2 within `gla.ac.uk/scholarships/`).
4. Temporarily relax Bucket A threshold while you validate extraction.
5. Only after 1–4 are working, expand sources.

---
You are getting **A=0 / B=1 / C=2** primarily because your current **hard-reject rules are being applied to “discovery/index” pages**, and because your country-eligibility signal is treated as **binary** even when the page does not explicitly provide an eligible-country list.

The good news: with the code you shared, you can fix this without “adding noisy sources.” You need to (1) **stop hard-rejecting discovery/index pages**, and (2) **treat Taiwan eligibility as tri-state** using `eligible_countries.len()` as the “explicit list exists” signal.

Below is a concrete, minimal set of changes.

---

# 1) Why it collapses to almost nothing (from your actual code + rules)

## A. `E-NONTARGET-001` is a hard reject with `not_any_regex`

In `rules.yaml`, `E-NONTARGET-001` triggers when **none** of the “portable/Glasgow” allowlist terms appear. That is fine for *final scholarship detail pages*, but it is catastrophic for:

* British Council hub pages
* FindAMasters funding index pages
* Prospects funding guide pages
* Government landing pages

Those pages often do **not** contain “University of Glasgow” or the portable keywords in the body text you build in `build_search_text()`. So they get hard-rejected early, before they can yield outbound links to real scholarships.

## B. `E-COUNTRY-001` hard rejects `is_taiwan_eligible: false`

Your Lead type already supports `is_taiwan_eligible: Option<bool>` and `eligible_countries: Vec<String>`, which is correct. But your rule is:

```yaml
when:
  is_taiwan_eligible: false
```

If your scraper sets `is_taiwan_eligible = Some(false)` by default whenever it can’t parse eligible countries, you will reject *most of the world* incorrectly.

Your rules engine currently treats `None` as “condition not met,” which is good. The problem is upstream: if you convert “unknown” → `false`, you nuke the dataset.

---

# 2) Minimal code changes that immediately increase A/B volume (without adding garbage)

## Change 1 — Only trigger `is_taiwan_eligible: false` if an explicit eligible-country list exists

You already have the perfect indicator: `eligible_countries.len() > 0`.

### Patch `check_rule_condition()` in `rules.rs`

Replace the country eligibility block:

```rust
// Check country eligibility condition
if let Some(expected_eligible) = condition.is_taiwan_eligible {
    has_any_condition = true;
    if let Some(actual_eligible) = lead.is_taiwan_eligible {
        if actual_eligible != expected_eligible {
            all_passed = false;
        }
    } else {
        all_passed = false;
    }
}
```

with this tri-state logic:

```rust
// Check country eligibility condition (tri-state)
// - expected=true  => require explicit Some(true)
// - expected=false => only trigger if eligible_countries list exists AND Some(false)
if let Some(expected_eligible) = condition.is_taiwan_eligible {
    has_any_condition = true;

    match expected_eligible {
        true => {
            // Must be explicitly confirmed eligible
            if lead.is_taiwan_eligible != Some(true) {
                all_passed = false;
            }
        }
        false => {
            // Only reject when the page explicitly lists eligible countries
            let has_explicit_country_list = !lead.eligible_countries.is_empty();
            if !has_explicit_country_list || lead.is_taiwan_eligible != Some(false) {
                all_passed = false;
            }
        }
    }
}
```

**Effect:** “unknown eligibility” stops being treated as “not eligible,” which will immediately expand Bucket B (watchlist) while preserving hard-reject for explicitly ineligible country-scoped scholarships.

---

## Change 2 — Prevent `E-NONTARGET-001` from firing on directory/discovery pages

You already have a field: `is_directory_page: bool` in `Lead`.

Right now `RuleCondition` cannot reference it. The clean fix is to add one boolean gate in `RuleCondition` and YAML.

### Step 2.1 — Extend `RuleCondition` in `types.rs`

Add:

```rust
#[serde(default)]
pub is_directory_page: Option<bool>, // true/false gate
```

### Step 2.2 — Implement it in `check_rule_condition()` in `rules.rs`

Add near the end (before the final return):

```rust
// Check directory page gate
if let Some(expected) = condition.is_directory_page {
    has_any_condition = true;
    if lead.is_directory_page != expected {
        all_passed = false;
    }
}
```

### Step 2.3 — Modify `E-NONTARGET-001` in `rules.yaml`

Make it only apply to **non-directory** pages:

```yaml
- id: "E-NONTARGET-001"
  name: "Non-Target and Non-Portable"
  stage: "eligibility"
  when:
    is_directory_page: false
    not_any_regex:
      - "(?i)gla\\.ac\\.uk"
      - "(?i)University\\s*of\\s*Glasgow"
      - "(?i)any\\s*(UK|British)\\s*university"
      - "(?i)\\bChevening\\b"
      - "(?i)\\bRotary\\s*Foundation\\b"
      - "(?i)\\bFulbright\\b"
  action:
    bucket: "C"
    reason: "Not from Glasgow and not portable to UK universities"
```

**Effect:** discovery/index pages are kept long enough to extract outbound scholarship detail URLs.

---

# 3) You also have a scoring/bucketing bug: thresholds ignore trust_tier and effort_score

In `rules.yaml` you set:

```yaml
bucket_thresholds:
  A:
    min_final_score: 100
    min_trust_tier: "B"
    max_effort_score: 60
```

But in `rules.rs`, you only check `min_final_score`. You never enforce trust tier or effort caps. That can produce weird classifications later.

**Fix (small):** in the “Determine final bucket” section, add checks:

* parse `lead.trust_tier` into `TrustTier`
* require `>= min_trust_tier`
* require `effort_score <= max_effort_score` (if present)

This won’t increase count, but it will increase correctness once your A bucket starts filling.

---

# 4) How to expand **high-quality scholarship resources** (without exploding noise)

Once the two rules above stop wiping out discovery pages, you will get more leads even with the current sources. To expand further **high-quality**:

## A. Expand “depth” on Glasgow (highest signal)

You already have `https://www.gla.ac.uk/scholarships/all/` and `/search/`.

What you need is a **frontier miner**:

1. Fetch the page.
2. Extract all internal links matching `https://www.gla.ac.uk/scholarships/…/`
3. Mark those extracted links as:

   * `source_type = "university"`
   * `is_directory_page = false`
   * `official_source_url = original index page`
4. Crawl those detail links and parse fields.

**This alone** typically turns 1–3 “Glasgow pages” into **dozens of real scholarship items**.

## B. Use discovery sites as *link generators*, not “scholarship records”

For sources like British Council / Prospects / FindAMasters:

* Treat the page itself as `is_directory_page=true`
* Extract outbound links to official scholarship pages
* Only score/triage the official pages

This is exactly how you increase count **while staying high-quality**.

## C. Add a small “UK university shortlist pack” (10–15) as disabled-by-default

You already have 3 disabled universities. Add 10 more *index pages only*, keep them disabled unless needed.

Reason: index pages are stable and high-signal; your frontier miner can extract scholarship detail pages cleanly.

Suggested pack criteria:

* Russell Group + Scotland focus
* “International postgraduate scholarships” index pages only
* No random departments, no “news posts”

---

# 5) Practical next run expectations (after the two rule fixes)

If your scraper successfully marks index pages as `is_directory_page=true` and extracts detail links:

* Bucket A: **2–10** (real apply-now items when open)
* Bucket B: **20–80** (watchlist + upcoming deadlines + needs verification)
* Bucket C/X: remainder

If you still see “items_processed=3” after this, the next bottleneck is not rules—it’s **your scraping stage not emitting multiple Lead objects per source**. In that case, the next file to inspect is `scrapers/university.rs` and `triage.rs` (how many leads you actually store per fetched page).

---

## What I need to give you a precise patch list for the scraper (no guessing)

Please paste these two files (or upload them):

* `scripts/search_scholarships/src/scrapers/university.rs`
* `scripts/search_scholarships/src/triage.rs`

Then I can tell you exactly:

* where to set `is_directory_page=true`
* where to extract scholarship detail URLs from Glasgow index pages
* how to emit N leads per source instead of 1 placeholder lead

You are not primarily “short on sources.” You are failing to convert high-quality sources into many scholarship-detail leads, so triage only sees 1–3 items and cannot populate Bucket A/B.

To expand high-quality resources and actually get more A/B items, you need two levers:

Depth expansion (frontier mining): take a small number of authoritative index pages and extract dozens of scholarship-detail URLs from them.

Breadth expansion (curated, authoritative sources only): add a small set of trusted hubs/search tools that link to official scholarship pages.

Below is an implementation-oriented plan, plus a copy/paste sources.yml expansion pack.

1) Depth expansion (highest ROI): mine Glasgow into dozens of scholarship-detail pages
Why this works

Glasgow’s official scholarship indexes are designed to be crawled into many detail pages:

Glasgow “All scholarships” index:

Glasgow “Scholarships & funding” hub:

What to implement (in your scrapers)

In both university.rs and third_party.rs, your current parsing pattern creates multiple “leads” but often assigns the same url = base_url (index page). That prevents depth growth and causes non-target filtering to nuke discovery pages.

Upgrade rule of thumb

If you’re on an index/discovery page: extract <a href> URLs and emit leads with url = resolved_href.

If you can’t extract a detail URL: treat it as directory only (do not emit as a scholarship lead).

Minimum patch

Add extract_url(element, base_url) (you already have this pattern in selenium.rs).

In parse_university_html and parse_third_party_html, set:

lead.url = extracted_detail_url

lead.official_source_url = Some(base_url.to_string())

If no detail URL found: either skip, or set is_directory_page=true and ensure rules don’t hard-reject it.

Immediate operational change

Your Glasgow …/scholarships/search/ page is likely JS-driven. Treat it as Selenium-only:

Keep https://www.gla.ac.uk/scholarships/all/ as the primary depth miner.

Switch https://www.gla.ac.uk/scholarships/search/ to the Selenium scraper if you still want it.

2) Breadth expansion (high-quality only): add authoritative hubs that feed official pages

These are “good” because they either:

are authoritative (British Council, UKCISA), or

provide structured scholarship search that points to official pages.

Recommended additions

British Council Study UK – Scholarships & funding (hub)

A canonical UK-level discovery gateway.

British Council GREAT Scholarships hub

Mostly country-scoped, but valuable for parsing eligible country lists and building correct country gating.

ScholarshipScanner (UK scholarships search tool)

A dedicated search tool for UK scholarships for 2026 entry.

UKCISA funding/finances guidance

Not a scholarship list, but a high-quality reference hub (useful for classification and sanity checks).

Country-specific scholarship pages (Tier-3 backups; not Glasgow-specific)

Birmingham “Taiwan/Japan/Korea Chancellor’s Scholarship” (Sep 2026 start).

Manchester “Taiwan scholarships” page.

Use these only if you accept “apply elsewhere” backups.

3) Copy/paste: sources.yml high-quality expansion pack

This stays high signal and is small enough that your crawler health won’t collapse.

# Add these to tracking/sources.yml

  # ============================================
  # TIER 1+: Glasgow depth mining (authoritative)
  # ============================================
  - name: "Glasgow All Scholarships (Index)"
    type: "university"
    url: "https://www.gla.ac.uk/scholarships/all/"
    enabled: true
    priority: 1
    scraper: "university"

  - name: "Glasgow Scholarships & Funding (Hub)"
    type: "university"
    url: "https://www.gla.ac.uk/scholarships/"
    enabled: true
    priority: 1
    scraper: "university"

  # If you keep this, use Selenium (JS-heavy)
  - name: "Glasgow Scholarship Search (JS)"
    type: "university"
    url: "https://www.gla.ac.uk/scholarships/search/"
    enabled: true
    priority: 1
    scraper: "selenium"

  # ============================================
  # TIER 2: Authoritative UK scholarship gateways
  # ============================================
  - name: "British Council - Study UK Scholarships & Funding"
    type: "third_party"
    url: "https://study-uk.britishcouncil.org/scholarships-funding"
    enabled: true
    priority: 2
    scraper: "third_party"

  - name: "British Council - GREAT Scholarships Hub"
    type: "third_party"
    url: "https://study-uk.britishcouncil.org/scholarships-funding/great-scholarships"
    enabled: true
    priority: 2
    scraper: "third_party"

  - name: "ScholarshipScanner - UK Scholarship Search"
    type: "third_party"
    url: "https://www.scholarshipscanner.com/scholarships"
    enabled: true
    priority: 2
    scraper: "third_party"

  - name: "UKCISA - Funding & Finances Guidance"
    type: "third_party"
    url: "https://www.ukcisa.org.uk/student-advice/finances/"
    enabled: true
    priority: 2
    scraper: "third_party"

  # ============================================
  # TIER 3 (optional): Country-specific backups
  # ============================================
  - name: "University of Birmingham - Taiwan Chancellor's Scholarship"
    type: "university"
    url: "https://www.birmingham.ac.uk/study/scholarships-funding/taiwan-japan-and-south-korea-postgraduate-chancellors-scholarships"
    enabled: false
    priority: 3
    scraper: "university"

  - name: "University of Manchester - Taiwan Scholarships"
    type: "university"
    url: "https://www.manchester.ac.uk/study/international/country-specific-information/taiwan/scholarships/"
    enabled: false
    priority: 3
    scraper: "university"


(Each of these links is stable and authoritative. )

4) Guardrails to keep expansion “high-quality” (so B doesn’t become garbage)
A. Treat discovery pages as “frontiers,” not scholarships

Policy:

is_directory_page=true for discovery/index sources

Extract scholarship detail URLs

Only detail pages are eligible for Bucket A/B

B. Add domain allowlist for “detail follow”

When a discovery page emits links, only follow if the target domain matches:

gla.ac.uk (Tier 1)

britishcouncil.org, study-uk.britishcouncil.org (Tier 2)

other university domains (Tier 3 shortlist only)

This prevents link explosions into random blogs.

C. Don’t hard-reject “country unknown”

Country lists are often absent; Taiwan eligibility should be tri-state:

Explicit list exists + Taiwan absent → reject

No list → keep, mark “needs verification”

5) What I will do next if you want a precise patch (no guessing)

You’ve already uploaded university.rs, third_party.rs, and triage.rs. If you confirm that you want code-level edits, I can provide:

A concrete diff-style patch for:

university.rs: extract <a href> and emit detail URLs

third_party.rs: same

selenium.rs: fix source_type and ensure extracted URLs are preserved

A small change to rules.yaml so discovery/index pages are not bucketed as “non-target”

If you want this, tell me which mode you prefer:


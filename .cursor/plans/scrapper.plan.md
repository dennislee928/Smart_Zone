---
name: ""
overview: ""
todos:
  - id: m1-url-normalization
    content: Implement URL normalization + canonical URL resolution
    status: completed
  - id: m1-entity-dedupe
    content: Add entity-level dedupe key (provider+title+deadline+award+level)
    status: completed
  - id: m1-directory-detection
    content: "Move 'directory page' detection to pipeline pre-triage: directory pages ONLY emit outbound/detail links; detail pages eligible for extraction + scoring"
    status: completed
  - id: m1-report-generator
    content: Update report generator to only print unique entities + include dup_count
    status: completed
  - id: m2-discovery-strategy
    content: "Per-source discovery strategy: robots.txt -> sitemap URLs, sitemap index traversal (with size limits), RSS/Atom feeds if present, site internal search endpoints (keyword templates)"
    status: completed
  - id: m2-candidate-url-queue
    content: "Build a unified CandidateUrl queue: url, source_id, discovered_from, confidence, discovered_at"
    status: completed
  - id: m2-allowlist-path-regex
    content: Add allowlist path regex per source (e.g., /scholarship|/funding|/bursary)
    status: completed
  - id: m2-content-type-gates
    content: Add content-type gates (skip large binaries; handle PDFs explicitly)
    status: completed
  - id: m3-url-state-storage
    content: "Add url_state storage (SQLite preferred): etag, last_modified, content_hash, last_seen, status"
    status: completed
  - id: m3-conditional-get
    content: Support conditional GET (If-None-Match / If-Modified-Since)
    status: completed
  - id: m3-evidence-fields
    content: "Evidence fields for extracted attributes: snippet + selector/xpath + url"
    status: completed
  - id: m3-extraction-fallbacks
    content: "Extraction fallbacks: JSON-LD / schema.org parsing, regex fallback for deadline/award with locale-aware parsing"
    status: completed
  - id: m4-matrix-sharding
    content: Split sources into domain-based shards (10–30 shards)
    status: completed
  - id: m4-matrix-config
    content: Use jobs.<job_id>.strategy.matrix with max-parallel control
    status: completed
  - id: m4-separate-workflows
    content: "Separate workflows: incremental.yml (daily/weekly), deepcrawl.yml (weekly)"
    status: completed
  - id: m4-schedule-default-branch
    content: Ensure schedule workflows exist on default branch
    status: completed
  - id: m5-error-taxonomy
    content: "Error taxonomy: blocked(403/429), timeout, parse_error, robots_disallow"
    status: completed
  - id: m5-cooldown-auto-disable
    content: Cooldown-based auto-disable (e.g., disable 24h after N consecutive blocked)
    status: completed
  - id: m5-per-domain-politeness
    content: "Per-domain politeness: min_delay_ms, max_concurrency, retry/backoff policy"
    status: completed
  - id: m5-dashboard-summary
    content: "Dashboard summary: unique_found, dup_rate, missing_deadline_rate, blocked_rate per source"
    status: completed
isProject: false
---

# ScholarshipOps Crawler Expansion Plan (Coverage + Quality)

## Objectives

- Increase scholarship discovery coverage (more unique, relevant detail pages).

- Improve output quality: deduped, auditable, stable across runs.

- Keep crawling polite and resilient (robots/sitemap-first, incremental fetch, bounded concurrency).

## Constraints

- Public repo Actions usage is free for standard GitHub-hosted runners. (billing)

- GitHub-hosted runner: 6h max per job; workflow run: 35 days; matrix: 256 jobs/run.

- Scheduled workflows run on default branch latest commit.

## Milestone M1 — Fix Output Noise (Dedupe + Directory gating)

- [ ] Implement URL normalization + canonical URL resolution

- [ ] Add entity-level dedupe key (provider+title+deadline+award+level)

- [ ] Move "directory page" detection to pipeline pre-triage:

    - directory pages: ONLY emit outbound/detail links

    - detail pages: eligible for extraction + scoring

- [ ] Update report generator to only print unique entities + include dup_count

## Milestone M2 — Discovery Engine (Sitemap/RSS/Search endpoints)

- [ ] Per-source discovery strategy:

    - robots.txt -> sitemap URLs

    - sitemap index traversal (with size limits)

    - RSS/Atom feeds if present

    - site internal search endpoints (keyword templates)

- [ ] Build a unified CandidateUrl queue:

    - url, source_id, discovered_from, confidence, discovered_at

- [ ] Add allowlist path regex per source (e.g., /scholarship|/funding|/bursary)

- [ ] Add content-type gates (skip large binaries; handle PDFs explicitly)

## Milestone M3 — Incremental Fetch + Evidence

- [ ] Add url_state storage (SQLite preferred):

    - etag, last_modified, content_hash, last_seen, status

- [ ] Support conditional GET (If-None-Match / If-Modified-Since)

- [ ] Evidence fields for extracted attributes:

    - snippet + selector/xpath + url

- [ ] Extraction fallbacks:

    - JSON-LD / schema.org parsing

    - regex fallback for deadline/award with locale-aware parsing

## Milestone M4 — Actions Scaling (Matrix sharding)

- [ ] Split sources into domain-based shards (10–30 shards)

- [ ] Use jobs.<job_id>.strategy.matrix with max-parallel control

- [ ] Separate workflows:

    - incremental.yml (daily/weekly)

    - deepcrawl.yml (weekly)

- [ ] Ensure schedule workflows exist on default branch

## Milestone M5 — Source Health Ops

- [ ] Error taxonomy: blocked(403/429), timeout, parse_error, robots_disallow

- [ ] Cooldown-based auto-disable (e.g., disable 24h after N consecutive blocked)

- [ ] Per-domain politeness:

    - min_delay_ms, max_concurrency, retry/backoff policy

- [ ] Dashboard summary:

    - unique_found, dup_rate, missing_deadline_rate, blocked_rate per source

## Todos

### Milestone M1 — Fix Output Noise (Dedupe + Directory gating)

- [ ] m1-url-normalization: Implement URL normalization + canonical URL resolution
- [ ] m1-entity-dedupe: Add entity-level dedupe key (provider+title+deadline+award+level)
- [ ] m1-directory-detection: Move "directory page" detection to pipeline pre-triage: directory pages ONLY emit outbound/detail links; detail pages eligible for extraction + scoring
- [ ] m1-report-generator: Update report generator to only print unique entities + include dup_count

### Milestone M2 — Discovery Engine (Sitemap/RSS/Search endpoints)

- [ ] m2-discovery-strategy: Per-source discovery strategy: robots.txt -> sitemap URLs, sitemap index traversal (with size limits), RSS/Atom feeds if present, site internal search endpoints (keyword templates)
- [ ] m2-candidate-url-queue: Build a unified CandidateUrl queue: url, source_id, discovered_from, confidence, discovered_at
- [ ] m2-allowlist-path-regex: Add allowlist path regex per source (e.g., /scholarship|/funding|/bursary)
- [ ] m2-content-type-gates: Add content-type gates (skip large binaries; handle PDFs explicitly)

### Milestone M3 — Incremental Fetch + Evidence

- [ ] m3-url-state-storage: Add url_state storage (SQLite preferred): etag, last_modified, content_hash, last_seen, status
- [ ] m3-conditional-get: Support conditional GET (If-None-Match / If-Modified-Since)
- [ ] m3-evidence-fields: Evidence fields for extracted attributes: snippet + selector/xpath + url
- [ ] m3-extraction-fallbacks: Extraction fallbacks: JSON-LD / schema.org parsing, regex fallback for deadline/award with locale-aware parsing

### Milestone M4 — Actions Scaling (Matrix sharding)

- [ ] m4-matrix-sharding: Split sources into domain-based shards (10–30 shards)
- [ ] m4-matrix-config: Use jobs.<job_id>.strategy.matrix with max-parallel control
- [ ] m4-separate-workflows: Separate workflows: incremental.yml (daily/weekly), deepcrawl.yml (weekly)
- [ ] m4-schedule-default-branch: Ensure schedule workflows exist on default branch

### Milestone M5 — Source Health Ops

- [ ] m5-error-taxonomy: Error taxonomy: blocked(403/429), timeout, parse_error, robots_disallow
- [ ] m5-cooldown-auto-disable: Cooldown-based auto-disable (e.g., disable 24h after N consecutive blocked)
- [ ] m5-per-domain-politeness: Per-domain politeness: min_delay_ms, max_concurrency, retry/backoff policy
- [ ] m5-dashboard-summary: Dashboard summary: unique_found, dup_rate, missing_deadline_rate, blocked_rate per source
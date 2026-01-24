---
name: ""
overview: ""
todos: []
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
# Dead Links Report

Generated: 2026-08-02 14:45 UTC

## Summary

- Total URLs checked: 9
- Healthy (200-299): 4
- **True Dead (404/410)**: 1
- Transient (403/5xx/Timeout): 4
- Rate limited (429): 1
- Redirects (301/302): 0

## True Dead Links (Confirmed 404/410)

These URLs are confirmed dead after GET verification:

| URL | HTTP Code | Checked At |
|-----|-----------|------------|
| https://www.gla.ac.uk/scholarships/scienceengineeringexce... | 404 | 2026-08-02 14:43:53 UTC |

## Transient Issues (May Recover)

These URLs had temporary issues - retry with backoff:

| URL | Status | HTTP Code | Error | Checked At |
|-----|--------|-----------|-------|------------|
| https://study-uk.britishcouncil.org/scholarship... | 403 Forbidden | 403 | - | 2026-08-02 14:43:53 UTC |
| https://www.wolfson.org.uk/ | 429 Rate Limited | 429 | - | 2026-08-02 14:43:53 UTC |
| https://www.chevening.org/scholarships/ | Timeout | - | Request timed out: error sending request for url (https://www.chevening.org/scholarships/): operation timed out | 2026-08-02 14:43:57 UTC |
| https://cscuk.fcdo.gov.uk/scholarships/commonwe... | 403 Forbidden | 403 | - | 2026-08-02 14:43:57 UTC |

## Rate Limited (429 - Needs Backoff)

These URLs returned 429 - implement exponential backoff:

| URL | Checked At |
|-----|------------|
| https://www.wolfson.org.uk/ | 2026-08-02 14:43:53 UTC |


# Dead Links Report

Generated: 2026-09-04 16:53 UTC

## Summary

- Total URLs checked: 10
- Healthy (200-299): 5
- **True Dead (404/410)**: 2
- Transient (403/5xx/Timeout): 3
- Rate limited (429): 1
- Redirects (301/302): 0

## True Dead Links (Confirmed 404/410)

These URLs are confirmed dead after GET verification:

| URL | HTTP Code | Checked At |
|-----|-----------|------------|
| https://www.gla.ac.uk/scholarships/mscinternationalandcom... | 404 | 2026-09-04 16:51:46 UTC |
| https://www.gla.ac.uk/scholarships/scienceengineeringexce... | 404 | 2026-09-04 16:51:50 UTC |

## Transient Issues (May Recover)

These URLs had temporary issues - retry with backoff:

| URL | Status | HTTP Code | Error | Checked At |
|-----|--------|-----------|-------|------------|
| https://www.rotary.org/en/our-programs/scholars... | 429 Rate Limited | 429 | - | 2026-09-04 16:51:46 UTC |
| https://www.chevening.org/scholarships/ | Timeout | - | Request timed out: error sending request for url (https://www.chevening.org/scholarships/): operation timed out | 2026-09-04 16:51:50 UTC |
| https://study-uk.britishcouncil.org/scholarship... | 403 Forbidden | 403 | - | 2026-09-04 16:51:50 UTC |

## Rate Limited (429 - Needs Backoff)

These URLs returned 429 - implement exponential backoff:

| URL | Checked At |
|-----|------------|
| https://www.rotary.org/en/our-programs/scholarships | 2026-09-04 16:51:46 UTC |


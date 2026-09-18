# Dead Links Report

Generated: 2026-09-18 17:06 UTC

## Summary

- Total URLs checked: 9
- Healthy (200-299): 3
- **True Dead (404/410)**: 3
- Transient (403/5xx/Timeout): 3
- Rate limited (429): 1
- Redirects (301/302): 0

## True Dead Links (Confirmed 404/410)

These URLs are confirmed dead after GET verification:

| URL | HTTP Code | Checked At |
|-----|-----------|------------|
| https://www.gla.ac.uk/scholarships/greatscholarships2026/ | 404 | 2026-09-18 17:05:06 UTC |
| https://www.gla.ac.uk/scholarships/scienceengineeringexce... | 404 | 2026-09-18 17:05:06 UTC |
| https://www.gla.ac.uk/scholarships/mscinternationalandcom... | 404 | 2026-09-18 17:05:06 UTC |

## Transient Issues (May Recover)

These URLs had temporary issues - retry with backoff:

| URL | Status | HTTP Code | Error | Checked At |
|-----|--------|-----------|-------|------------|
| https://www.chevening.org/scholarships/ | Timeout | - | Request timed out: error sending request for url (https://www.chevening.org/scholarships/): operation timed out | 2026-09-18 17:05:07 UTC |
| https://www.rotary.org/en/our-programs/scholars... | 429 Rate Limited | 429 | - | 2026-09-18 17:05:07 UTC |
| https://study-uk.britishcouncil.org/scholarship... | 403 Forbidden | 403 | - | 2026-09-18 17:05:07 UTC |

## Rate Limited (429 - Needs Backoff)

These URLs returned 429 - implement exponential backoff:

| URL | Checked At |
|-----|------------|
| https://www.rotary.org/en/our-programs/scholarships | 2026-09-18 17:05:07 UTC |


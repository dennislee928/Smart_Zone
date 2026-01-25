# ScholarshipOps automation

[![Python](https://img.shields.io/badge/Python-3.8%2B-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Go](https://img.shields.io/badge/Go-1.21-00ADD8?logo=go&logoColor=white)](https://golang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker&logoColor=white)](https://hub.docker.com)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![GitHub Actions](https://img.shields.io/badge/GitHub%20Actions-7%20workflows-2088FF?logo=github-actions&logoColor=white)](.github/workflows)
[![MCP](https://img.shields.io/badge/MCP-Compatible-purple.svg)](https://github.com/dennislee928/Smart_Zone)
[![Cloudflare](https://img.shields.io/badge/Cloudflare-Workers%20%2B%20D1-F38020?logo=cloudflare&logoColor=white)](https://workers.cloudflare.com/)
[![Version](https://img.shields.io/badge/Version-0.1.0-orange.svg)](https://github.com/dennislee928/Smart_Zone/releases)
[![Stars](https://img.shields.io/github/stars/dennislee928/Smart_Zone?style=social)](https://github.com/dennislee928/Smart_Zone)

This repository automates scholarship reminder pushes (lunch, evening, weekend) via GitHub Actions and optional Telegram/Slack/Discord notifications. It also supports scholarship search (Rust + web scraping), application scheduling (Go), and progress tracking (Go).

## Schedule (Asia/Taipei)

| Workflow | Local time | UTC cron | Frequency |
| --- | --- | --- | --- |
| Reminder (Lunch) | 12:10 | `10 4 * * 1-5` | Weekdays |
| Reminder (Evening) | 21:10 | `10 13 * * 1-5` | Weekdays |
| Reminder (Weekend) | 10:00 | `0 2 * * 6,0` | Weekends |
| Search | 21:10 | `10 13 * * 3` | Wednesday |
| Schedule | 12:10 | `10 4 * * 1` | Monday |
| Track | 21:10 | `10 13 * * 5` | Friday |

The workflow always runs on the default branch with the latest commit, as required by GitHub Actions scheduled events.

## Secrets

Configure secrets in **Settings → Secrets and variables → Actions**.

### Telegram (recommended)

- `TELEGRAM_BOT_TOKEN`
- `TELEGRAM_CHAT_ID`

### Slack (optional)

- `SLACK_WEBHOOK_URL`

### Discord (optional)

- `DISCORD_WEBHOOK_URL`

## GitHub-only fallback

The workflow can create daily issues to trigger GitHub mobile notifications. This is controlled by `CREATE_ISSUE` in the workflow.

## Workflows

### Reminder Workflow (`.github/workflows/remind.yml`)

Daily reminders at lunch (12:10), evening (21:10), and weekend (10:00) Asia/Taipei time.

### Search Workflow (`.github/workflows/search.yml`)

Weekly scholarship search via web scraping (Rust). Runs every Wednesday 21:10 Asia/Taipei.

- Searches university websites, government sites, and third-party databases
- Filters by eligibility criteria with advanced profile matching
- Multi-dimensional sorting by ROI, urgency, and source reliability
- Updates `tracking/leads.json`
- Generates multi-format reports (HTML, Markdown, TXT) in `scripts/productions/YYYY-MM-DD_HH-MM/`
- Automatically commits report folders to repository
- Sends summary notifications via Telegram/Slack/Discord

### Schedule Workflow (`.github/workflows/schedule.yml`)

Weekly application scheduling suggestions (Go). Runs every Monday 12:10 Asia/Taipei.

- Analyzes upcoming deadlines
- Suggests applications to prioritize
- Reads from `tracking/applications.json` and `tracking/leads.json`

### Track Workflow (`.github/workflows/track.yml`)

Weekly progress tracking report (Go). Runs every Friday 21:10 Asia/Taipei.

- Calculates statistics (total, in progress, completed, upcoming deadlines)
- Generates progress report
- Tracks D-7, D-14, D-21 deadlines

## Files

### Core Scripts

- `.github/workflows/remind.yml`: reminder workflow (Python)
- `.github/workflows/search.yml`: search workflow (Rust)
- `.github/workflows/schedule.yml`: schedule workflow (Go)
- `.github/workflows/track.yml`: track workflow (Go)
- `scripts/remind.py`: Python reminder script
- `scripts/search_scholarships/`: Rust scholarship search project
- `scripts/schedule_applications/`: Go scheduling application
- `scripts/track_progress/`: Go progress tracker

### Task Configuration

- `tasks/daily_2026-01-18_to_02-10.yml`: daily plan
- `tasks/phases_2026-02-11_to_06-01.yml`: phase-based cadence through 2026-06-01
- `tasks/deadlines.yml`: upcoming deadlines for D-21 style reminders

### Tracking Data (JSON/YAML)

- `tracking/applications.json`: application tracking (JSON)
- `tracking/leads.json`: potential scholarships list (JSON)
- `tracking/criteria.yml`: search eligibility criteria (YAML)
- `tracking/sources.yml`: manually maintained scraping targets (YAML)

### Generated Reports

- `scripts/productions/YYYY-MM-DD_HH-MM/`: Date-based report folders containing:
  - `report.html`: HTML format with styled tables and responsive design
  - `report.md`: Markdown format with tables and detailed information
  - `report.txt`: Plain text format for easy reading

## Data Formats

- **JSON**: Used for `applications.json`, `leads.json` (efficient for program processing)
- **YAML**: Used for `criteria.yml`, `sources.yml`, `tasks/*.yml` (easy manual editing)
- **Dual format support**: Go/Rust programs can read/write both formats

## Report Generation

The search workflow automatically generates comprehensive reports in three formats:

- **HTML**: Styled reports with tables, color-coded urgency indicators, and responsive design
- **Markdown**: Formatted reports with tables and detailed scholarship information
- **TXT**: Plain text reports for easy reading and processing

Reports are saved in date-based folders (`scripts/productions/YYYY-MM-DD_HH-MM/`) and automatically committed to the repository. Each report includes:

- Complete list of qualified scholarships (no truncation)
- Multi-dimensional sorting scores (Match + ROI + Urgency + Source Reliability)
- Detailed eligibility information and match reasons
- Deadline urgency indicators (D-7, D-14, D-21)
- Filtered out scholarships summary
- Error reports for failed scraping attempts

## Updating

- **Daily changes**: Edit the YAML files under `tasks/` and push.
- **Weekly**: Update `tasks/deadlines.yml` with new confirmed deadlines.
- **Scholarship search**: Update `tracking/sources.yml` to add new scraping targets.
- **Application tracking**: Update `tracking/applications.json` manually or via scripts.
- **Search criteria**: Update `tracking/criteria.yml` to modify eligibility requirements and profile information.

## Language

- [English](README.md)
- [繁體中文](readme_zhTW.md)
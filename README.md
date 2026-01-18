# ScholarshipOps automation

This repository automates scholarship reminder pushes (lunch, evening, weekend) via GitHub Actions and optional Telegram/Slack/Discord notifications. It also supports scholarship search (Rust + web scraping), application scheduling (Go), and progress tracking (Go).

## Schedule (Asia/Taipei)

| Reminder | Local time | UTC cron |
| --- | --- | --- |
| Lunch | 12:10 | `10 4 * * 1-5` |
| Evening | 21:10 | `10 13 * * 1-5` |
| Weekend | 10:00 | `0 2 * * 6,0` |

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
- Filters by eligibility criteria
- Updates `tracking/leads.json`

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

## Data Formats

- **JSON**: Used for `applications.json`, `leads.json` (efficient for program processing)
- **YAML**: Used for `criteria.yml`, `sources.yml`, `tasks/*.yml` (easy manual editing)
- **Dual format support**: Go/Rust programs can read/write both formats

## Updating

- Daily changes: edit the YAML files under `tasks/` and push.
- Weekly: update `tasks/deadlines.yml` with new confirmed deadlines.
- Scholarship search: update `tracking/sources.yml` to add new scraping targets.
- Application tracking: update `tracking/applications.json` manually or via scripts.
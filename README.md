# ScholarshipOps automation

This repository automates scholarship reminder pushes (lunch, evening, weekend) via GitHub Actions and optional Telegram/Slack notifications. It also supports a GitHub-issue-only fallback for mobile push notifications.

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

## GitHub-only fallback

The workflow can create daily issues to trigger GitHub mobile notifications. This is controlled by `CREATE_ISSUE` in the workflow.

## Files

- `.github/workflows/remind.yml`: schedule + run the reminder script.
- `scripts/remind.py`: load tasks, format messages, send pushes.
- `tasks/daily_2026-01-18_to_02-10.yml`: daily plan.
- `tasks/phases_2026-02-11_to_06-01.yml`: phase-based cadence through 2026-06-01.
- `tasks/deadlines.yml`: upcoming deadlines for D-21 style reminders.

## Updating

- Daily changes: edit the YAML files under `tasks/` and push.
- Weekly: update `tasks/deadlines.yml` with new confirmed deadlines.

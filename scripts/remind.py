import os
import datetime as dt
from dateutil import tz
import requests
import yaml

ROOT = os.path.dirname(os.path.dirname(__file__))


def load_yaml(relpath: str) -> dict:
    with open(os.path.join(ROOT, relpath), "r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def today_taipei() -> dt.date:
    tpe = tz.gettz("Asia/Taipei")
    return dt.datetime.now(tpe).date()


def detect_mode(event_schedule: str | None) -> str:
    if not event_schedule:
        return "manual"
    if event_schedule.strip() == "10 4 * * 1-5":
        return "lunch"
    if event_schedule.strip() == "10 13 * * 1-5":
        return "evening"
    if event_schedule.strip() == "0 2 * * 6,0":
        return "weekend"
    return "unknown"


def pick_tasks(date_str: str, mode: str) -> dict:
    daily = load_yaml("tasks/daily_2026-01-18_to_02-10.yml")
    phases = load_yaml("tasks/phases_2026-02-11_to_06-01.yml")

    if date_str in daily.get("days", {}):
        return daily["days"][date_str].get(mode, daily["days"][date_str].get("default", {}))

    d = dt.date.fromisoformat(date_str)
    for ph in phases.get("phases", []):
        start = dt.date.fromisoformat(ph["start"])
        end = dt.date.fromisoformat(ph["end"])
        if start <= d <= end:
            key = "weekend" if mode == "weekend" else "weekday"
            return ph.get(key, {})
    return {
        "title": "No tasks configured",
        "todo": ["Update your phases/tasks files"],
        "deliverable": "",
    }


def upcoming_deadlines(date_str: str, horizon_days: int = 21) -> list[dict]:
    d0 = dt.date.fromisoformat(date_str)
    d1 = d0 + dt.timedelta(days=horizon_days)
    dd = load_yaml("tasks/deadlines.yml").get("deadlines", [])
    out = []
    for item in dd:
        due = dt.date.fromisoformat(item["date"])
        if d0 <= due <= d1:
            out.append({**item, "d_minus": (due - d0).days})
    out.sort(key=lambda x: x["d_minus"])
    return out


def format_message(date_str: str, mode: str, task: dict, deadlines: list[dict]) -> str:
    mode_label = {
        "lunch": "Lunch 10m (Inbox / Deadline / 待辦更新)",
        "evening": "Evening 50–65m 深工（寫作/送件/材料整理）",
        "weekend": "Weekend 2.5–4h 深工（完成一整份申請/材料包升級）",
        "manual": "Manual run",
    }.get(mode, mode)

    lines = []
    lines.append(f"[ScholarshipOps] {date_str} — {mode_label}")
    if task.get("title"):
        lines.append(f"- Focus: {task['title']}")
    todo = task.get("todo", [])
    if todo:
        lines.append("- Today TODO:")
        for i, t in enumerate(todo, 1):
            lines.append(f"  {i}. {t}")
    if task.get("deliverable"):
        lines.append(f"- Deliverable: {task['deliverable']}")

    if deadlines:
        lines.append("")
        lines.append("Upcoming deadlines (next 21d):")
        for it in deadlines[:8]:
            lines.append(f"- D-{it['d_minus']:02d} {it['name']} ({it['date']})")

    return "\n".join(lines)


def send_slack(webhook_url: str, text: str) -> None:
    r = requests.post(webhook_url, json={"text": text}, timeout=15)
    r.raise_for_status()


def send_telegram(token: str, chat_id: str, text: str) -> None:
    url = f"https://api.telegram.org/bot{token}/sendMessage"
    r = requests.post(url, data={"chat_id": chat_id, "text": text}, timeout=15)
    r.raise_for_status()


def create_github_issue(repo: str, gh_token: str, title: str, body: str) -> None:
    api = f"https://api.github.com/repos/{repo}/issues"
    headers = {"Authorization": f"Bearer {gh_token}", "Accept": "application/vnd.github+json"}
    r = requests.post(api, headers=headers, json={"title": title, "body": body}, timeout=15)
    r.raise_for_status()


def main() -> None:
    date_str = today_taipei().isoformat()
    mode = detect_mode(os.getenv("EVENT_SCHEDULE"))
    task = pick_tasks(date_str, mode)
    dd = upcoming_deadlines(date_str, horizon_days=21)
    msg = format_message(date_str, mode, task, dd)

    slack = os.getenv("SLACK_WEBHOOK_URL", "").strip()
    t_token = os.getenv("TELEGRAM_BOT_TOKEN", "").strip()
    t_chat = os.getenv("TELEGRAM_CHAT_ID", "").strip()

    sent = False
    if t_token and t_chat:
        send_telegram(t_token, t_chat, msg)
        sent = True
    if slack:
        send_slack(slack, msg)
        sent = True

    if os.getenv("CREATE_ISSUE", "false").lower() == "true":
        repo = os.getenv("REPO", "").strip()
        gh_token = os.getenv("GH_TOKEN", "").strip()
        if repo and gh_token:
            create_github_issue(repo, gh_token, f"{date_str} [{mode}] ScholarshipOps", msg)

    if not sent:
        print(msg)


if __name__ == "__main__":
    main()

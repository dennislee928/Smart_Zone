# ScholarshipOps 自動化系統

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

本儲存庫透過 GitHub Actions 自動化獎學金提醒推送（午餐、晚上、週末），並支援可選的 Telegram/Slack/Discord 通知。同時支援獎學金搜尋（Rust + 網頁爬蟲）、申請排程（Go）和進度追蹤（Go）。

## 排程（Asia/Taipei）

| 工作流程 | 本地時間 | UTC cron | 頻率 |
| --- | --- | --- | --- |
| 提醒（午餐） | 12:10 | `10 4 * * 1-5` | 平日 |
| 提醒（晚上） | 21:10 | `10 13 * * 1-5` | 平日 |
| 提醒（週末） | 10:00 | `0 2 * * 6,0` | 週末 |
| 搜尋 | 21:10 | `10 13 * * 3` | 每週三 |
| 排程 | 12:10 | `10 4 * * 1` | 每週一 |
| 追蹤 | 21:10 | `10 13 * * 5` | 每週五 |

工作流程始終在預設分支上執行最新提交，這是 GitHub Actions 排程事件的要求。

## 機密設定

在 **Settings → Secrets and variables → Actions** 中設定機密資訊。

### Telegram（推薦）

- `TELEGRAM_BOT_TOKEN`
- `TELEGRAM_CHAT_ID`

### Slack（可選）

- `SLACK_WEBHOOK_URL`

### Discord（可選）

- `DISCORD_WEBHOOK_URL`

## 僅 GitHub 備用方案

工作流程可以建立每日 issue 來觸發 GitHub 行動裝置通知。這由工作流程中的 `CREATE_ISSUE` 控制。

## 工作流程

### 提醒工作流程（`.github/workflows/remind.yml`）

每日提醒時間：午餐（12:10）、晚上（21:10）和週末（10:00）Asia/Taipei 時間。

### 搜尋工作流程（`.github/workflows/search.yml`）

每週透過網頁爬蟲進行獎學金搜尋（Rust）。每週三 21:10 Asia/Taipei 執行。

- 搜尋大學網站、政府網站和第三方資料庫
- 透過進階個人資料匹配依資格條件篩選
- 多維度排序（投資報酬率、緊急程度、來源可靠性）
- 更新 `tracking/leads.json`
- 在 `scripts/productions/YYYY-MM-DD_HH-MM/` 產生多格式報告（HTML、Markdown、TXT）
- 自動將報告資料夾提交至儲存庫
- 透過 Telegram/Slack/Discord 發送摘要通知

### 排程工作流程（`.github/workflows/schedule.yml`）

每週申請排程建議（Go）。每週一 12:10 Asia/Taipei 執行。

- 分析即將到來的截止日期
- 建議優先處理的申請
- 從 `tracking/applications.json` 和 `tracking/leads.json` 讀取資料

### 追蹤工作流程（`.github/workflows/track.yml`）

每週進度追蹤報告（Go）。每週五 21:10 Asia/Taipei 執行。

- 計算統計資料（總數、進行中、已完成、即將到來的截止日期）
- 產生進度報告
- 追蹤 D-7、D-14、D-21 截止日期

## 檔案結構

### 核心腳本

- `.github/workflows/remind.yml`: 提醒工作流程（Python）
- `.github/workflows/search.yml`: 搜尋工作流程（Rust）
- `.github/workflows/schedule.yml`: 排程工作流程（Go）
- `.github/workflows/track.yml`: 追蹤工作流程（Go）
- `scripts/remind.py`: Python 提醒腳本
- `scripts/search_scholarships/`: Rust 獎學金搜尋專案
- `scripts/schedule_applications/`: Go 排程應用程式
- `scripts/track_progress/`: Go 進度追蹤器

### 任務設定

- `tasks/daily_2026-01-18_to_02-10.yml`: 每日計畫
- `tasks/phases_2026-02-11_to_06-01.yml`: 階段性節奏，至 2026-06-01
- `tasks/deadlines.yml`: 即將到來的截止日期，用於 D-21 風格提醒

### 追蹤資料（JSON/YAML）

- `tracking/applications.json`: 申請追蹤（JSON）
- `tracking/leads.json`: 潛在獎學金清單（JSON）
- `tracking/criteria.yml`: 搜尋資格條件（YAML）
- `tracking/sources.yml`: 手動維護的爬蟲目標（YAML）

### 產生的報告

- `scripts/productions/YYYY-MM-DD_HH-MM/`: 基於日期的報告資料夾，包含：
  - `report.html`: HTML 格式，含樣式表格和響應式設計
  - `report.md`: Markdown 格式，含表格和詳細資訊
  - `report.txt`: 純文字格式，易於閱讀

## 資料格式

- **JSON**: 用於 `applications.json`、`leads.json`（適合程式處理）
- **YAML**: 用於 `criteria.yml`、`sources.yml`、`tasks/*.yml`（易於手動編輯）
- **雙格式支援**: Go/Rust 程式可以讀寫兩種格式

## 報告產生

搜尋工作流程會自動產生三種格式的完整報告：

- **HTML**: 含樣式表格、顏色編碼緊急程度指標和響應式設計的報告
- **Markdown**: 含表格和詳細獎學金資訊的格式化報告
- **TXT**: 易於閱讀和處理的純文字報告

報告儲存在基於日期的資料夾（`scripts/productions/YYYY-MM-DD_HH-MM/`）中，並自動提交至儲存庫。每個報告包含：

- 完整符合資格的獎學金清單（無截斷）
- 多維度排序分數（匹配度 + 投資報酬率 + 緊急程度 + 來源可靠性）
- 詳細資格資訊和匹配原因
- 截止日期緊急程度指標（D-7、D-14、D-21）
- 已篩除獎學金摘要
- 失敗爬蟲嘗試的錯誤報告

## 更新方式

- **每日變更**: 編輯 `tasks/` 下的 YAML 檔案並推送。
- **每週**: 在 `tasks/deadlines.yml` 中更新新的確認截止日期。
- **獎學金搜尋**: 更新 `tracking/sources.yml` 以新增爬蟲目標。
- **申請追蹤**: 手動或透過腳本更新 `tracking/applications.json`。
- **搜尋條件**: 更新 `tracking/criteria.yml` 以修改資格要求和個人資料資訊。

## 語言

- [English](README.md)
- [繁體中文](readme_zhTW.md)

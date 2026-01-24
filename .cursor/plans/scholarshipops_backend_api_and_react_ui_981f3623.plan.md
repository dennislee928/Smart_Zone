---
name: ScholarshipOps Backend API and React UI
overview: 建立 Cloudflare Workers + Hono 後端 API（使用 D1 資料庫）和 React UI，整合現有的 Rust 爬蟲與 Go 應用程式，並提供 Docker 容器化部署方案。
todos:
  - id: backend-setup
    content: 建立 container/ 目錄結構，初始化 Cloudflare Workers + Hono 專案，設定 package.json、tsconfig.json、wrangler.toml
    status: pending
  - id: d1-schema
    content: 設計並實作 D1 資料庫 schema（leads, applications, criteria, sources, source_health 表），建立 migrations/0001_init.sql
    status: pending
  - id: api-routes
    content: 實作 Hono API 路由：leads, applications, criteria, stats, triggers 端點，加入 CORS middleware
    status: pending
  - id: db-layer
    content: 實作 D1 資料庫操作層（db/leads.ts, db/applications.ts, db/criteria.ts），提供 CRUD 函數
    status: pending
  - id: script-triggers
    content: 實作腳本觸發器（scripts/rust-scraper.ts, go-scheduler.ts, go-tracker.ts），整合 Rust/Go 二進位檔
    status: pending
  - id: dockerfile
    content: 建立多階段 Dockerfile，建置 Rust/Go 專案，設定 Node.js runtime，安裝 wrangler，暴露端口 8787
    status: pending
  - id: env-example
    content: 建立 container/.env.example，包含 Cloudflare 帳號、D1 資料庫、CORS 等環境變數
    status: pending
  - id: react-setup
    content: 初始化 web-UI/ React + Vite + TypeScript 專案，設定 Tailwind CSS（可選）
    status: pending
  - id: api-client
    content: 建立 React API 客戶端（api/client.ts, leads.ts, applications.ts, criteria.ts, stats.ts, triggers.ts）
    status: pending
  - id: typescript-types
    content: 定義 TypeScript 型別（types/lead.ts, application.ts, criteria.ts, stats.ts），對應後端資料結構
    status: pending
  - id: ui-components
    content: 建立主要 UI 元件：Dashboard, LeadsList, LeadCard, ApplicationsList, ApplicationForm, CriteriaEditor
    status: pending
  - id: routing
    content: 設定 React Router，建立主要路由：/, /leads, /applications, /criteria
    status: pending
  - id: data-migration
    content: 建立資料遷移腳本（scripts/migrate-data.ts），從 JSON/YAML 檔案匯入資料到 D1
    status: pending
  - id: testing
    content: 測試 API 端點、React UI 功能、Docker 容器執行
    status: pending
isProject: false
---

# ScholarshipOps Backend API 與 React UI 實施計劃

## 架構概覽

```
┌─────────────┐
│  React UI   │ (web-UI/)
│  (Vite)     │
└──────┬──────┘
       │ HTTP/REST API
       ▼
┌─────────────────────────────────┐
│  Cloudflare Workers + Hono      │ (container/)
│  Backend API                    │
├─────────────────────────────────┤
│  - /api/leads                   │
│  - /api/applications            │
│  - /api/criteria                │
│  - /api/stats                   │
│  - /api/trigger/search          │
│  - /api/trigger/schedule        │
│  - /api/trigger/track           │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│  Cloudflare D1 Database         │
│  (SQLite)                       │
├─────────────────────────────────┤
│  - leads                        │
│  - applications                 │
│  - criteria                     │
│  - sources                      │
└─────────────────────────────────┘
       │
       │ Triggers
       ▼
┌─────────────────────────────────┐
│  Scripts                        │
│  - Rust scraper                 │
│  - Go scheduler                 │
│  - Go tracker                   │
└─────────────────────────────────┘
```

## 實施步驟

### Phase 1: 後端 API 基礎架構 (container/)

#### 1.1 建立 Cloudflare Workers 專案結構

- 在 `container/` 目錄建立專案
- 初始化 `package.json` 與 TypeScript 設定
- 安裝依賴：`hono`, `@cloudflare/workers-types`, `wrangler`
- 建立 `wrangler.toml` 設定檔

#### 1.2 設計 D1 資料庫 Schema

建立 `container/migrations/0001_init.sql`：

- `leads` 表：儲存獎學金資訊（對應 `tracking/leads.json` 結構）
- `applications` 表：儲存申請追蹤（對應 `tracking/applications.json`）
- `criteria` 表：儲存搜尋條件與個人資料
- `sources` 表：儲存爬蟲來源設定
- `source_health` 表：追蹤來源健康狀態

主要欄位：

- `leads`: id, name, amount, deadline, source, source_type, status, match_score, bucket, url, eligibility (JSON), match_reasons (JSON), 等
- `applications`: id, name, deadline, status, current_stage, next_action, required_docs (JSON), progress, notes
- `criteria`: id, criteria_json, profile_json, updated_at

#### 1.3 實作 Hono API 路由

建立 `container/src/index.ts`：

- CORS middleware（允許 React UI 存取）
- 路由群組：
  - `GET /api/leads` - 列出所有獎學金
  - `GET /api/leads/:id` - 取得單一獎學金
  - `POST /api/leads` - 新增獎學金
  - `PUT /api/leads/:id` - 更新獎學金
  - `DELETE /api/leads/:id` - 刪除獎學金
  - `GET /api/applications` - 列出所有申請
  - `POST /api/applications` - 新增申請
  - `PUT /api/applications/:id` - 更新申請
  - `GET /api/criteria` - 取得搜尋條件
  - `PUT /api/criteria` - 更新搜尋條件
  - `GET /api/stats` - 取得統計資料（總數、進行中、已完成、即將到期）
  - `POST /api/trigger/search` - 觸發 Rust 爬蟲
  - `POST /api/trigger/schedule` - 觸發 Go 排程建議
  - `POST /api/trigger/track` - 觸發 Go 進度追蹤

#### 1.4 實作 D1 資料庫操作層

建立 `container/src/db/`：

- `leads.ts` - leads CRUD 操作
- `applications.ts` - applications CRUD 操作
- `criteria.ts` - criteria CRUD 操作
- `migrations.ts` - 遷移執行工具

#### 1.5 實作腳本觸發器

建立 `container/src/scripts/`：

- `rust-scraper.ts` - 呼叫 Rust 二進位檔執行爬蟲
- `go-scheduler.ts` - 呼叫 Go 二進位檔執行排程
- `go-tracker.ts` - 呼叫 Go 二進位檔執行追蹤
- 處理腳本輸出並更新 D1 資料庫

#### 1.6 建立 Dockerfile

在 `container/Dockerfile`：

- 多階段建置：
  - Stage 1: 建置 Rust 專案（`scripts/search_scholarships/`）
  - Stage 2: 建置 Go 專案（`scripts/schedule_applications/`, `scripts/track_progress/`）
  - Stage 3: Node.js runtime + 複製所有建置產物
- 安裝 wrangler CLI
- 設定環境變數
- 暴露端口 8787（Cloudflare Workers 預設）
- 啟動命令：`wrangler dev`（開發）或 `wrangler deploy`（生產）

#### 1.7 建立 .env.example

在 `container/.env.example`：

- `CLOUDFLARE_ACCOUNT_ID` - Cloudflare 帳號 ID
- `CLOUDFLARE_API_TOKEN` - API Token
- `D1_DATABASE_ID` - D1 資料庫 ID
- `D1_DATABASE_NAME` - D1 資料庫名稱
- `ENVIRONMENT` - 環境（development/production）
- `CORS_ORIGIN` - React UI 的來源 URL

### Phase 2: React UI (web-UI/)

#### 2.1 初始化 React 專案

- 使用 Vite + React + TypeScript
- 安裝依賴：`react`, `react-dom`, `react-router-dom`, `axios` 或 `fetch`
- 設定 Tailwind CSS（可選，用於樣式）

#### 2.2 建立 API Client

建立 `web-UI/src/api/`：

- `client.ts` - API 客戶端（axios 或 fetch wrapper）
- `leads.ts` - leads API 函數
- `applications.ts` - applications API 函數
- `criteria.ts` - criteria API 函數
- `stats.ts` - stats API 函數
- `triggers.ts` - 腳本觸發 API 函數

#### 2.3 定義 TypeScript 型別

建立 `web-UI/src/types/`：

- `lead.ts` - Lead 型別（對應後端）
- `application.ts` - Application 型別
- `criteria.ts` - Criteria 型別
- `stats.ts` - Statistics 型別

#### 2.4 建立主要 UI 元件

建立 `web-UI/src/components/`：

- `Dashboard.tsx` - 儀表板（顯示統計資料）
- `LeadsList.tsx` - 獎學金列表
- `LeadCard.tsx` - 單一獎學金卡片
- `LeadDetail.tsx` - 獎學金詳情頁
- `ApplicationsList.tsx` - 申請列表
- `ApplicationForm.tsx` - 新增/編輯申請表單
- `ApplicationCard.tsx` - 申請卡片
- `CriteriaEditor.tsx` - 搜尋條件編輯器
- `TriggerButtons.tsx` - 觸發腳本按鈕

#### 2.5 建立路由

建立 `web-UI/src/App.tsx` 與路由：

- `/` - Dashboard
- `/leads` - 獎學金列表
- `/leads/:id` - 獎學金詳情
- `/applications` - 申請列表
- `/applications/new` - 新增申請
- `/applications/:id` - 申請詳情
- `/criteria` - 搜尋條件設定

#### 2.6 實作狀態管理（可選）

- 使用 React Context 或 Zustand 管理全域狀態
- 快取 API 回應
- 實作樂觀更新

#### 2.7 建立環境設定

建立 `web-UI/.env.example`：

- `VITE_API_URL` - 後端 API URL（預設 `http://localhost:8787`）

### Phase 3: 整合與測試

#### 3.1 資料遷移腳本

建立 `container/scripts/migrate-data.ts`：

- 從 `tracking/leads.json` 匯入資料到 D1
- 從 `tracking/applications.json` 匯入資料到 D1
- 從 `tracking/criteria.yml` 匯入資料到 D1

#### 3.2 測試 API 端點

- 使用 curl 或 Postman 測試所有端點
- 驗證 CORS 設定
- 測試腳本觸發功能

#### 3.3 測試 React UI

- 驗證所有頁面正常載入
- 測試 CRUD 操作
- 測試腳本觸發按鈕

#### 3.4 Docker 整合測試

- 建置 Docker 映像
- 執行容器並測試 API
- 驗證 Rust/Go 腳本可在容器內執行

## 檔案結構

```
Smart_Zone/
├── container/                    # 後端 API
│   ├── src/
│   │   ├── index.ts             # Hono 應用程式入口
│   │   ├── routes/               # API 路由
│   │   │   ├── leads.ts
│   │   │   ├── applications.ts
│   │   │   ├── criteria.ts
│   │   │   ├── stats.ts
│   │   │   └── triggers.ts
│   │   ├── db/                   # D1 資料庫操作
│   │   │   ├── leads.ts
│   │   │   ├── applications.ts
│   │   │   ├── criteria.ts
│   │   │   └── migrations.ts
│   │   ├── scripts/               # 腳本觸發器
│   │   │   ├── rust-scraper.ts
│   │   │   ├── go-scheduler.ts
│   │   │   └── go-tracker.ts
│   │   └── types/                 # TypeScript 型別
│   ├── migrations/
│   │   └── 0001_init.sql         # D1 資料庫 schema
│   ├── Dockerfile
│   ├── wrangler.toml
│   ├── package.json
│   ├── tsconfig.json
│   └── .env.example
│
├── web-UI/                       # React UI
│   ├── src/
│   │   ├── api/                  # API 客戶端
│   │   ├── components/            # React 元件
│   │   ├── types/                 # TypeScript 型別
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── .env.example
│
└── [現有專案檔案...]
```

## 技術堆疊

### 後端

- **Runtime**: Cloudflare Workers
- **Framework**: Hono
- **Database**: Cloudflare D1 (SQLite)
- **Language**: TypeScript
- **Deployment**: Docker + Wrangler

### 前端

- **Framework**: React 18+
- **Build Tool**: Vite
- **Language**: TypeScript
- **HTTP Client**: Fetch API 或 Axios
- **Routing**: React Router

### 腳本整合

- **Rust**: 現有 `scripts/search_scholarships/`
- **Go**: 現有 `scripts/schedule_applications/`, `scripts/track_progress/`

## 注意事項

1. **D1 資料庫限制**：D1 是 SQLite，不支援某些進階 SQL 功能，需注意查詢語法
2. **Workers 執行時間限制**：腳本觸發可能需要非同步處理或使用 Queue
3. **CORS 設定**：確保後端正確設定 CORS 允許 React UI 存取
4. **環境變數**：所有敏感資訊應透過環境變數管理
5. **資料遷移**：首次部署需執行資料遷移腳本
6. **Docker 建置**：Rust 和 Go 建置可能較慢，考慮使用建置快取

## 後續優化

1. 實作 API 認證（如需要）
2. 新增 API 速率限制
3. 實作 WebSocket 即時更新
4. 新增單元測試與整合測試
5. 實作 API 文件（OpenAPI/Swagger）
6. 新增錯誤追蹤（Sentry）
7. 實作快取策略（Cloudflare KV）
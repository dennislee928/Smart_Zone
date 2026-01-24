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

## Agent Skills 使用指南

本計劃使用專案中的 Agent Skills 系統來確保最佳實踐和錯誤預防。在開始實施前，請先調用相關的 skills：

### 調用方式

```bash
# 單一 skill
npx openskills read cloudflare-worker-base

# 多個 skills（用逗號分隔）
npx openskills read cloudflare-worker-base,cloudflare-d1,hono-routing,drizzle-orm-d1
```

### 使用的 Agent Skills

#### 後端 Skills

1. **cloudflare-worker-base** - Cloudflare Workers 基礎設定

   - 用途：設定 Workers 專案結構、Vite 配置、wrangler.jsonc
   - 調用：`npx openskills read cloudflare-worker-base`
   - 預防錯誤：export 語法錯誤、路由衝突、Vite 8 相容性問題

2. **cloudflare-d1** - D1 資料庫操作

   - 用途：建立 D1 資料庫、撰寫 migrations、查詢最佳實踐
   - 調用：`npx openskills read cloudflare-d1`
   - 預防錯誤：D1_ERROR、statement too long、migration 失敗、查詢效能問題

3. **drizzle-orm-d1** - Drizzle ORM 與 D1 整合

   - 用途：型別安全的 schema 定義、migrations 管理、批次操作
   - 調用：`npx openskills read drizzle-orm-d1`
   - 預防錯誤：SQL BEGIN 失敗、cascade 資料遺失、100 參數限制、外鍵問題
   - **建議使用**：使用 Drizzle ORM 而非原生 SQL，提供更好的型別安全

4. **hono-routing** - Hono 路由與中介軟體

   - 用途：API 路由設定、請求驗證（Zod）、CORS、安全性
   - 調用：`npx openskills read hono-routing`
   - 預防錯誤：驗證 hooks、RPC 型別、中介軟體鏈、JWT 驗證演算法

5. **cloudflare-queues** - 非同步任務處理（可選）

   - 用途：腳本觸發的非同步處理、背景任務
   - 調用：`npx openskills read cloudflare-queues`
   - 使用場景：Rust/Go 腳本執行時間較長時，使用 Queue 避免 Workers 超時

#### 前端 Skills

6. **tanstack-query** - React 狀態管理與資料獲取

   - 用途：API 資料快取、樂觀更新、錯誤處理
   - 調用：`npx openskills read tanstack-query`
   - 建議使用：取代簡單的 fetch，提供更好的快取和狀態管理

7. **zustand-state-management** - 輕量級狀態管理（可選）

   - 用途：全域狀態管理（如使用者設定、UI 狀態）
   - 調用：`npx openskills read zustand-state-management`
   - 使用場景：需要簡單的全域狀態時使用

8. **tailwind-v4-shadcn** - Tailwind v4 + shadcn/ui（可選）

   - 用途：現代化 UI 元件庫、設計系統
   - 調用：`npx openskills read tailwind-v4-shadcn`
   - 使用場景：需要快速建立美觀 UI 時使用

#### 開發與測試 Skills

9. **test-driven-development** - 測試驅動開發

   - 用途：在實作功能前撰寫測試
   - 調用：`npx openskills read test-driven-development`
   - 使用時機：實作新功能或修復 bug 前

10. **verification-before-completion** - 完成前驗證

    - 用途：在聲稱工作完成前執行驗證命令
    - 調用：`npx openskills read verification-before-completion`
    - 使用時機：完成每個階段後驗證功能

### Skills 調用順序建議

**Phase 1 開始前：**

```bash
npx openskills read cloudflare-worker-base,cloudflare-d1,drizzle-orm-d1,hono-routing
```

**Phase 2 開始前：**

```bash
npx openskills read tanstack-query,tailwind-v4-shadcn
```

**測試階段：**

```bash
npx openskills read test-driven-development,verification-before-completion
```

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

**使用的 Agent Skills**: `cloudflare-worker-base`, `cloudflare-d1`, `drizzle-orm-d1`, `hono-routing`

**開始前調用**:

```bash
npx openskills read cloudflare-worker-base,cloudflare-d1,drizzle-orm-d1,hono-routing
```

#### 1.1 建立 Cloudflare Workers 專案結構

**使用 Skill**: `cloudflare-worker-base`

- 在 `container/` 目錄建立專案
- 參考 `cloudflare-worker-base` skill 的 Quick Start 指南
- 初始化 `package.json` 與 TypeScript 設定
- 安裝依賴：`hono@4.11.3`, `@cloudflare/workers-types`, `wrangler@4.54.0+`, `@cloudflare/vite-plugin@1.17.1`, `vite@7.3.1`
- 建立 `wrangler.jsonc` 設定檔（**注意**：使用 `.jsonc` 而非 `.toml`，參考 skill 說明）
- 建立 `vite.config.ts` 並配置 `@cloudflare/vite-plugin`
- **關鍵配置**：在 `wrangler.jsonc` 中加入 `run_worker_first: ["/api/*"]` 防止 SPA fallback 攔截 API 路由

#### 1.2 設計 D1 資料庫 Schema

**使用 Skills**: `cloudflare-d1`, `drizzle-orm-d1`

**建議使用 Drizzle ORM** 而非原生 SQL，提供型別安全和更好的開發體驗。

**選項 A：使用 Drizzle ORM（推薦）**

1. 安裝 Drizzle：`npm install drizzle-orm` 和 `npm install -D drizzle-kit`
2. 建立 `container/src/db/schema.ts` 定義 schema（參考 `drizzle-orm-d1` skill）
3. 配置 `drizzle.config.ts` 指向 D1
4. 使用 `npx drizzle-kit generate` 產生 migrations
5. 使用 `npx wrangler d1 migrations apply` 套用 migrations

**選項 B：使用原生 SQL**

1. 使用 `npx wrangler d1 create scholarshipops-db` 建立資料庫
2. 建立 `container/migrations/0001_init.sql`
3. 使用 `npx wrangler d1 migrations create` 建立 migration 檔案

**資料表設計**：

- `leads` 表：儲存獎學金資訊（對應 `tracking/leads.json` 結構）
  - 主要欄位：id (INTEGER PRIMARY KEY), name (TEXT), amount (TEXT), deadline (TEXT), source (TEXT), source_type (TEXT), status (TEXT), match_score (INTEGER), bucket (TEXT), url (TEXT), eligibility (TEXT JSON), match_reasons (TEXT JSON), 等
  - **注意**：D1 不支援原生 JSON 型別，使用 TEXT 儲存 JSON 字串
- `applications` 表：儲存申請追蹤（對應 `tracking/applications.json`）
  - 主要欄位：id (INTEGER PRIMARY KEY), name (TEXT), deadline (TEXT), status (TEXT), current_stage (TEXT), next_action (TEXT), required_docs (TEXT JSON), progress (INTEGER), notes (TEXT)
- `criteria` 表：儲存搜尋條件與個人資料
  - 主要欄位：id (INTEGER PRIMARY KEY), criteria_json (TEXT JSON), profile_json (TEXT JSON), updated_at (INTEGER timestamp)
- `sources` 表：儲存爬蟲來源設定
- `source_health` 表：追蹤來源健康狀態

**關鍵注意事項**（參考 `cloudflare-d1` skill）：

- 使用 `integer` 搭配 `mode: 'timestamp'` 儲存日期（D1 無原生 date 型別）
- 使用 `PRAGMA optimize` 優化資料庫
- 測試 migrations 時先使用 `--local` 再使用 `--remote`

#### 1.3 實作 Hono API 路由

**使用 Skill**: `hono-routing`

建立 `container/src/index.ts`：

**關鍵配置**（參考 `hono-routing` skill）：

- 使用 `export default app`（**不是** `{ fetch: app.fetch }`）
- 使用 `c.json()`, `c.text()`, `c.html()` 回傳回應
- 使用 `@hono/zod-validator` 進行請求驗證

**路由結構**：

```typescript
import { Hono } from 'hono'
import { cors } from 'hono/cors'
import { zValidator } from '@hono/zod-validator'
import { z } from 'zod'

const app = new Hono<{ Bindings: { DB: D1Database } }>()

// CORS middleware（允許 React UI 存取）
app.use('/*', cors({
  origin: ['http://localhost:5173', process.env.CORS_ORIGIN || '*'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization'],
}))

// 路由群組
app.get('/api/leads', ...)           // 列出所有獎學金
app.get('/api/leads/:id', ...)       // 取得單一獎學金
app.post('/api/leads', zValidator('json', leadSchema), ...)  // 新增獎學金（含驗證）
app.put('/api/leads/:id', ...)       // 更新獎學金
app.delete('/api/leads/:id', ...)    // 刪除獎學金

app.get('/api/applications', ...)    // 列出所有申請
app.post('/api/applications', ...)   // 新增申請
app.put('/api/applications/:id', ...) // 更新申請

app.get('/api/criteria', ...)       // 取得搜尋條件
app.put('/api/criteria', ...)       // 更新搜尋條件

app.get('/api/stats', ...)          // 取得統計資料（總數、進行中、已完成、即將到期）

app.post('/api/trigger/search', ...)    // 觸發 Rust 爬蟲
app.post('/api/trigger/schedule', ...) // 觸發 Go 排程建議
app.post('/api/trigger/track', ...)    // 觸發 Go 進度追蹤

export default app  // CRITICAL: 使用此模式
```

**請求驗證範例**（參考 `hono-routing` skill）：

```typescript
const leadSchema = z.object({
  name: z.string().min(1),
  amount: z.string(),
  deadline: z.string(),
  // ... 其他欄位
})

app.post('/api/leads', zValidator('json', leadSchema), async (c) => {
  const data = c.req.valid('json')  // 型別安全的資料
  // ... 處理邏輯
})
```

#### 1.4 實作 D1 資料庫操作層

**使用 Skills**: `cloudflare-d1`, `drizzle-orm-d1`

建立 `container/src/db/`：

**如果使用 Drizzle ORM**（推薦）：

- `schema.ts` - Drizzle schema 定義（已於 1.2 建立）
- `leads.ts` - 使用 `db.select().from(leads)` 等 Drizzle 查詢
- `applications.ts` - 使用 Drizzle 查詢
- `criteria.ts` - 使用 Drizzle 查詢
- **關鍵**：使用 `db.batch()` 進行交易（D1 不支援 SQL BEGIN/COMMIT）

**如果使用原生 SQL**：

- `leads.ts` - 使用 `env.DB.prepare()` 和 `env.DB.batch()` 進行 CRUD
- `applications.ts` - 原生 SQL 查詢
- `criteria.ts` - 原生 SQL 查詢
- `migrations.ts` - 遷移執行工具

**D1 查詢最佳實踐**（參考 `cloudflare-d1` skill）：

- 使用 prepared statements：`env.DB.prepare('SELECT * FROM leads WHERE id = ?').bind(id)`
- 批次操作使用 `env.DB.batch()` 而非多個獨立查詢
- 避免超過 100 個參數的查詢（使用批次 API）
- 使用索引優化查詢效能

#### 1.5 實作腳本觸發器

**使用 Skill**: `cloudflare-queues`（可選，用於長時間執行的腳本）

建立 `container/src/scripts/`：

- `rust-scraper.ts` - 呼叫 Rust 二進位檔執行爬蟲
- `go-scheduler.ts` - 呼叫 Go 二進位檔執行排程
- `go-tracker.ts` - 呼叫 Go 二進位檔執行追蹤
- 處理腳本輸出並更新 D1 資料庫

**執行方式選擇**：

**選項 A：直接執行（適合短時間腳本）**
- 在 Worker 中直接執行子進程（使用 `Deno.Command` 或類似 API）
- **限制**：Workers 有執行時間限制（免費版 10 秒，付費版 30 秒）

**選項 B：使用 Cloudflare Queues（推薦，適合長時間腳本）**
- 參考 `cloudflare-queues` skill
- 將腳本執行任務放入 Queue
- Queue consumer 執行腳本並更新資料庫
- **優點**：避免 Workers 超時，支援重試機制

**實作範例（直接執行）**：
```typescript
// container/src/scripts/rust-scraper.ts
export async function runRustScraper(env: Env): Promise<void> {
  // 執行 Rust 二進位檔
  const command = new Deno.Command('./target/release/search_scholarships', {
    env: { ROOT: '/app' },
  })
  const { stdout, stderr } = await command.output()
  
  // 解析輸出並更新 D1
  const output = new TextDecoder().decode(stdout)
  // ... 處理邏輯
}
```

**實作範例（使用 Queue）**：
```typescript
// 參考 cloudflare-queues skill
// 將任務放入 Queue，由 consumer 處理
```

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

**使用的 Agent Skills**: `tanstack-query`, `tailwind-v4-shadcn`（可選）, `zustand-state-management`（可選）

**開始前調用**:
```bash
npx openskills read tanstack-query,tailwind-v4-shadcn
```

#### 2.1 初始化 React 專案

- 使用 Vite + React + TypeScript
- 安裝依賴：
  - 核心：`react@18+`, `react-dom@18+`, `react-router-dom`
  - 資料獲取：`@tanstack/react-query@5+`（**推薦使用**，參考 `tanstack-query` skill）
  - HTTP 客戶端：`axios` 或使用原生 `fetch`（TanStack Query 已包含）
  - 樣式：`tailwindcss@4+`（參考 `tailwind-v4-shadcn` skill 設定）
  - UI 元件庫（可選）：`shadcn/ui`（參考 `tailwind-v4-shadcn` skill）
  - 狀態管理（可選）：`zustand`（參考 `zustand-state-management` skill）

#### 2.2 建立 API Client

**使用 Skill**: `tanstack-query`

建立 `web-UI/src/api/`：

- `client.ts` - API 客戶端（axios 或 fetch wrapper）
  - 設定 base URL：`import.meta.env.VITE_API_URL || 'http://localhost:8787'`
  - 設定預設 headers（Content-Type, Authorization 等）
  - 錯誤處理 middleware
- `leads.ts` - leads API 函數（使用 TanStack Query hooks）
- `applications.ts` - applications API 函數
- `criteria.ts` - criteria API 函數
- `stats.ts` - stats API 函數
- `triggers.ts` - 腳本觸發 API 函數

**TanStack Query 設定**（參考 `tanstack-query` skill）：
```typescript
// web-UI/src/main.tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 分鐘
      gcTime: 1000 * 60 * 10,   // 10 分鐘（v5 的 cacheTime）
    },
  },
})

// 使用範例
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

// 查詢
const { data, isLoading, error } = useQuery({
  queryKey: ['leads'],
  queryFn: () => fetchLeads(),
})

// 變更
const mutation = useMutation({
  mutationFn: createLead,
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['leads'] })
  },
})
```

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
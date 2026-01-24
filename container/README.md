# ScholarshipOps Backend API

Cloudflare Workers + Hono 後端 API，使用 D1 資料庫儲存獎學金和申請資料。

## 技術堆疊

- **Runtime**: Cloudflare Workers
- **Framework**: Hono 4.11.3+
- **Database**: Cloudflare D1 (SQLite) with Drizzle ORM
- **Language**: TypeScript
- **Build Tool**: Vite 7.3.1+

## 快速開始

### 1. 安裝依賴

```bash
cd container
npm install
```

### 2. 設定環境變數

複製 `.env.example` 為 `.env` 並填入實際值：

```bash
cp .env.example .env
```

編輯 `.env` 檔案，填入：

- `CLOUDFLARE_ACCOUNT_ID` - 您的 Cloudflare 帳號 ID
- `CLOUDFLARE_API_TOKEN` - API Token
- `D1_DATABASE_ID` - D1 資料庫 ID（執行 `wrangler d1 create scholarshipops-db` 取得）
- `D1_DATABASE_NAME` - 資料庫名稱（預設：`scholarshipops-db`）

### 3. 建立 D1 資料庫

```bash
# 建立資料庫
npx wrangler d1 create scholarshipops-db

# 輸出會包含 database_id，將其填入 wrangler.jsonc 和 .env
```

### 4. 更新 wrangler.jsonc

將 `wrangler.jsonc` 中的 `database_id` 更新為實際的資料庫 ID。

### 5. 產生並套用 Migrations

```bash
# 產生 migrations（使用 Drizzle Kit）
npm run db:generate

# 套用到本地資料庫（測試）
npm run db:migrate:local

# 套用到遠端資料庫（生產）
npm run db:migrate:remote
```

### 6. 啟動開發伺服器

```bash
npm run dev
```

API 將在 `http://localhost:8787` 啟動。

## API 端點

### Leads（獎學金）

- `GET /api/leads` - 列出所有獎學金（支援查詢參數：`status`, `bucket`, `search`）
- `GET /api/leads/:id` - 取得單一獎學金
- `POST /api/leads` - 新增獎學金
- `PUT /api/leads/:id` - 更新獎學金
- `DELETE /api/leads/:id` - 刪除獎學金

### Applications（申請）

- `GET /api/applications` - 列出所有申請
- `GET /api/applications/:id` - 取得單一申請
- `POST /api/applications` - 新增申請
- `PUT /api/applications/:id` - 更新申請
- `DELETE /api/applications/:id` - 刪除申請

### Criteria（搜尋條件）

- `GET /api/criteria` - 取得搜尋條件
- `PUT /api/criteria` - 更新搜尋條件

### Stats（統計）

- `GET /api/stats` - 取得統計資料（總數、進行中、已完成、即將到期）

### Triggers（腳本觸發）

- `POST /api/trigger/search` - 觸發 Rust 爬蟲
- `POST /api/trigger/schedule` - 觸發 Go 排程建議
- `POST /api/trigger/track` - 觸發 Go 進度追蹤

## 資料遷移

從現有的 JSON/YAML 檔案匯入資料到 D1：

```bash
# 執行遷移腳本（需要在 Node.js 環境中執行，非 Workers）
npx tsx src/scripts/migrate-data.ts
```

## Docker 部署

### 建置映像

```bash
docker build -t scholarshipops-api -f container/Dockerfile .
```

### 執行容器

```bash
docker run -p 8787:8787 \
  -e CLOUDFLARE_ACCOUNT_ID=your_account_id \
  -e CLOUDFLARE_API_TOKEN=your_token \
  -e D1_DATABASE_ID=your_database_id \
  scholarshipops-api
```

## 部署到 Cloudflare

```bash
npm run deploy
```

## 開發指令

- `npm run dev` - 啟動本地開發伺服器
- `npm run deploy` - 部署到 Cloudflare
- `npm run db:generate` - 產生 Drizzle migrations
- `npm run db:migrate:local` - 套用 migrations 到本地資料庫
- `npm run db:migrate:remote` - 套用 migrations 到遠端資料庫
- `npm run db:studio` - 開啟 Drizzle Studio（視覺化資料庫瀏覽器）

## 專案結構

```
container/
├── src/
│   ├── index.ts           # Hono 應用程式入口
│   ├── routes/            # API 路由
│   ├── db/                # D1 資料庫操作（Drizzle ORM）
│   ├── scripts/           # 腳本觸發器
│   └── types/             # TypeScript 型別定義
├── migrations/            # Drizzle migrations
├── Dockerfile             # Docker 建置檔案
├── wrangler.jsonc         # Wrangler 設定檔
└── package.json
```

## 注意事項

1. **D1 資料庫限制**：

   - 不支援 SQL `BEGIN TRANSACTION`，需使用 `db.batch()`（Drizzle）
   - 使用 `integer` 搭配 `mode: 'timestamp'` 儲存日期
   - 避免超過 100 個參數的查詢
2. **Workers 執行時間限制**：

   - 免費版：10 秒 CPU 時間
   - 付費版：30 秒 CPU 時間
   - 長時間腳本應使用 Cloudflare Queues
3. **Export 語法**：

   - **必須**使用 `export default app`，**不要**使用 `{ fetch: app.fetch }`
4. **Wrangler 配置**：

   - 使用 `wrangler.jsonc` 而非 `wrangler.toml`
   - 設定 `run_worker_first: ["/api/*"]` 防止 SPA fallback 攔截 API 路由

## Agent Skills 參考

本專案使用以下 Agent Skills（參考 `AGENTS.md`）：

- `cloudflare-worker-base` - Workers 基礎設定
- `cloudflare-d1` - D1 資料庫操作
- `drizzle-orm-d1` - Drizzle ORM 整合
- `hono-routing` - Hono 路由與驗證

調用方式：

```bash
npx openskills read cloudflare-worker-base,cloudflare-d1,drizzle-orm-d1,hono-routing
```

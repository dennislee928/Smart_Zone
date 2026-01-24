# ScholarshipOps API & UI - 快速開始指南

## 概述

本專案包含兩個主要部分：
1. **container/** - Cloudflare Workers 後端 API（Hono + D1）
2. **web-UI/** - React 前端 UI（Vite + TanStack Query）

## 前置需求

- Node.js 18+
- npm 或 pnpm
- Cloudflare 帳號（用於 D1 資料庫）
- Docker（可選，用於容器化部署）

## 後端 API 設定

### 1. 安裝依賴

```bash
cd container
npm install
```

### 2. 建立 D1 資料庫

```bash
npx wrangler d1 create scholarshipops-db
```

記下輸出的 `database_id`，需要填入 `wrangler.jsonc` 和 `.env`。

### 3. 設定環境變數

```bash
cp .env.example .env
```

編輯 `.env`，填入：
- `CLOUDFLARE_ACCOUNT_ID`
- `CLOUDFLARE_API_TOKEN`
- `D1_DATABASE_ID`（從步驟 2 取得）

### 4. 更新 wrangler.jsonc

將 `wrangler.jsonc` 中的 `database_id` 更新為實際的資料庫 ID。

### 5. 產生並套用 Migrations

```bash
# 產生 migrations
npm run db:generate

# 套用到本地（測試）
npm run db:migrate:local

# 套用到遠端（生產）
npm run db:migrate:remote
```

### 6. 啟動開發伺服器

```bash
npm run dev
```

API 將在 `http://localhost:8787` 啟動。

## 前端 UI 設定

### 1. 安裝依賴

```bash
cd web-UI
npm install
```

### 2. 設定環境變數

```bash
cp .env.example .env
```

編輯 `.env`，確認 `VITE_API_URL` 指向後端 API（預設：`http://localhost:8787`）。

### 3. 啟動開發伺服器

```bash
npm run dev
```

UI 將在 `http://localhost:5173` 啟動。

## 資料遷移

從現有的 JSON/YAML 檔案匯入資料：

```bash
cd container
# 注意：這需要在 Node.js 環境執行，非 Workers 環境
# 可能需要調整路徑或使用不同的執行方式
npx tsx src/scripts/migrate-data.ts
```

## Docker 部署

### 建置後端映像

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

## API 測試

### 測試健康檢查

```bash
curl http://localhost:8787/
```

### 測試 Leads API

```bash
# 取得所有獎學金
curl http://localhost:8787/api/leads

# 取得單一獎學金
curl http://localhost:8787/api/leads/1

# 新增獎學金
curl -X POST http://localhost:8787/api/leads \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Scholarship","status":"qualified"}'
```

### 測試 Stats API

```bash
curl http://localhost:8787/api/stats
```

## 常見問題

### Q: D1 migrations 失敗？

A: 確保：
1. `database_id` 在 `wrangler.jsonc` 中正確設定
2. 使用 `--local` 先測試，再使用 `--remote`
3. 檢查 Cloudflare API token 權限

### Q: CORS 錯誤？

A: 檢查 `container/src/index.ts` 中的 CORS 設定，確保包含前端 URL。

### Q: Workers 執行時間超時？

A: 長時間執行的腳本應使用 Cloudflare Queues（參考 `cloudflare-queues` skill）。

## 下一步

1. 實作腳本觸發器的實際執行邏輯（使用 Cloudflare Queues）
2. 新增 API 認證（如需要）
3. 實作更完整的錯誤處理
4. 新增單元測試和整合測試

## 參考文件

- [後端 API 文件](container/README.md)
- [前端 UI 文件](web-UI/README.md)
- [實施計劃](.cursor/plans/scholarshipops_backend_api_and_react_ui_981f3623.plan.md)
- [Agent Skills](AGENTS.md)

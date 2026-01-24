# ScholarshipOps - Backend API & React UI

完整的獎學金管理系統，包含 Cloudflare Workers 後端 API 和 React 前端 UI。

## 專案結構

```
Smart_Zone/
├── container/          # 後端 API（Cloudflare Workers + Hono + D1）
├── web-UI/            # React 前端 UI
├── scripts/           # Rust/Go 腳本（爬蟲、排程、追蹤）
└── tracking/          # 資料檔案（JSON/YAML）
```

## 快速開始

### 後端 API

```bash
cd container
npm install
cp .env.example .env
# 編輯 .env 填入 Cloudflare 設定
npm run dev
```

### 前端 UI

```bash
cd web-UI
npm install
cp .env.example .env
npm run dev
```

## 功能

- ✅ 獎學金（Leads）CRUD API
- ✅ 申請（Applications）CRUD API
- ✅ 搜尋條件（Criteria）管理
- ✅ 統計資料 API
- ✅ 腳本觸發端點
- ✅ React UI 完整介面
- ✅ TanStack Query 資料快取
- ✅ Docker 容器化支援

## 部署

### Docker

```bash
# 建置後端映像
docker build -t scholarshipops-api -f container/Dockerfile .

# 執行容器
docker run -p 8787:8787 \
  -e CLOUDFLARE_ACCOUNT_ID=your_account_id \
  -e CLOUDFLARE_API_TOKEN=your_token \
  -e D1_DATABASE_ID=your_database_id \
  scholarshipops-api
```

### Cloudflare Workers

```bash
cd container
npm run deploy
```

## Agent Skills

本專案使用專案中的 Agent Skills 系統確保最佳實踐：

**後端 Skills**:
- `cloudflare-worker-base`
- `cloudflare-d1`
- `drizzle-orm-d1`
- `hono-routing`

**前端 Skills**:
- `tanstack-query`
- `tailwind-v4-shadcn`

詳細資訊請參考 `AGENTS.md`。

## 文件

- [後端 API 文件](container/README.md)
- [前端 UI 文件](web-UI/README.md)
- [實施計劃](.cursor/plans/scholarshipops_backend_api_and_react_ui_981f3623.plan.md)

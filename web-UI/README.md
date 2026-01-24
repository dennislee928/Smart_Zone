# ScholarshipOps React UI

React + Vite + TypeScript 前端應用程式，使用 TanStack Query 進行資料獲取和狀態管理。

## 技術堆疊

- **Framework**: React 18+
- **Build Tool**: Vite
- **Language**: TypeScript
- **資料獲取**: TanStack Query 5+
- **HTTP Client**: Axios
- **Routing**: React Router
- **樣式**: Tailwind CSS v4

## 快速開始

### 1. 安裝依賴

```bash
cd web-UI
npm install
```

### 2. 設定環境變數

複製 `.env.example` 為 `.env`：

```bash
cp .env.example .env
```

編輯 `.env`，設定後端 API URL（預設：`http://localhost:8787`）：

```
VITE_API_URL=http://localhost:8787
```

### 3. 啟動開發伺服器

```bash
npm run dev
```

應用程式將在 `http://localhost:5173` 啟動。

### 4. 建置生產版本

```bash
npm run build
```

建置產物將在 `dist/` 目錄。

## 專案結構

```
web-UI/
├── src/
│   ├── api/              # API 客戶端（TanStack Query hooks）
│   ├── pages/            # 頁面元件
│   ├── types/            # TypeScript 型別定義
│   ├── App.tsx           # 主應用程式元件
│   └── main.tsx          # 應用程式入口
├── index.html
├── vite.config.ts
└── package.json
```

## 頁面路由

- `/` - Dashboard（儀表板）
- `/leads` - 獎學金列表
- `/leads/:id` - 獎學金詳情
- `/applications` - 申請列表
- `/applications/new` - 新增申請
- `/applications/:id` - 申請詳情
- `/criteria` - 搜尋條件設定

## API 整合

使用 TanStack Query 進行資料獲取：

```typescript
import { useLeads } from './api/leads'

function LeadsList() {
  const { data: leads, isLoading, error } = useLeads()
  // ...
}
```

## 開發指令

- `npm run dev` - 啟動開發伺服器
- `npm run build` - 建置生產版本
- `npm run preview` - 預覽生產版本
- `npm run lint` - 執行 ESLint

## Agent Skills 參考

本專案使用以下 Agent Skills（參考 `AGENTS.md`）：

- `tanstack-query` - React 資料獲取與快取
- `tailwind-v4-shadcn` - Tailwind v4 + shadcn/ui（可選）

調用方式：
```bash
npx openskills read tanstack-query,tailwind-v4-shadcn
```

# Robot Framework API 測試套件

本目錄包含 ScholarshipOps API 的完整 Robot Framework 測試套件。

## 安裝依賴

```bash
pip install -r requirements.txt
```

## 執行測試

### 執行所有測試
```bash
npm run test
# 或
robot --outputdir output suites/
```

### 執行特定測試套件
```bash
npm run test:health      # Health check 測試
npm run test:leads       # Leads API 測試
npm run test:applications # Applications API 測試
npm run test:criteria    # Criteria API 測試
npm run test:stats       # Stats API 測試
npm run test:triggers    # Triggers API 測試
```

### 生成 HTML 報告
```bash
npm run test:report
# 報告會生成在 output/report.html
```

## 測試結構

```
test/
├── variables/
│   └── config.robot          # 測試配置（API URL、超時等）
├── resources/
│   ├── api_keywords.robot    # API 請求關鍵字庫
│   ├── fixtures.robot        # 測試資料設置和清理
│   └── validators.robot      # 響應驗證關鍵字
└── suites/
    ├── health_check.robot    # Health check 測試
    ├── leads.robot           # Leads API 測試
    ├── applications.robot    # Applications API 測試
    ├── criteria.robot        # Criteria API 測試
    ├── stats.robot           # Stats API 測試
    └── triggers.robot        # Triggers API 測試
```

## 測試資料管理

測試使用 fixtures 模式確保測試隔離：

1. **Setup**: 每個測試套件開始前會清理資料庫
2. **Test Data**: 測試中使用 `Create Test Lead/Application/Criteria` 建立測試資料
3. **Teardown**: 測試結束後清理所有測試資料

## 前置條件

- API 伺服器必須在 `http://localhost:8787` 運行
- 建議使用本地開發資料庫（測試會修改資料庫內容）
- 確保資料庫已執行 migration

## 測試報告

測試執行後會生成：
- `log.html` - 詳細測試日誌
- `report.html` - 測試報告摘要
- `output.xml` - XML 格式測試結果（用於 CI/CD）

## 注意事項

- 測試之間應該獨立，不依賴執行順序
- 使用 Robot Framework 的 `[Setup]` 和 `[Teardown]` 確保測試隔離
- 所有測試資料會在測試結束後自動清理

# API 基礎配置
API_BASE_URL = 'http://localhost:8787'
API_TIMEOUT = 10  # 秒數（數字類型，RequestsLibrary 需要）

# API 端點路徑
API_HEALTH = f'{API_BASE_URL}/'
API_LEADS = f'{API_BASE_URL}/api/leads'
API_APPLICATIONS = f'{API_BASE_URL}/api/applications'
API_CRITERIA = f'{API_BASE_URL}/api/criteria'
API_STATS = f'{API_BASE_URL}/api/stats'
API_TRIGGER_SEARCH = f'{API_BASE_URL}/api/trigger/search'
API_TRIGGER_SCHEDULE = f'{API_BASE_URL}/api/trigger/schedule'
API_TRIGGER_TRACK = f'{API_BASE_URL}/api/trigger/track'

# HTTP 狀態碼
STATUS_OK = 200
STATUS_CREATED = 201
STATUS_ACCEPTED = 202
STATUS_BAD_REQUEST = 400
STATUS_NOT_FOUND = 404
STATUS_INTERNAL_ERROR = 500

# 測試資料變數
TEST_LEAD_NAME = 'Test Scholarship'
TEST_LEAD_AMOUNT = '5000'
TEST_LEAD_STATUS = 'qualified'
TEST_LEAD_BUCKET = 'test-bucket'

TEST_APPLICATION_NAME = 'Test Application'
TEST_APPLICATION_STATUS = 'not_started'

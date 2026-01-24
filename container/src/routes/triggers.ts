import { Hono } from 'hono'
import type { Env } from '../types'

const app = new Hono<{ Bindings: Env }>()

// POST /api/trigger/search - 觸發 Rust 爬蟲
app.post('/search', async (c) => {
  // TODO: 實作 Rust 爬蟲觸發邏輯
  // 這需要執行 scripts/search_scholarships 二進位檔
  // 可以使用 Cloudflare Queues 進行非同步處理
  
  return c.json({ 
    message: 'Search trigger received',
    status: 'pending',
    note: 'Implementation pending - requires script execution setup'
  }, 202)
})

// POST /api/trigger/schedule - 觸發 Go 排程建議
app.post('/schedule', async (c) => {
  // TODO: 實作 Go scheduler 觸發邏輯
  // 這需要執行 scripts/schedule_applications 二進位檔
  
  return c.json({ 
    message: 'Schedule trigger received',
    status: 'pending',
    note: 'Implementation pending - requires script execution setup'
  }, 202)
})

// POST /api/trigger/track - 觸發 Go 進度追蹤
app.post('/track', async (c) => {
  // TODO: 實作 Go tracker 觸發邏輯
  // 這需要執行 scripts/track_progress 二進位檔
  
  return c.json({ 
    message: 'Track trigger received',
    status: 'pending',
    note: 'Implementation pending - requires script execution setup'
  }, 202)
})

export default app

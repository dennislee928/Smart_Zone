import { Hono } from 'hono'
import { getDb } from '../db/index'
import * as statsDb from '../db/stats'
import type { Env } from '../types'

const app = new Hono<{ Bindings: Env }>()

// GET /api/stats - 取得統計資料
app.get('/', async (c) => {
  const db = getDb(c.env.DB)
  const stats = await statsDb.getStats(db)
  return c.json({ stats })
})

export default app

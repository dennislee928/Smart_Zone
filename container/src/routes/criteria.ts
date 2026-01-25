import { Hono } from 'hono'
import { zValidator } from '@hono/zod-validator'
import { z } from 'zod'
import { getDb } from '../db/index'
import * as criteriaDb from '../db/criteria'
import type { Env } from '../types'

const app = new Hono<{ Bindings: Env }>()

// Schema for criteria validation
const criteriaSchema = z.object({
  criteriaJson: z.object({
    required: z.array(z.string()),
    preferred: z.array(z.string()),
    excluded_keywords: z.array(z.string()),
  }).optional(),
  profileJson: z.object({
    nationality: z.string(),
    target_university: z.string(),
    target_country: z.string(),
    programme_level: z.string(),
    programme_start: z.string(),
    education: z.array(z.object({
      degree: z.string(),
      university: z.string(),
      department: z.string(),
      gpa: z.number(),
      gpa_scale: z.number(),
      status: z.string(),
    })),
    min_deadline: z.string().optional(),
    max_gpa_requirement: z.number().optional(),
  }).optional(),
})

// GET /api/criteria - 取得搜尋條件
app.get('/', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const criteria = await criteriaDb.getCriteria(db)
    
    if (!criteria) {
      return c.json({ criteria: null })
    }

    return c.json({ criteria })
  } catch (error) {
    console.error('Error getting criteria:', error)
    return c.json({ error: 'Failed to get criteria', details: String(error) }, 500)
  }
})

// PUT /api/criteria - 更新搜尋條件
app.put('/', zValidator('json', criteriaSchema, (result, c) => {
  if (!result.success) {
    console.error('Validation error:', result.error.errors)
    return c.json({ 
      error: 'Validation failed',
      details: result.error.errors 
    }, 400)
  }
}), async (c) => {
  const db = getDb(c.env.DB)
  const data = c.req.valid('json')

  try {
    const criteria = await criteriaDb.createOrUpdateCriteria(db, data)
    return c.json({ criteria })
  } catch (error) {
    console.error('Error updating criteria:', error)
    return c.json({ error: 'Failed to update criteria', details: String(error) }, 500)
  }
})

export default app

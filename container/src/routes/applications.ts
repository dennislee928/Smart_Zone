import { Hono } from 'hono'
import { zValidator } from '@hono/zod-validator'
import { z } from 'zod'
import { getDb } from '../db/index'
import * as applicationsDb from '../db/applications'
import type { Env } from '../types'

const app = new Hono<{ Bindings: Env }>()

// Schema for application validation
const applicationSchema = z.object({
  name: z.string().min(1),
  deadline: z.string().optional(),
  status: z.string().default('not_started'),
  currentStage: z.string().optional(),
  nextAction: z.string().optional(),
  requiredDocs: z.array(z.string()).optional(),
  progress: z.number().min(0).max(100).optional(),
  notes: z.string().optional(),
})

// GET /api/applications - 列出所有申請
app.get('/', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const applications = await applicationsDb.getAllApplications(db)
    return c.json({ applications })
  } catch (error) {
    console.error('Error getting applications:', error)
    return c.json({ error: 'Failed to get applications', details: String(error) }, 500)
  }
})

// GET /api/applications/:id - 取得單一申請
app.get('/:id', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const id = parseInt(c.req.param('id'))

    if (isNaN(id)) {
      return c.json({ error: 'Invalid ID' }, 400)
    }

    const application = await applicationsDb.getApplicationById(db, id)
    if (!application) {
      return c.json({ error: 'Application not found' }, 404)
    }

    return c.json({ application })
  } catch (error) {
    console.error('Error getting application:', error)
    return c.json({ error: 'Failed to get application', details: String(error) }, 500)
  }
})

// POST /api/applications - 新增申請
app.post('/', zValidator('json', applicationSchema, (result, c) => {
  if (!result.success) {
    console.error('Validation error:', result.error.errors)
    return c.json({ 
      error: 'Validation failed',
      details: result.error.errors 
    }, 400)
  }
}), async (c) => {
  try {
    const db = getDb(c.env.DB)
    const data = c.req.valid('json')

    const application = await applicationsDb.createApplication(db, data)
    return c.json({ application }, 201)
  } catch (error) {
    console.error('Error creating application:', error)
    return c.json({ error: 'Failed to create application', details: String(error) }, 500)
  }
})

// PUT /api/applications/:id - 更新申請
app.put('/:id', zValidator('json', applicationSchema.partial(), (result, c) => {
  if (!result.success) {
    console.error('Validation error:', result.error.errors)
    return c.json({ 
      error: 'Validation failed',
      details: result.error.errors 
    }, 400)
  }
}), async (c) => {
  try {
    const db = getDb(c.env.DB)
    const id = parseInt(c.req.param('id'))
    const data = c.req.valid('json')

    if (isNaN(id)) {
      return c.json({ error: 'Invalid ID' }, 400)
    }

    const application = await applicationsDb.updateApplication(db, id, data)
    if (!application) {
      return c.json({ error: 'Application not found' }, 404)
    }
    return c.json({ application })
  } catch (error) {
    console.error('Error updating application:', error)
    return c.json({ error: 'Failed to update application', details: String(error) }, 500)
  }
})

// DELETE /api/applications/:id - 刪除申請
app.delete('/:id', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const id = parseInt(c.req.param('id'))

    if (isNaN(id)) {
      return c.json({ error: 'Invalid ID' }, 400)
    }

    const deleted = await applicationsDb.deleteApplication(db, id)
    if (!deleted) {
      return c.json({ error: 'Application not found' }, 404)
    }

    return c.json({ success: true })
  } catch (error) {
    console.error('Error deleting application:', error)
    return c.json({ error: 'Failed to delete application', details: String(error) }, 500)
  }
})

export default app

import { Hono } from 'hono'
import { zValidator } from '@hono/zod-validator'
import { z } from 'zod'
import { getDb } from '../db/index'
import * as leadsDb from '../db/leads'
import type { Env } from '../types'

const app = new Hono<{ Bindings: Env }>()

// Schema for lead validation
const leadSchema = z.object({
  name: z.string().min(1),
  amount: z.string().optional(),
  deadline: z.string().optional(),
  source: z.string().optional(),
  sourceType: z.string().optional(),
  status: z.string().default('qualified'),
  eligibility: z.array(z.string()).optional(),
  notes: z.string().optional(),
  addedDate: z.string().optional(),
  url: z.string().optional(),
  matchScore: z.number().optional(),
  matchReasons: z.array(z.string()).optional(),
  hardFailReasons: z.array(z.string()).optional(),
  softFlags: z.array(z.string()).optional(),
  bucket: z.string().optional(),
  httpStatus: z.number().optional(),
  effortScore: z.number().optional(),
  trustTier: z.string().optional(),
  riskFlags: z.array(z.string()).optional(),
  matchedRuleIds: z.array(z.string()).optional(),
  eligibleCountries: z.array(z.string()).optional(),
  isTaiwanEligible: z.boolean().optional(),
  taiwanEligibilityConfidence: z.string().optional(),
  deadlineDate: z.string().optional(),
  deadlineLabel: z.string().optional(),
  intakeYear: z.string().optional(),
  studyStart: z.string().optional(),
  deadlineConfidence: z.string().optional(),
  canonicalUrl: z.string().optional(),
  isDirectoryPage: z.boolean().optional(),
  officialSourceUrl: z.string().optional(),
  sourceDomain: z.string().optional(),
  confidence: z.number().optional(),
  eligibilityConfidence: z.number().optional(),
  tags: z.array(z.string()).optional(),
  isIndexOnly: z.boolean().optional(),
  firstSeenAt: z.string().optional(),
  lastCheckedAt: z.string().optional(),
  nextCheckAt: z.string().optional(),
  persistenceStatus: z.string().optional(),
  sourceSeed: z.string().optional(),
  checkCount: z.number().optional(),
})

// GET /api/leads - 列出所有獎學金
app.get('/', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const status = c.req.query('status')
    const bucket = c.req.query('bucket')
    const search = c.req.query('search')

    const filters = {
      ...(status && { status }),
      ...(bucket && { bucket }),
      ...(search && { search }),
    }

    const leads = await leadsDb.getAllLeads(db, Object.keys(filters).length > 0 ? filters : undefined)
    return c.json({ leads })
  } catch (error) {
    console.error('Error getting leads:', error)
    return c.json({ error: 'Failed to get leads', details: String(error) }, 500)
  }
})

// GET /api/leads/:id - 取得單一獎學金
app.get('/:id', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const id = parseInt(c.req.param('id'))

    if (isNaN(id)) {
      return c.json({ error: 'Invalid ID' }, 400)
    }

    const lead = await leadsDb.getLeadById(db, id)
    if (!lead) {
      return c.json({ error: 'Lead not found' }, 404)
    }

    return c.json({ lead })
  } catch (error) {
    console.error('Error getting lead:', error)
    return c.json({ error: 'Failed to get lead', details: String(error) }, 500)
  }
})

// POST /api/leads - 新增獎學金
app.post('/', zValidator('json', leadSchema, (result, c) => {
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

    const lead = await leadsDb.createLead(db, data)
    return c.json({ lead }, 201)
  } catch (error) {
    console.error('Error creating lead:', error)
    return c.json({ error: 'Failed to create lead', details: String(error) }, 500)
  }
})

// PUT /api/leads/:id - 更新獎學金
app.put('/:id', zValidator('json', leadSchema.partial(), (result, c) => {
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

    const lead = await leadsDb.updateLead(db, id, data)
    if (!lead) {
      return c.json({ error: 'Lead not found' }, 404)
    }
    return c.json({ lead })
  } catch (error) {
    console.error('Error updating lead:', error)
    return c.json({ error: 'Failed to update lead', details: String(error) }, 500)
  }
})

// DELETE /api/leads/:id - 刪除獎學金
app.delete('/:id', async (c) => {
  try {
    const db = getDb(c.env.DB)
    const id = parseInt(c.req.param('id'))

    if (isNaN(id)) {
      return c.json({ error: 'Invalid ID' }, 400)
    }

    const deleted = await leadsDb.deleteLead(db, id)
    if (!deleted) {
      return c.json({ error: 'Lead not found' }, 404)
    }

    return c.json({ success: true })
  } catch (error) {
    console.error('Error deleting lead:', error)
    return c.json({ error: 'Failed to delete lead', details: String(error) }, 500)
  }
})

export default app

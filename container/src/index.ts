import { Hono } from 'hono'
import { cors } from 'hono/cors'
import leadsRoutes from './routes/leads'
import applicationsRoutes from './routes/applications'
import criteriaRoutes from './routes/criteria'
import statsRoutes from './routes/stats'
import triggersRoutes from './routes/triggers'
import type { Env } from './types'

const app = new Hono<{ Bindings: Env }>()

// CORS middleware（允許 React UI 存取）
app.use('/*', cors({
  origin: ['http://localhost:5173', 'http://localhost:3000', '*'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization'],
  credentials: true,
}))

// Health check
app.get('/', (c) => {
  return c.json({ 
    message: 'ScholarshipOps API',
    version: '0.1.0',
    status: 'ok'
  })
})

// API routes
app.route('/api/leads', leadsRoutes)
app.route('/api/applications', applicationsRoutes)
app.route('/api/criteria', criteriaRoutes)
app.route('/api/stats', statsRoutes)
app.route('/api/trigger', triggersRoutes)

// 404 handler
app.notFound((c) => {
  return c.json({ error: 'Not found' }, 404)
})

// Error handler
app.onError((err, c) => {
  console.error('Error:', err)
  return c.json({ 
    error: 'Internal server error',
    message: err.message 
  }, 500)
})

// CRITICAL: Use export default app (NOT { fetch: app.fetch })
export default app

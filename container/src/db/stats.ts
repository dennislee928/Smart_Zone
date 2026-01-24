import { eq, and, gte, lte } from 'drizzle-orm'
import { getDb } from './index'
import { leads, applications } from './schema'
import type { Stats } from '../types'

export async function getStats(db: ReturnType<typeof getDb>): Promise<Stats> {
  // Get all leads and applications
  const allLeads = await db.select().from(leads).all()
  const allApplications = await db.select().from(applications).all()

  // Calculate stats
  const now = new Date()
  const sevenDaysFromNow = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000)
  const fourteenDaysFromNow = new Date(now.getTime() + 14 * 24 * 60 * 60 * 1000)
  const twentyOneDaysFromNow = new Date(now.getTime() + 21 * 24 * 60 * 60 * 1000)

  const inProgress = allApplications.filter(app => app.status === 'in_progress').length
  const completed = allApplications.filter(app => 
    ['submitted', 'accepted', 'rejected'].includes(app.status)
  ).length
  const notStarted = allApplications.filter(app => app.status === 'not_started').length

  // Count upcoming deadlines
  let upcoming7 = 0
  let upcoming14 = 0
  let upcoming21 = 0

  for (const app of allApplications) {
    if (app.deadline) {
      const deadline = new Date(app.deadline)
      if (deadline > now && deadline <= sevenDaysFromNow) {
        upcoming7++
      } else if (deadline > sevenDaysFromNow && deadline <= fourteenDaysFromNow) {
        upcoming14++
      } else if (deadline > fourteenDaysFromNow && deadline <= twentyOneDaysFromNow) {
        upcoming21++
      }
    }
  }

  return {
    totalLeads: allLeads.length,
    totalApplications: allApplications.length,
    inProgress,
    completed,
    notStarted,
    upcoming7,
    upcoming14,
    upcoming21,
  }
}

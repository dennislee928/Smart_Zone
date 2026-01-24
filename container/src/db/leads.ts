import { eq, desc, and, or, like } from 'drizzle-orm'
import { getDb } from './index'
import { leads } from './schema'
import type { Lead } from '../types'

export async function getAllLeads(db: ReturnType<typeof getDb>, filters?: {
  status?: string
  bucket?: string
  search?: string
}): Promise<Lead[]> {
  let query = db.select().from(leads)

  if (filters) {
    const conditions = []
    if (filters.status) {
      conditions.push(eq(leads.status, filters.status))
    }
    if (filters.bucket) {
      conditions.push(eq(leads.bucket, filters.bucket))
    }
    if (filters.search) {
      conditions.push(
        or(
          like(leads.name, `%${filters.search}%`),
          like(leads.source, `%${filters.search}%`)
        )!
      )
    }
    if (conditions.length > 0) {
      query = query.where(and(...conditions))
    }
  }

  const results = await query.orderBy(desc(leads.matchScore), desc(leads.createdAt)).all()
  return results.map(mapLeadFromDb)
}

export async function getLeadById(db: ReturnType<typeof getDb>, id: number): Promise<Lead | null> {
  const result = await db.select().from(leads).where(eq(leads.id, id)).limit(1).get()
  return result ? mapLeadFromDb(result) : null
}

export async function createLead(db: ReturnType<typeof getDb>, lead: Omit<Lead, 'id' | 'createdAt' | 'updatedAt'>): Promise<Lead> {
  const result = await db.insert(leads).values({
    name: lead.name,
    amount: lead.amount,
    deadline: lead.deadline,
    source: lead.source,
    sourceType: lead.sourceType,
    status: lead.status || 'qualified',
    eligibility: lead.eligibility,
    notes: lead.notes,
    addedDate: lead.addedDate,
    url: lead.url,
    matchScore: lead.matchScore,
    matchReasons: lead.matchReasons,
    hardFailReasons: lead.hardFailReasons,
    softFlags: lead.softFlags,
    bucket: lead.bucket,
    httpStatus: lead.httpStatus,
    effortScore: lead.effortScore,
    trustTier: lead.trustTier,
    riskFlags: lead.riskFlags,
    matchedRuleIds: lead.matchedRuleIds,
    eligibleCountries: lead.eligibleCountries,
    isTaiwanEligible: lead.isTaiwanEligible,
    taiwanEligibilityConfidence: lead.taiwanEligibilityConfidence,
    deadlineDate: lead.deadlineDate,
    deadlineLabel: lead.deadlineLabel,
    intakeYear: lead.intakeYear,
    studyStart: lead.studyStart,
    deadlineConfidence: lead.deadlineConfidence,
    canonicalUrl: lead.canonicalUrl,
    isDirectoryPage: lead.isDirectoryPage,
    officialSourceUrl: lead.officialSourceUrl,
    sourceDomain: lead.sourceDomain,
    confidence: lead.confidence,
    eligibilityConfidence: lead.eligibilityConfidence,
    tags: lead.tags,
    isIndexOnly: lead.isIndexOnly,
    firstSeenAt: lead.firstSeenAt,
    lastCheckedAt: lead.lastCheckedAt,
    nextCheckAt: lead.nextCheckAt,
    persistenceStatus: lead.persistenceStatus,
    sourceSeed: lead.sourceSeed,
    checkCount: lead.checkCount,
  }).returning().get()

  return mapLeadFromDb(result)
}

export async function updateLead(
  db: ReturnType<typeof getDb>,
  id: number,
  updates: Partial<Omit<Lead, 'id' | 'createdAt'>>
): Promise<Lead | null> {
  const result = await db.update(leads)
    .set({
      ...updates,
      updatedAt: new Date(),
    })
    .where(eq(leads.id, id))
    .returning()
    .get()

  return result ? mapLeadFromDb(result) : null
}

export async function deleteLead(db: ReturnType<typeof getDb>, id: number): Promise<boolean> {
  const result = await db.delete(leads).where(eq(leads.id, id)).returning().get()
  return !!result
}

function mapLeadFromDb(row: typeof leads.$inferSelect): Lead {
  return {
    id: row.id,
    name: row.name,
    amount: row.amount || undefined,
    deadline: row.deadline || undefined,
    source: row.source || undefined,
    sourceType: row.sourceType || undefined,
    status: row.status,
    eligibility: row.eligibility || undefined,
    notes: row.notes || undefined,
    addedDate: row.addedDate || undefined,
    url: row.url || undefined,
    matchScore: row.matchScore || undefined,
    matchReasons: row.matchReasons || undefined,
    hardFailReasons: row.hardFailReasons || undefined,
    softFlags: row.softFlags || undefined,
    bucket: row.bucket || undefined,
    httpStatus: row.httpStatus || undefined,
    effortScore: row.effortScore || undefined,
    trustTier: row.trustTier || undefined,
    riskFlags: row.riskFlags || undefined,
    matchedRuleIds: row.matchedRuleIds || undefined,
    eligibleCountries: row.eligibleCountries || undefined,
    isTaiwanEligible: row.isTaiwanEligible || undefined,
    taiwanEligibilityConfidence: row.taiwanEligibilityConfidence || undefined,
    deadlineDate: row.deadlineDate || undefined,
    deadlineLabel: row.deadlineLabel || undefined,
    intakeYear: row.intakeYear || undefined,
    studyStart: row.studyStart || undefined,
    deadlineConfidence: row.deadlineConfidence || undefined,
    canonicalUrl: row.canonicalUrl || undefined,
    isDirectoryPage: row.isDirectoryPage || undefined,
    officialSourceUrl: row.officialSourceUrl || undefined,
    sourceDomain: row.sourceDomain || undefined,
    confidence: row.confidence || undefined,
    eligibilityConfidence: row.eligibilityConfidence || undefined,
    tags: row.tags || undefined,
    isIndexOnly: row.isIndexOnly || undefined,
    firstSeenAt: row.firstSeenAt || undefined,
    lastCheckedAt: row.lastCheckedAt || undefined,
    nextCheckAt: row.nextCheckAt || undefined,
    persistenceStatus: row.persistenceStatus || undefined,
    sourceSeed: row.sourceSeed || undefined,
    checkCount: row.checkCount || undefined,
    createdAt: row.createdAt,
    updatedAt: row.updatedAt,
  }
}

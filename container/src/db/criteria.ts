import { eq } from 'drizzle-orm'
import { getDb } from './index'
import { criteria } from './schema'
import type { Criteria } from '../types'

export async function getCriteria(db: ReturnType<typeof getDb>): Promise<Criteria | null> {
  // Get the first (and should be only) criteria record
  const result = await db.select().from(criteria).limit(1).get()
  if (!result) {
    return null
  }
  
  // Check if the record is effectively empty (both JSON fields are null, undefined, or empty objects)
  const mapped = mapCriteriaFromDb(result)
  const isEmpty = (
    !mapped.criteriaJson || 
    mapped.criteriaJson === null ||
    (typeof mapped.criteriaJson === 'object' && Object.keys(mapped.criteriaJson).length === 0)
  ) && (
    !mapped.profileJson || 
    mapped.profileJson === null ||
    (typeof mapped.profileJson === 'object' && Object.keys(mapped.profileJson).length === 0)
  )
  
  return isEmpty ? null : mapped
}

export async function createOrUpdateCriteria(
  db: ReturnType<typeof getDb>,
  criteriaData: Omit<Criteria, 'id' | 'updatedAt'>
): Promise<Criteria> {
  const existing = await db.select().from(criteria).limit(1).get()

  if (existing) {
    // Update existing
    const result = await db.update(criteria)
      .set({
        criteriaJson: criteriaData.criteriaJson,
        profileJson: criteriaData.profileJson,
        updatedAt: new Date(),
      })
      .where(eq(criteria.id, existing.id))
      .returning()
      .get()

    if (!result) {
      throw new Error('Failed to update criteria')
    }

    return mapCriteriaFromDb(result)
  } else {
    // Create new
    const result = await db.insert(criteria).values({
      criteriaJson: criteriaData.criteriaJson,
      profileJson: criteriaData.profileJson,
    }).returning().get()

    if (!result) {
      throw new Error('Failed to create criteria')
    }

    return mapCriteriaFromDb(result)
  }
}

function mapCriteriaFromDb(row: typeof criteria.$inferSelect): Criteria {
  return {
    id: row.id,
    criteriaJson: row.criteriaJson || undefined,
    profileJson: row.profileJson || undefined,
    updatedAt: row.updatedAt,
  }
}

import { eq, desc } from 'drizzle-orm'
import { getDb } from './index'
import { applications } from './schema'
import type { Application } from '../types'

export async function getAllApplications(db: ReturnType<typeof getDb>): Promise<Application[]> {
  const results = await db.select().from(applications).orderBy(desc(applications.createdAt)).all()
  return results.map(mapApplicationFromDb)
}

export async function getApplicationById(db: ReturnType<typeof getDb>, id: number): Promise<Application | null> {
  const result = await db.select().from(applications).where(eq(applications.id, id)).limit(1).get()
  return result ? mapApplicationFromDb(result) : null
}

export async function createApplication(
  db: ReturnType<typeof getDb>,
  application: Omit<Application, 'id' | 'createdAt' | 'updatedAt'>
): Promise<Application> {
  const result = await db.insert(applications).values({
    name: application.name,
    deadline: application.deadline,
    status: application.status || 'not_started',
    currentStage: application.currentStage,
    nextAction: application.nextAction,
    requiredDocs: application.requiredDocs,
    progress: application.progress || 0,
    notes: application.notes,
  }).returning().get()

  if (!result) {
    throw new Error('Failed to create application')
  }

  return mapApplicationFromDb(result)
}

export async function updateApplication(
  db: ReturnType<typeof getDb>,
  id: number,
  updates: Partial<Omit<Application, 'id' | 'createdAt'>>
): Promise<Application | null> {
  const result = await db.update(applications)
    .set({
      ...updates,
      updatedAt: new Date(),
    })
    .where(eq(applications.id, id))
    .returning()
    .get()

  return result ? mapApplicationFromDb(result) : null
}

export async function deleteApplication(db: ReturnType<typeof getDb>, id: number): Promise<boolean> {
  const result = await db.delete(applications).where(eq(applications.id, id)).returning().get()
  return !!result
}

function mapApplicationFromDb(row: typeof applications.$inferSelect): Application {
  return {
    id: row.id,
    name: row.name,
    deadline: row.deadline || undefined,
    status: row.status,
    currentStage: row.currentStage || undefined,
    nextAction: row.nextAction || undefined,
    requiredDocs: row.requiredDocs || undefined,
    progress: row.progress || undefined,
    notes: row.notes || undefined,
    createdAt: row.createdAt,
    updatedAt: row.updatedAt,
  }
}

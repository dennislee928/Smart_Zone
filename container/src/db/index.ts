import { drizzle } from 'drizzle-orm/d1'
import * as schema from './schema'

export function getDb(database: D1Database) {
  return drizzle(database, { schema })
}

export type Database = ReturnType<typeof getDb>

/**
 * Data Migration Script
 * 
 * This script migrates data from JSON/YAML files to D1 database.
 * Run this once after setting up the database.
 * 
 * Usage:
 *   npx tsx src/scripts/migrate-data.ts
 */

import { getDb } from '../db/index'
import * as leadsDb from '../db/leads'
import * as applicationsDb from '../db/applications'
import * as criteriaDb from '../db/criteria'
import leadsData from '../../../tracking/leads.json'
import applicationsData from '../../../tracking/applications.json'
import { readFileSync } from 'fs'
import { parse } from 'yaml'

// This would be run in a Node.js environment, not in Workers
// For Workers, you'd need to create an API endpoint that triggers this
export async function migrateData(db: ReturnType<typeof getDb>) {
  console.log('Starting data migration...')

  // Migrate leads
  if (leadsData.leads && Array.isArray(leadsData.leads)) {
    console.log(`Migrating ${leadsData.leads.length} leads...`)
    for (const lead of leadsData.leads) {
      try {
        await leadsDb.createLead(db, lead as any)
      } catch (error) {
        console.error(`Failed to migrate lead ${lead.name}:`, error)
      }
    }
  }

  // Migrate applications
  if (applicationsData.applications && Array.isArray(applicationsData.applications)) {
    console.log(`Migrating ${applicationsData.applications.length} applications...`)
    for (const application of applicationsData.applications) {
      try {
        await applicationsDb.createApplication(db, application as any)
      } catch (error) {
        console.error(`Failed to migrate application ${application.name}:`, error)
      }
    }
  }

  // Migrate criteria
  try {
    const criteriaYaml = readFileSync('../../../tracking/criteria.yml', 'utf-8')
    const criteria = parse(criteriaYaml)
    await criteriaDb.createOrUpdateCriteria(db, {
      criteriaJson: criteria.criteria,
      profileJson: criteria.profile,
    })
    console.log('Migrated criteria')
  } catch (error) {
    console.error('Failed to migrate criteria:', error)
  }

  console.log('Data migration completed!')
}

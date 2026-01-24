import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core'

// Leads table - 儲存獎學金資訊
export const leads = sqliteTable('leads', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  amount: text('amount'),
  deadline: text('deadline'),
  source: text('source'),
  sourceType: text('source_type'),
  status: text('status').notNull().default('qualified'),
  eligibility: text('eligibility', { mode: 'json' }).$type<string[]>(),
  notes: text('notes'),
  addedDate: text('added_date'),
  url: text('url'),
  matchScore: integer('match_score').default(0),
  matchReasons: text('match_reasons', { mode: 'json' }).$type<string[]>(),
  hardFailReasons: text('hard_fail_reasons', { mode: 'json' }).$type<string[]>(),
  softFlags: text('soft_flags', { mode: 'json' }).$type<string[]>(),
  bucket: text('bucket'),
  httpStatus: integer('http_status'),
  effortScore: integer('effort_score'),
  trustTier: text('trust_tier'),
  riskFlags: text('risk_flags', { mode: 'json' }).$type<string[]>(),
  matchedRuleIds: text('matched_rule_ids', { mode: 'json' }).$type<string[]>(),
  eligibleCountries: text('eligible_countries', { mode: 'json' }).$type<string[]>(),
  isTaiwanEligible: integer('is_taiwan_eligible', { mode: 'boolean' }),
  taiwanEligibilityConfidence: text('taiwan_eligibility_confidence'),
  deadlineDate: text('deadline_date'),
  deadlineLabel: text('deadline_label'),
  intakeYear: text('intake_year'),
  studyStart: text('study_start'),
  deadlineConfidence: text('deadline_confidence'),
  canonicalUrl: text('canonical_url'),
  isDirectoryPage: integer('is_directory_page', { mode: 'boolean' }).default(false),
  officialSourceUrl: text('official_source_url'),
  sourceDomain: text('source_domain'),
  confidence: integer('confidence'), // Stored as integer (0-100) instead of float
  eligibilityConfidence: integer('eligibility_confidence'),
  tags: text('tags', { mode: 'json' }).$type<string[]>(),
  isIndexOnly: integer('is_index_only', { mode: 'boolean' }).default(false),
  firstSeenAt: text('first_seen_at'),
  lastCheckedAt: text('last_checked_at'),
  nextCheckAt: text('next_check_at'),
  persistenceStatus: text('persistence_status'),
  sourceSeed: text('source_seed'),
  checkCount: integer('check_count').default(0),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
})

// Applications table - 儲存申請追蹤
export const applications = sqliteTable('applications', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  deadline: text('deadline'),
  status: text('status').notNull().default('not_started'),
  currentStage: text('current_stage'),
  nextAction: text('next_action'),
  requiredDocs: text('required_docs', { mode: 'json' }).$type<string[]>(),
  progress: integer('progress').default(0),
  notes: text('notes'),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
})

// Criteria table - 儲存搜尋條件與個人資料
export const criteria = sqliteTable('criteria', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  criteriaJson: text('criteria_json', { mode: 'json' }).$type<{
    required: string[]
    preferred: string[]
    excluded_keywords: string[]
  }>(),
  profileJson: text('profile_json', { mode: 'json' }).$type<{
    nationality: string
    target_university: string
    target_country: string
    programme_level: string
    programme_start: string
    education: Array<{
      degree: string
      university: string
      department: string
      gpa: number
      gpa_scale: number
      status: string
    }>
    min_deadline?: string
    max_gpa_requirement?: number
  }>(),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
})

// Sources table - 儲存爬蟲來源設定
export const sources = sqliteTable('sources', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  type: text('type').notNull(),
  url: text('url').notNull(),
  enabled: integer('enabled', { mode: 'boolean' }).default(true),
  scraper: text('scraper'),
  priority: integer('priority'),
  createdAt: integer('created_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
})

// Source health table - 追蹤來源健康狀態
export const sourceHealth = sqliteTable('source_health', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  url: text('url').notNull().unique(),
  name: text('name').notNull(),
  sourceType: text('source_type').notNull(),
  consecutiveFailures: integer('consecutive_failures').default(0),
  totalAttempts: integer('total_attempts').default(0),
  totalSuccesses: integer('total_successes').default(0),
  lastStatus: text('last_status').notNull().default('Unknown'),
  lastHttpCode: integer('last_http_code'),
  lastError: text('last_error'),
  lastChecked: text('last_checked'),
  autoDisabled: integer('auto_disabled', { mode: 'boolean' }).default(false),
  disabledReason: text('disabled_reason'),
  updatedAt: integer('updated_at', { mode: 'timestamp' }).$defaultFn(() => new Date()),
})

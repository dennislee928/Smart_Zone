// Type definitions matching backend API

export interface Lead {
  id?: number
  name: string
  amount?: string
  deadline?: string
  source?: string
  sourceType?: string
  status: string
  eligibility?: string[]
  notes?: string
  addedDate?: string
  url?: string
  matchScore?: number
  matchReasons?: string[]
  hardFailReasons?: string[]
  softFlags?: string[]
  bucket?: string
  httpStatus?: number
  effortScore?: number
  trustTier?: string
  riskFlags?: string[]
  matchedRuleIds?: string[]
  eligibleCountries?: string[]
  isTaiwanEligible?: boolean
  taiwanEligibilityConfidence?: string
  deadlineDate?: string
  deadlineLabel?: string
  intakeYear?: string
  studyStart?: string
  deadlineConfidence?: string
  canonicalUrl?: string
  isDirectoryPage?: boolean
  officialSourceUrl?: string
  sourceDomain?: string
  confidence?: number
  eligibilityConfidence?: number
  tags?: string[]
  isIndexOnly?: boolean
  firstSeenAt?: string
  lastCheckedAt?: string
  nextCheckAt?: string
  persistenceStatus?: string
  sourceSeed?: string
  checkCount?: number
  createdAt?: string
  updatedAt?: string
}

export interface Application {
  id?: number
  name: string
  deadline?: string
  status: string
  currentStage?: string
  nextAction?: string
  requiredDocs?: string[]
  progress?: number
  notes?: string
  createdAt?: string
  updatedAt?: string
}

export interface Criteria {
  id?: number
  criteriaJson?: {
    required: string[]
    preferred: string[]
    excluded_keywords: string[]
  }
  profileJson?: {
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
  }
  updatedAt?: string
}

export interface Stats {
  totalLeads: number
  totalApplications: number
  inProgress: number
  completed: number
  notStarted: number
  upcoming7: number
  upcoming14: number
  upcoming21: number
}

export interface ApiResponse<T> {
  [key: string]: T | T[] | null
}

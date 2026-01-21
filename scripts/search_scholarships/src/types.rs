use serde::{Deserialize, Serialize};

// ============================================
// Core Lead Structure
// ============================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lead {
    pub name: String,
    pub amount: String,
    pub deadline: String,
    pub source: String,
    #[serde(rename = "source_type")]
    pub source_type: String,
    pub status: String,
    pub eligibility: Vec<String>,
    pub notes: String,
    #[serde(rename = "added_date")]
    pub added_date: String,
    pub url: String,
    #[serde(default)]
    pub match_score: i32,
    #[serde(default)]
    pub match_reasons: Vec<String>,
    
    // Triage fields
    #[serde(default)]
    pub bucket: Option<Bucket>,
    #[serde(default)]
    pub http_status: Option<u16>,
    #[serde(default)]
    pub effort_score: Option<i32>,
    #[serde(default)]
    pub trust_tier: Option<String>,
    #[serde(default)]
    pub risk_flags: Vec<String>,
    #[serde(default)]
    pub matched_rule_ids: Vec<String>,
    
    // Country eligibility fields
    #[serde(default)]
    pub eligible_countries: Vec<String>,    // Normalized country list
    #[serde(default)]
    pub is_taiwan_eligible: Option<bool>,   // Explicit eligibility flag
    
    // Structured date fields
    #[serde(default)]
    pub deadline_date: Option<String>,      // ISO format YYYY-MM-DD (application deadline)
    #[serde(default)]
    pub deadline_label: Option<String>,     // "applications close", "deadline", etc.
    #[serde(default)]
    pub intake_year: Option<String>,        // "2026/27"
    #[serde(default)]
    pub study_start: Option<String>,        // "2026-09"
    #[serde(default)]
    pub deadline_confidence: Option<String>, // "confirmed", "inferred", "unknown"
    
    // Deduplication fields
    #[serde(default)]
    pub canonical_url: Option<String>,      // Normalized URL for deduplication
    #[serde(default)]
    pub is_directory_page: bool,            // True if this is a landing/directory page
    
    // Source priority fields
    #[serde(default)]
    pub official_source_url: Option<String>, // Official source URL (if found via aggregator)
}

// ============================================
// Bucket / Triage Enums
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Bucket {
    A,  // 主攻 (High priority, apply now)
    B,  // 備援 (Needs verification / medium priority)
    C,  // 淘汰 (Hard fail - ineligible)
    X,  // 已截止 (Missed - keep for next cycle)
}

impl std::fmt::Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bucket::A => write!(f, "A"),
            Bucket::B => write!(f, "B"),
            Bucket::C => write!(f, "C"),
            Bucket::X => write!(f, "X"),
        }
    }
}

impl Default for Bucket {
    fn default() -> Self {
        Bucket::C
    }
}

// ============================================
// Trust Tier Enum
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TrustTier {
    S,  // Official university/government - highest trust
    A,  // Major foundation (Gates, Rhodes, Chevening)
    B,  // Verified aggregator (British Council, FindAPhD)
    C,  // Unverified source
}

impl std::fmt::Display for TrustTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustTier::S => write!(f, "S"),
            TrustTier::A => write!(f, "A"),
            TrustTier::B => write!(f, "B"),
            TrustTier::C => write!(f, "C"),
        }
    }
}

impl Default for TrustTier {
    fn default() -> Self {
        TrustTier::C
    }
}

impl TrustTier {
    /// Parse trust tier from string
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "S" => TrustTier::S,
            "A" => TrustTier::A,
            "B" => TrustTier::B,
            _ => TrustTier::C,
        }
    }
    
    /// Get trust tier from source type
    pub fn from_source_type(source_type: &str) -> Self {
        match source_type.to_lowercase().as_str() {
            "university" => TrustTier::S,
            "government" => TrustTier::S,
            "foundation" => TrustTier::A,
            "third_party" => TrustTier::B,
            _ => TrustTier::C,
        }
    }
}

// ============================================
// Link Health Types
// ============================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkHealthResult {
    pub url: String,
    pub status: LinkHealthStatus,
    pub http_code: Option<u16>,
    pub final_url: Option<String>,  // After redirects
    pub checked_at: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LinkHealthStatus {
    Ok,
    Redirect,
    NotFound,
    Forbidden,
    RateLimited,
    ServerError,
    Timeout,
    Unknown,
}

impl std::fmt::Display for LinkHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkHealthStatus::Ok => write!(f, "OK"),
            LinkHealthStatus::Redirect => write!(f, "Redirect"),
            LinkHealthStatus::NotFound => write!(f, "404 Not Found"),
            LinkHealthStatus::Forbidden => write!(f, "403 Forbidden"),
            LinkHealthStatus::RateLimited => write!(f, "429 Rate Limited"),
            LinkHealthStatus::ServerError => write!(f, "5xx Server Error"),
            LinkHealthStatus::Timeout => write!(f, "Timeout"),
            LinkHealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

// ============================================
// Rules Engine Types
// ============================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesConfig {
    #[serde(default)]
    pub hard_reject_rules: Vec<Rule>,
    #[serde(default)]
    pub soft_downgrade_rules: Vec<Rule>,
    #[serde(default)]
    pub positive_scoring_rules: Vec<Rule>,
    #[serde(default)]
    pub scoring_weights: Option<ScoringWeights>,
    #[serde(default)]
    pub bucket_thresholds: Option<BucketThresholds>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub description: Option<String>,
    pub when: RuleCondition,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleCondition {
    #[serde(default)]
    pub any_regex: Option<Vec<String>>,
    #[serde(default)]
    pub not_any_regex: Option<Vec<String>>,  // Negative match - trigger if NONE match
    #[serde(default)]
    pub deadline: Option<DeadlineCondition>,
    #[serde(default)]
    pub http_status: Option<HttpStatusCondition>,
    #[serde(default)]
    pub effort_score: Option<EffortScoreCondition>,
    #[serde(default)]
    pub is_taiwan_eligible: Option<bool>,  // Country eligibility gate
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DeadlineCondition {
    #[serde(default)]
    pub lt_today: Option<bool>,
    #[serde(default)]
    pub is_null: Option<bool>,
    #[serde(default)]
    pub gt_study_start: Option<bool>,      // Deadline is after target study start date
    #[serde(default)]
    pub safety_margin_days: Option<i64>,   // Days before study start to reject (default: 60)
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HttpStatusCondition {
    #[serde(default)]
    pub any_of: Option<Vec<u16>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EffortScoreCondition {
    #[serde(default)]
    pub gt: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleAction {
    #[serde(default)]
    pub bucket: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub score_add: Option<i32>,
    #[serde(default)]
    pub effort_reduce: Option<i32>,
    #[serde(default)]
    pub add_to_watchlist: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringWeights {
    pub award_value: f64,
    pub probability: f64,
    pub timeline: f64,
    pub effort_penalty: f64,
    pub risk_penalty: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BucketThresholds {
    #[serde(rename = "A")]
    pub a: Option<BucketThreshold>,
    #[serde(rename = "B")]
    pub b: Option<BucketThreshold>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BucketThreshold {
    pub min_final_score: i32,
    #[serde(default)]
    pub min_trust_tier: Option<String>,
    #[serde(default)]
    pub max_effort_score: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleMatch {
    pub rule_id: String,
    pub rule_name: String,
    pub stage: String,
    pub action: String,
    pub reason: String,
}

// ============================================
// Triage Result Types
// ============================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TriageResult {
    pub timestamp: String,
    pub total_processed: usize,
    pub bucket_a: Vec<Lead>,
    pub bucket_b: Vec<Lead>,
    pub bucket_c: Vec<Lead>,
    pub watchlist: Vec<Lead>,
    pub dead_links: Vec<LinkHealthResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesAudit {
    pub version: String,
    pub timestamp: String,
    pub rules_file: String,
    pub total_rules: usize,
    pub items_processed: usize,
    pub buckets: BucketCounts,
    pub rule_hits: Vec<RuleHitCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BucketCounts {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    #[serde(default)]
    pub x: usize,  // Missed/Closed - saved for next cycle
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleHitCount {
    pub rule_id: String,
    pub rule_name: String,
    pub hit_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeadsFile {
    pub leads: Vec<Lead>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Criteria {
    pub criteria: CriteriaContent,
    #[serde(default)]
    pub profile: Option<Profile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CriteriaContent {
    pub required: Vec<String>,
    pub preferred: Vec<String>,
    pub excluded_keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub nationality: String,
    pub target_university: String,
    pub target_country: String,
    pub programme_level: String,
    pub programme_start: String,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub min_deadline: Option<String>,
    #[serde(default)]
    pub max_gpa_requirement: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Education {
    pub degree: String,
    pub university: String,
    pub department: String,
    pub gpa: f64,
    pub gpa_scale: f64,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sources {
    pub sources: Vec<Source>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Source {
    pub name: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    pub enabled: bool,
    pub scraper: String,
}

// ============================================
// Source Health Tracking Types
// ============================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceHealthFile {
    pub last_updated: String,
    pub sources: Vec<SourceHealth>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceHealth {
    pub url: String,
    pub name: String,
    pub source_type: String,
    pub consecutive_failures: u32,
    pub total_attempts: u32,
    pub total_successes: u32,
    pub last_status: SourceStatus,
    pub last_http_code: Option<u16>,
    pub last_error: Option<String>,
    pub last_checked: String,
    pub auto_disabled: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum SourceStatus {
    Ok,
    NotFound,
    Forbidden,
    RateLimited,
    ServerError,
    SslError,
    Timeout,
    TooManyRedirects,
    NetworkError,
    Unknown,
}

impl std::fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceStatus::Ok => write!(f, "OK"),
            SourceStatus::NotFound => write!(f, "404 Not Found"),
            SourceStatus::Forbidden => write!(f, "403 Forbidden"),
            SourceStatus::RateLimited => write!(f, "429 Rate Limited"),
            SourceStatus::ServerError => write!(f, "5xx Server Error"),
            SourceStatus::SslError => write!(f, "SSL Error"),
            SourceStatus::Timeout => write!(f, "Timeout"),
            SourceStatus::TooManyRedirects => write!(f, "Too Many Redirects"),
            SourceStatus::NetworkError => write!(f, "Network Error"),
            SourceStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Default for SourceHealth {
    fn default() -> Self {
        Self {
            url: String::new(),
            name: String::new(),
            source_type: String::new(),
            consecutive_failures: 0,
            total_attempts: 0,
            total_successes: 0,
            last_status: SourceStatus::Unknown,
            last_http_code: None,
            last_error: None,
            last_checked: String::new(),
            auto_disabled: false,
            disabled_reason: None,
        }
    }
}

// ============================================
// Source Filter Config
// ============================================

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SourceFilterConfig {
    /// 只爬取這些類型的來源 (空 = 全部)
    #[serde(default)]
    pub include_types: Vec<String>,
    /// 排除這些類型的來源
    #[serde(default)]
    pub exclude_types: Vec<String>,
    /// 連續失敗幾次後自動停用
    #[serde(default = "default_max_failures")]
    pub max_consecutive_failures: u32,
    /// 是否跳過已自動停用的來源
    #[serde(default = "default_true")]
    pub skip_auto_disabled: bool,
}

fn default_max_failures() -> u32 { 3 }
fn default_true() -> bool { true }

// ============================================
// Scrape Result for Health Tracking
// ============================================

#[derive(Debug, Clone)]
pub struct ScrapeResult {
    pub leads: Vec<Lead>,
    pub status: SourceStatus,
    pub http_code: Option<u16>,
    pub error_message: Option<String>,
}

use serde::{Deserialize, Serialize};

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

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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeadsFile {
    pub leads: Vec<Lead>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Criteria {
    pub criteria: CriteriaContent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CriteriaContent {
    pub required: Vec<String>,
    pub preferred: Vec<String>,
    pub excluded_keywords: Vec<String>,
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

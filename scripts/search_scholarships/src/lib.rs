//! ScholarshipOps Library
//! 
//! Core library for scholarship discovery and qualification

pub mod scrapers;
pub mod filter;
pub mod storage;
pub mod notify;
pub mod types;
pub mod sorter;
pub mod rules;
pub mod link_health;
pub mod triage;
pub mod effort;
pub mod source_health;
pub mod normalize;
pub mod discovery;
pub mod url_state;
pub mod extraction_fallbacks;
pub mod js_detector;
pub mod browser_queue;
pub mod api_discovery;

pub use types::*;

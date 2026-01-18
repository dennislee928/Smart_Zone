use crate::types::{Lead, Criteria};

pub fn matches_criteria(lead: &Lead, criteria: &Criteria) -> bool {
    let text = format!("{} {} {}", lead.name, lead.notes, lead.eligibility.join(" ")).to_lowercase();
    
    // Check excluded keywords
    for keyword in &criteria.criteria.excluded_keywords {
        if text.contains(&keyword.to_lowercase()) {
            return false;
        }
    }
    
    // Check required criteria (at least one must match)
    if !criteria.criteria.required.is_empty() {
        let matches_required = criteria.criteria.required.iter()
            .any(|req| text.contains(&req.to_lowercase()));
        if !matches_required {
            return false;
        }
    }
    
    true
}

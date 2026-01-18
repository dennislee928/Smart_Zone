mod scrapers;
mod filter;
mod storage;
mod notify;
mod types;

pub use types::*;

fn main() -> Result<()> {
    let root = std::env::var("ROOT").unwrap_or_else(|_| ".".to_string());
    
    // Load criteria
    let criteria = storage::load_criteria(&root)?;
    
    // Load sources
    let sources = storage::load_sources(&root)?;
    let enabled_sources: Vec<_> = sources.sources.iter().filter(|s| s.enabled).collect();
    
    // Load existing leads
    let mut leads_file = storage::load_leads(&root)?;
    let existing_names: std::collections::HashSet<String> = leads_file.leads.iter()
        .map(|l| l.name.clone())
        .collect();
    
    // Scrape scholarships
    let mut new_leads = Vec::new();
    for source in &enabled_sources {
        println!("Scraping: {} ({})", source.name, source.url);
        let scraped = scrapers::scrape_source(source)?;
        
        for mut lead in scraped {
            // Skip if already exists
            if existing_names.contains(&lead.name) {
                continue;
            }
            
            // Filter by criteria
            if filter::matches_criteria(&lead, &criteria) {
                lead.status = "qualified".to_string();
                lead.source_type = source.source_type.clone();
                lead.added_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                new_leads.push(lead);
            } else {
                println!("Filtered out: {}", lead.name);
            }
        }
    }
    
    // Add new leads
    if !new_leads.is_empty() {
        leads_file.leads.append(&mut new_leads);
        storage::save_leads(&root, &leads_file)?;
        
        // Format and send notification
        let msg = format!(
            "[ScholarshipOps Search] Found {} new qualified scholarships:\n{}",
            new_leads.len(),
            new_leads.iter()
                .take(10)
                .map(|l| format!("- {} ({}) - {}", l.name, l.deadline, l.amount))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        notify::send_notifications(&msg)?;
    } else {
        println!("No new scholarships found.");
    }
    
    Ok(())
}

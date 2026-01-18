mod university;
mod government;
mod third_party;

use crate::types::{Source, Lead};
use anyhow::Result;

pub fn scrape_source(source: &Source) -> Result<Vec<Lead>> {
    match source.scraper.as_str() {
        "university" => university::scrape(source.url.as_str()),
        "government" => government::scrape(source.url.as_str()),
        "third_party" => third_party::scrape(source.url.as_str()),
        _ => {
            eprintln!("Unknown scraper type: {}", source.scraper);
            Ok(vec![])
        }
    }
}

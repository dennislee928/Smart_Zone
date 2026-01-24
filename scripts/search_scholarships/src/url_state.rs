//! URL State Storage Module
//!
//! Provides SQLite-based storage for URL state tracking:
//! - etag, last_modified, content_hash, last_seen, status
//! - Conditional GET support (If-None-Match / If-Modified-Since)
//! - Incremental fetch optimization

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::sync::Mutex;

/// URL state information
#[derive(Debug, Clone)]
pub struct UrlState {
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_hash: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub status: UrlStatus,
    pub http_code: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UrlStatus {
    Ok,
    NotFound,
    Forbidden,
    RateLimited,
    ServerError,
    Timeout,
    ParseError,
    RobotsDisallow,
    Unknown,
}

impl UrlStatus {
    pub fn from_http_code(code: u16) -> Self {
        match code {
            200..=299 => UrlStatus::Ok,
            404 => UrlStatus::NotFound,
            403 => UrlStatus::Forbidden,
            429 => UrlStatus::RateLimited,
            500..=599 => UrlStatus::ServerError,
            _ => UrlStatus::Unknown,
        }
    }
}

/// URL State Storage Manager
pub struct UrlStateStorage {
    conn: Mutex<Connection>,
}

impl UrlStateStorage {
    /// Initialize or open URL state database
    pub fn new(root: &str) -> Result<Self> {
        let db_path = PathBuf::from(root).join("tracking").join("url_state.db");
        
        // Create tracking directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create tracking directory")?;
        }
        
        let conn = Connection::open(&db_path)
            .context("Failed to open URL state database")?;
        
        let storage = Self {
            conn: Mutex::new(conn),
        };
        
        storage.init_schema()?;
        Ok(storage)
    }
    
    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS url_states (
                url TEXT PRIMARY KEY,
                etag TEXT,
                last_modified TEXT,
                content_hash TEXT,
                last_seen TEXT,
                status TEXT,
                http_code INTEGER
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_seen ON url_states(last_seen)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_status ON url_states(status)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Get URL state
    pub fn get(&self, url: &str) -> Result<Option<UrlState>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT etag, last_modified, content_hash, last_seen, status, http_code
             FROM url_states WHERE url = ?1"
        )?;
        
        let result = stmt.query_row(params![url], |row| {
            Ok(UrlState {
                url: url.to_string(),
                etag: row.get(0)?,
                last_modified: row.get(1)?,
                content_hash: row.get(2)?,
                last_seen: row.get::<_, Option<String>>(3)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                status: parse_status(row.get(4)?),
                http_code: row.get(5)?,
            })
        });
        
        match result {
            Ok(state) => Ok(Some(state)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Update URL state
    pub fn update(&self, state: &UrlState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO url_states 
             (url, etag, last_modified, content_hash, last_seen, status, http_code)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                state.url,
                state.etag,
                state.last_modified,
                state.content_hash,
                state.last_seen.map(|dt| dt.to_rfc3339()),
                status_to_string(&state.status),
                state.http_code,
            ],
        )?;
        
        Ok(())
    }
    
    /// Calculate content hash from response body
    pub fn calculate_content_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
    
    /// Clean up old entries (older than specified days)
    pub fn cleanup_old(&self, days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();
        
        let count = conn.execute(
            "DELETE FROM url_states WHERE last_seen < ?1",
            params![cutoff_str],
        )?;
        
        Ok(count)
    }
}

fn parse_status(s: String) -> UrlStatus {
    match s.as_str() {
        "ok" => UrlStatus::Ok,
        "not_found" => UrlStatus::NotFound,
        "forbidden" => UrlStatus::Forbidden,
        "rate_limited" => UrlStatus::RateLimited,
        "server_error" => UrlStatus::ServerError,
        "timeout" => UrlStatus::Timeout,
        "parse_error" => UrlStatus::ParseError,
        "robots_disallow" => UrlStatus::RobotsDisallow,
        _ => UrlStatus::Unknown,
    }
}

fn status_to_string(status: &UrlStatus) -> &str {
    match status {
        UrlStatus::Ok => "ok",
        UrlStatus::NotFound => "not_found",
        UrlStatus::Forbidden => "forbidden",
        UrlStatus::RateLimited => "rate_limited",
        UrlStatus::ServerError => "server_error",
        UrlStatus::Timeout => "timeout",
        UrlStatus::ParseError => "parse_error",
        UrlStatus::RobotsDisallow => "robots_disallow",
        UrlStatus::Unknown => "unknown",
    }
}

/// Build conditional GET headers from URL state
pub fn build_conditional_headers(state: &UrlState) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    
    if let Some(ref etag) = state.etag {
        headers.push(("If-None-Match".to_string(), etag.clone()));
    }
    
    if let Some(ref last_modified) = state.last_modified {
        headers.push(("If-Modified-Since".to_string(), last_modified.clone()));
    }
    
    headers
}

/// Check if response indicates content hasn't changed (304 Not Modified)
pub fn is_not_modified(status_code: u16) -> bool {
    status_code == 304
}

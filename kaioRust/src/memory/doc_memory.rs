use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRecord {
    pub id: Option<i64>,
    pub file_id: i64,
    pub doc_path: String,
    pub doc_level: u8,
    pub source_hash: Option<String>,
    pub status: String, // "fresh", "stale", "maybe_stale"
    pub updated_at: String,
    pub stale_reason: Option<String>,
}

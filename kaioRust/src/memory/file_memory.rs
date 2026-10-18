use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Option<i64>,
    pub path: String,
    pub language: Option<String>,
    pub size: usize,
    pub source_hash: Option<String>,
    pub parse_status: Option<String>,
    pub last_commit: Option<String>,
    pub last_modified: Option<String>,
    pub summary: Option<String>,
    pub risk_level: Option<String>,
    pub doc_path: Option<String>,
    pub doc_status: Option<String>,
}

impl FileRecord {
    pub fn new(path: impl Into<String>, size: usize) -> Self {
        Self {
            id: None,
            path: path.into(),
            language: None,
            size,
            source_hash: None,
            parse_status: Some("ok".to_string()),
            last_commit: None,
            last_modified: None,
            summary: None,
            risk_level: None,
            doc_path: None,
            doc_status: Some("unindexed".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawImportRecord {
    pub id: Option<i64>,
    pub file_id: i64,
    pub raw_import: String,
    pub resolved_file_id: Option<i64>,
    pub resolved_symbol_id: Option<i64>,
    pub status: String,
}

impl RawImportRecord {
    pub fn new(file_id: i64, raw_import: impl Into<String>) -> Self {
        Self {
            id: None,
            file_id,
            raw_import: raw_import.into(),
            resolved_file_id: None,
            resolved_symbol_id: None,
            status: "unresolved".to_string(),
        }
    }
}

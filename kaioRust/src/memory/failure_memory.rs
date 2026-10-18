use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub id: Option<i64>,
    pub error_type: String, // compiler, test, runtime, linter
    pub message: String,
    pub file_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub fix_summary: Option<String>,
    pub status: Option<String>, // unresolved, resolved
    pub resolved_at: Option<String>,
    pub created_at: String,
}

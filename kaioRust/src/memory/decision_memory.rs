use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    pub status: String, // accepted, proposed, deprecated
    pub created_at: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub id: Option<i64>,
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    pub line_start: usize,
    pub line_end: usize,
    pub visibility: Option<String>,
    pub signature: Option<String>,
    pub summary: Option<String>,
    pub parent: Option<String>,
}

impl SymbolRecord {
    pub fn new(
        file_id: i64,
        name: impl Into<String>,
        kind: impl Into<String>,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        Self {
            id: None,
            file_id,
            name: name.into(),
            kind: kind.into(),
            line_start,
            line_end,
            visibility: None,
            signature: None,
            summary: None,
            parent: None,
        }
    }
}

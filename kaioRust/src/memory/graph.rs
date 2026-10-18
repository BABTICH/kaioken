use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub id: Option<i64>,
    pub source_symbol_id: Option<i64>,
    pub target_symbol_id: Option<i64>,
    pub edge_type: String, // calls, imports, implements, covers_test
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub edges: Vec<EdgeRecord>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }
}

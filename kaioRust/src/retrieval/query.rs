use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalContext {
    pub matched_files: Vec<PathBuf>,
    pub matched_symbols: Vec<String>,
    pub related_docs: Vec<PathBuf>,
    pub failure_hints: Vec<String>,
}

impl RetrievalContext {
    pub fn empty() -> Self {
        Self {
            matched_files: Vec::new(),
            matched_symbols: Vec::new(),
            related_docs: Vec::new(),
            failure_hints: Vec::new(),
        }
    }
}

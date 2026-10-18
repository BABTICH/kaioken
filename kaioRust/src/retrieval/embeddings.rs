use crate::error::Result;

pub struct SemanticSearchEngine;

impl SemanticSearchEngine {
    pub fn search(_query: &str, _limit: usize) -> Result<Vec<String>> {
        // Stub for future vector search
        Ok(Vec::new())
    }
}

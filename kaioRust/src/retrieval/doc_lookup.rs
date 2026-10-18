use std::path::Path;
use crate::error::Result;
use crate::memory::MemoryEngine;
use crate::retrieval::context_packet::DocStalenessWarning;

pub struct DocLookup<'a> {
    memory: &'a MemoryEngine,
    repo_root: &'a Path,
}

impl<'a> DocLookup<'a> {
    pub fn new(memory: &'a MemoryEngine, repo_root: &'a Path) -> Self {
        Self { memory, repo_root }
    }

    /// Evaluates documentation status for a set of file IDs and returns any staleness warnings.
    pub fn check_doc_staleness(&self, file_ids: &[i64]) -> Result<Vec<DocStalenessWarning>> {
        let mut warnings = Vec::new();

        for &fid in file_ids {
            let file_rec = match self.memory.get_file_by_id(fid)? {
                Some(f) => f,
                None => continue,
            };

            if let Some(doc_rec) = self.memory.get_doc_by_file_id(fid)? {
                let is_stale_status = doc_rec.status == "stale" || doc_rec.status == "maybe-stale";
                let hash_mismatch = file_rec.source_hash.as_ref() != doc_rec.source_hash.as_ref();

                if is_stale_status || hash_mismatch {
                    let reason = if let Some(r) = doc_rec.stale_reason {
                        r
                    } else if hash_mismatch {
                        "File content hash does not match documentation provenance hash".to_string()
                    } else {
                        "Documentation marked as stale".to_string()
                    };

                    warnings.push(DocStalenessWarning {
                        doc_path: doc_rec.doc_path,
                        file_path: file_rec.path,
                        reason,
                        status: doc_rec.status,
                    });
                }
            } else {
                // Check if doc file exists on disk even if not in table
                let expected_doc_rel = format!(".agent-docs/{}.md", file_rec.path);
                let expected_doc_full = self.repo_root.join(&expected_doc_rel);
                if !expected_doc_full.exists() {
                    // Not documented yet or missing
                }
            }
        }

        Ok(warnings)
    }
}

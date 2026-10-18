use std::collections::HashSet;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::memory::MemoryEngine;

/// Detailed report resulting from a staleness cascade pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessCascadeReport {
    pub changed_files: Vec<String>,
    pub directly_stale: Vec<String>,
    pub maybe_stale_dependents: Vec<String>,
    pub cascade_depth: usize,
}

pub struct StalenessCascadeEngine;

impl StalenessCascadeEngine {
    /// Cascades changes from a set of modified files:
    /// 1. Directly modified files are marked `stale` immediately.
    /// 2. Direct 1-hop dependents are marked `maybe_stale`.
    /// 3. Transitive dependents (>1 hop) are never automatically updated, strictly bounding cost.
    pub fn cascade_changes(
        repo_root: &Path,
        changed_paths: &[String],
        memory: &MemoryEngine,
    ) -> Result<StalenessCascadeReport> {
        let mut directly_stale = Vec::new();
        let mut maybe_stale_dependents = Vec::new();
        let mut direct_stale_ids = HashSet::new();

        // 1. Mark directly changed files as 'stale'
        for path_str in changed_paths {
            let normalized = path_str.replace('\\', "/");
            let file_rec = if let Some(rec) = memory.get_file_by_path(&normalized)? {
                Some(rec)
            } else {
                let full = repo_root.join(&normalized);
                if let Ok(rel) = full.strip_prefix(repo_root) {
                    let rel_norm = rel.to_string_lossy().replace('\\', "/");
                    memory.get_file_by_path(&rel_norm)?
                } else {
                    None
                }
            };

            if let Some(file) = file_rec {
                if let Some(fid) = file.id {
                    direct_stale_ids.insert(fid);
                    memory.mark_doc_status(fid, "stale", Some("Direct code modification"))?;
                    directly_stale.push(file.path);
                }
            }
        }

        // 2. Mark 1-hop direct dependents as 'maybe_stale'
        let mut maybe_stale_ids = HashSet::new();
        for &fid in &direct_stale_ids {
            let dependents = memory.get_dependent_file_ids(fid)?;
            for dep_id in dependents {
                // Only mark maybe_stale if it's NOT already directly stale
                if !direct_stale_ids.contains(&dep_id) && !maybe_stale_ids.contains(&dep_id) {
                    let changed_source = memory
                        .get_file_by_id(fid)?
                        .map(|f| f.path)
                        .unwrap_or_else(|| format!("file_id {}", fid));

                    let reason = format!("Direct dependency '{}' changed", changed_source);
                    memory.mark_doc_status(dep_id, "maybe_stale", Some(&reason))?;
                    maybe_stale_ids.insert(dep_id);

                    if let Ok(Some(dep_file)) = memory.get_file_by_id(dep_id) {
                        maybe_stale_dependents.push(dep_file.path);
                    }
                }
            }
        }

        directly_stale.sort();
        maybe_stale_dependents.sort();

        Ok(StalenessCascadeReport {
            changed_files: changed_paths.to_vec(),
            directly_stale,
            maybe_stale_dependents,
            cascade_depth: 1,
        })
    }
}

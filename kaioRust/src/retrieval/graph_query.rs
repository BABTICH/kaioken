use std::collections::HashSet;
use rusqlite::params;
use crate::error::{KaiokenError, Result};
use crate::memory::MemoryEngine;
use crate::retrieval::context_packet::GraphSubset;

pub struct GraphQueryEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> GraphQueryEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    /// Queries the dependency graph around a set of symbol IDs and file IDs up to specified depth.
    pub fn query_subgraph(&self, symbol_ids: &[i64], file_ids: &[i64], max_depth: usize) -> Result<GraphSubset> {
        let conn = self.memory.db().connection();
        let depth = max_depth.min(2).max(1);

        let mut direct_callers: Vec<String> = Vec::new();
        let mut direct_callees: Vec<String> = Vec::new();
        let mut direct_imports: Vec<String> = Vec::new();
        let mut affected_tests: Vec<String> = Vec::new();

        let mut visited_symbols: HashSet<i64> = symbol_ids.iter().copied().collect();
        let mut current_level_symbols = symbol_ids.to_vec();

        for _ in 0..depth {
            let mut next_level_symbols = Vec::new();

            for &sym_id in &current_level_symbols {
                if direct_callers.len() >= 30 && direct_callees.len() >= 30 {
                    break;
                }

                // 1. Callers: edges where target_symbol_id == sym_id
                let mut caller_stmt = conn.prepare(
                    "SELECT s.id, s.name, s.kind, f.path
                     FROM edges e
                     JOIN symbols s ON e.source_symbol_id = s.id
                     JOIN files f ON s.file_id = f.id
                     WHERE e.target_symbol_id = ?1"
                ).map_err(|e| KaiokenError::RetrievalError(format!("Failed to prepare caller query: {}", e)))?;

                let caller_rows = caller_stmt.query_map(params![sym_id], |row| {
                    let id: i64 = row.get(0)?;
                    let name: String = row.get(1)?;
                    let kind: String = row.get(2)?;
                    let path: String = row.get(3)?;
                    Ok((id, name, kind, path))
                }).map_err(|e| KaiokenError::RetrievalError(format!("Failed to query callers: {}", e)))?;

                for r in caller_rows {
                    if let Ok((src_id, name, kind, path)) = r {
                        if !self.is_excluded_path(&path) {
                            let label = format!("{}::{} ({})", path, name, kind);
                            if !direct_callers.contains(&label) {
                                direct_callers.push(label);
                            }
                            if self.is_test_path(&path) || name.starts_with("test_") {
                                if !affected_tests.contains(&path) {
                                    affected_tests.push(path.clone());
                                }
                            }
                            if visited_symbols.insert(src_id) {
                                next_level_symbols.push(src_id);
                            }
                        }
                    }
                }

                // 2. Callees: edges where source_symbol_id == sym_id
                let mut callee_stmt = conn.prepare(
                    "SELECT s.id, s.name, s.kind, f.path
                     FROM edges e
                     JOIN symbols s ON e.target_symbol_id = s.id
                     JOIN files f ON s.file_id = f.id
                     WHERE e.source_symbol_id = ?1"
                ).map_err(|e| KaiokenError::RetrievalError(format!("Failed to prepare callee query: {}", e)))?;

                let callee_rows = callee_stmt.query_map(params![sym_id], |row| {
                    let id: i64 = row.get(0)?;
                    let name: String = row.get(1)?;
                    let kind: String = row.get(2)?;
                    let path: String = row.get(3)?;
                    Ok((id, name, kind, path))
                }).map_err(|e| KaiokenError::RetrievalError(format!("Failed to query callees: {}", e)))?;

                for r in callee_rows {
                    if let Ok((target_id, name, kind, path)) = r {
                        if !self.is_excluded_path(&path) {
                            let label = format!("{}::{} ({})", path, name, kind);
                            if !direct_callees.contains(&label) {
                                direct_callees.push(label);
                            }
                            if visited_symbols.insert(target_id) {
                                next_level_symbols.push(target_id);
                            }
                        }
                    }
                }
            }

            current_level_symbols = next_level_symbols;
        }

        // 3. Raw Imports for touched files
        for &fid in file_ids {
            let imports = self.memory.get_raw_imports_for_file(fid)?;
            for imp in imports {
                if !direct_imports.contains(&imp.raw_import) {
                    direct_imports.push(imp.raw_import);
                }
            }

            // Check dependent files for tests
            let dependent_file_ids = self.memory.get_dependent_file_ids(fid)?;
            for dep_id in dependent_file_ids {
                if let Ok(Some(f)) = self.memory.get_file_by_id(dep_id) {
                    if self.is_test_path(&f.path) && !affected_tests.contains(&f.path) {
                        affected_tests.push(f.path);
                    }
                }
            }
        }

        // Limit results to quotas
        direct_callers.truncate(30);
        direct_callees.truncate(30);
        direct_imports.truncate(20);
        affected_tests.truncate(20);

        Ok(GraphSubset {
            direct_callers,
            direct_callees,
            direct_imports,
            affected_tests,
        })
    }

    fn is_test_path(&self, path: &str) -> bool {
        let p_lower = path.to_lowercase();
        p_lower.contains("test") || p_lower.starts_with("tests/") || p_lower.ends_with("_test.rs")
    }

    fn is_excluded_path(&self, path: &str) -> bool {
        let p_lower = path.to_lowercase();
        p_lower.contains("node_modules/")
            || p_lower.contains("target/")
            || p_lower.contains("vendor/")
            || p_lower.contains(".agent/")
            || p_lower.ends_with(".gen.rs")
            || p_lower.ends_with(".pb.rs")
    }
}

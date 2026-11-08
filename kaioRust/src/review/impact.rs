use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use crate::error::Result;
use crate::memory::MemoryEngine;
use crate::review::diff::{AffectedSymbolInfo, ReviewFileSummary};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AffectedCaller {
    pub caller_file: String,
    pub caller_symbol: Option<String>,
    pub caller_line: Option<usize>,
    pub target_symbol: String,
    pub target_file: String,
    pub edge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestGap {
    pub file: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaleDocInfo {
    pub file_path: String,
    pub doc_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixSuggestion {
    pub target: String,
    pub description: String,
    pub action: String,
}

pub struct ImpactAnalyzer;

impl ImpactAnalyzer {
    /// Identify callers and dependents of all modified symbols using the dependency graph and import memory
    pub fn find_affected_callers(
        affected_symbols: &[AffectedSymbolInfo],
        memory: &MemoryEngine,
    ) -> Result<Vec<AffectedCaller>> {
        let mut callers = Vec::new();
        let mut seen = HashSet::new();

        for aff in affected_symbols {
            let sym_id = match aff.symbol.id {
                Some(id) => id,
                None => continue,
            };

            // 1. Query incoming edges where target_symbol_id == sym_id
            let incoming_edges = memory.get_edges_for_target(sym_id)?;
            for edge in incoming_edges {
                if let Some(src_sym_id) = edge.source_symbol_id {
                    let conn = memory.db().connection();
                    if let Ok((src_name, src_line, src_path)) = conn.query_row(
                        "SELECT s.name, s.line_start, f.path
                         FROM symbols s
                         JOIN files f ON s.file_id = f.id
                         WHERE s.id = ?1;",
                        rusqlite::params![src_sym_id],
                        |row| {
                            let name: String = row.get(0)?;
                            let line: usize = row.get::<_, i64>(1)? as usize;
                            let path: String = row.get(2)?;
                            Ok((name, line, path))
                        },
                    ) {
                        let key = format!("{}:{}:{}", src_path, src_name, aff.symbol.name);
                        if seen.insert(key) {
                            callers.push(AffectedCaller {
                                caller_file: src_path,
                                caller_symbol: Some(src_name),
                                caller_line: Some(src_line),
                                target_symbol: aff.symbol.name.clone(),
                                target_file: aff.file_path.clone(),
                                edge_type: edge.edge_type,
                            });
                        }
                    }
                }
            }

            // 2. Query raw imports that import this module or symbol
            if let Ok(Some(file_rec)) = memory.get_file_by_path(&aff.file_path) {
                if let Some(fid) = file_rec.id {
                    if let Ok(dep_ids) = memory.get_dependent_file_ids(fid) {
                        for dep_id in dep_ids {
                            if let Ok(Some(dep_file)) = memory.get_file_by_id(dep_id) {
                                let already_has_symbol_caller = callers.iter().any(|c| {
                                    c.caller_file == dep_file.path
                                        && c.target_symbol == aff.symbol.name
                                });
                                if !already_has_symbol_caller {
                                    let key = format!("{}:mod:{}", dep_file.path, aff.symbol.name);
                                    if seen.insert(key) {
                                        callers.push(AffectedCaller {
                                            caller_file: dep_file.path,
                                            caller_symbol: None,
                                            caller_line: None,
                                            target_symbol: aff.symbol.name.clone(),
                                            target_file: aff.file_path.clone(),
                                            edge_type: "imports".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        callers.sort_by(|a, b| {
            a.caller_file
                .cmp(&b.caller_file)
                .then(a.target_symbol.cmp(&b.target_symbol))
        });
        Ok(callers)
    }

    /// Check if changed source files have corresponding test files or unit tests
    pub fn check_test_coverage(
        repo_root: &Path,
        changed_files: &[ReviewFileSummary],
        memory: &MemoryEngine,
    ) -> Result<Vec<TestGap>> {
        let mut gaps = Vec::new();
        let tests_dir = repo_root.join("tests");

        for cf in changed_files {
            // Skip non-code files, docs, or files already in test directories
            let path_lower = cf.path.to_lowercase();
            if path_lower.starts_with("tests/")
                || path_lower.starts_with("test/")
                || path_lower.contains("test")
                || path_lower.ends_with(".md")
                || path_lower.ends_with(".toml")
                || path_lower.ends_with(".json")
            {
                continue;
            }

            let path_obj = Path::new(&cf.path);
            let stem = match path_obj.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };

            // Check if a test file matches the stem in tests/
            let mut has_test = false;
            if tests_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&tests_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        if name.contains(&stem.to_lowercase()) {
                            has_test = true;
                            break;
                        }
                    }
                }
            }

            // Check if file has incoming 'covers_test' edges in memory
            if !has_test {
                if let Ok(Some(file_rec)) = memory.get_file_by_path(&cf.path) {
                    if let Some(fid) = file_rec.id {
                        if let Ok(edges) = memory.get_edges_to_file(fid) {
                            if edges.iter().any(|e| e.edge_type == "covers_test") {
                                has_test = true;
                            }
                        }
                    }
                }
            }

            if !has_test {
                let suggestion = format!("Create tests/{}_tests.rs", stem);
                gaps.push(TestGap {
                    file: cf.path.clone(),
                    suggestion,
                });
            }
        }

        Ok(gaps)
    }

    /// Identify docs that will become stale due to changes in source files
    pub fn check_doc_staleness(
        changed_files: &[ReviewFileSummary],
        memory: &MemoryEngine,
    ) -> Result<Vec<StaleDocInfo>> {
        let mut stale_docs = Vec::new();

        for cf in changed_files {
            if let Ok(Some(file_rec)) = memory.get_file_by_path(&cf.path) {
                if let Some(fid) = file_rec.id {
                    if let Ok(Some(doc)) = memory.get_doc_by_file_id(fid) {
                        stale_docs.push(StaleDocInfo {
                            file_path: cf.path.clone(),
                            doc_path: doc.doc_path,
                            reason: "will become STALE".to_string(),
                        });
                    } else if let Some(doc_p) = file_rec.doc_path {
                        stale_docs.push(StaleDocInfo {
                            file_path: cf.path.clone(),
                            doc_path: doc_p,
                            reason: "will become STALE".to_string(),
                        });
                    } else {
                        // Conventional doc path
                        let conv_doc =
                            format!(".agent-docs/{}.md", cf.path.trim_end_matches(".rs"));
                        stale_docs.push(StaleDocInfo {
                            file_path: cf.path.clone(),
                            doc_path: conv_doc,
                            reason: "will become STALE".to_string(),
                        });
                    }
                }
            }
        }

        stale_docs.sort_by(|a, b| a.doc_path.cmp(&b.doc_path));
        stale_docs.dedup_by(|a, b| a.doc_path == b.doc_path);
        Ok(stale_docs)
    }
}

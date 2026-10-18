use std::collections::HashSet;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::Result;
use crate::memory::file_memory::FileRecord;
use crate::memory::symbol_memory::SymbolRecord;
use crate::memory::MemoryEngine;
use crate::scanner::git::{ChangedFile, ChangeType, GitCommitInfo, GitScanner};
use crate::scanner::languages::Language;
use crate::scanner::tree_sitter::AstParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRecord {
    pub id: Option<i64>,
    pub hash: String,
    pub message: String,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub summary: Option<String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitChangeRecord {
    pub id: Option<i64>,
    pub commit_id: i64,
    pub file_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub change_type: String, // added, modified, deleted, renamed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchedSymbolInfo {
    pub symbol_id: Option<i64>,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAnalysis {
    pub commit: CommitRecord,
    pub changes: Vec<CommitChangeRecord>,
    pub changed_files_count: usize,
    pub affected_symbols_count: usize,
    pub touched_symbols: Vec<TouchedSymbolInfo>,
    pub risk_level: String,
    pub markdown_summary: String,
}

pub struct CommitAnalyzer;

impl CommitAnalyzer {
    /// Analyze a commit, detecting changed files, modified line ranges, touched AST symbols,
    /// deterministic risk level, and generating a structured markdown summary.
    pub fn analyze(
        repo_root: &Path,
        commit_info: &GitCommitInfo,
        memory: &MemoryEngine,
    ) -> Result<CommitAnalysis> {
        let changed_files = GitScanner::get_commit_changed_files_for_hash(repo_root, &commit_info.hash)?;
        let diff_text = GitScanner::get_commit_diff(repo_root, &commit_info.hash).unwrap_or_default();
        let hunk_lines_by_file = GitScanner::parse_diff_hunks_changed_lines(&diff_text);

        let mut changes = Vec::new();
        let mut touched_symbols = Vec::new();
        let mut touched_symbol_ids = HashSet::new();

        for changed in &changed_files {
            let rel_path_str = changed.path.to_string_lossy().replace('\\', "/");
            let file_full_path = repo_root.join(&changed.path);

            // Step 1: Ensure file record and symbols exist in SQLite
            let file_id = match changed.change_type {
                ChangeType::Deleted => {
                    // Look up existing file id if previously indexed
                    if let Some(existing) = memory.get_file_by_path(&rel_path_str)? {
                        let fid = existing.id.unwrap_or(0);
                        // Update status to deleted
                        let mut updated = existing;
                        updated.parse_status = Some("deleted".to_string());
                        let _ = memory.upsert_file(&updated);
                        Some(fid)
                    } else {
                        None
                    }
                }
                _ => {
                    // File is Added, Modified, or Renamed
                    if file_full_path.exists() && file_full_path.is_file() {
                        let content = fs::read(&file_full_path).unwrap_or_default();
                        let size = content.len();
                        let mut hasher = Sha256::new();
                        hasher.update(&content);
                        let source_hash = format!("{:x}", hasher.finalize());

                        let lang = Language::detect_from_path(&file_full_path);
                        let lang_str = lang.as_str().to_string();

                        let file_rec = FileRecord {
                            id: None,
                            path: rel_path_str.clone(),
                            language: Some(lang_str.clone()),
                            size,
                            source_hash: Some(source_hash),
                            parse_status: Some("parsed".to_string()),
                            last_commit: Some(commit_info.hash.clone()),
                            last_modified: Some(commit_info.timestamp.clone()),
                            summary: None,
                            risk_level: None,
                            doc_path: None,
                            doc_status: Some("unindexed".to_string()),
                        };

                        let fid = memory.upsert_file(&file_rec)?;

                        // If file is in an AST-supported language, extract symbols and imports
                        if lang.is_ast_supported() {
                            if let Ok(content_str) = std::str::from_utf8(&content) {
                                if let Ok(ast_result) = AstParser::parse_source(content_str, lang) {
                                    memory.clear_symbols_for_file(fid)?;
                                    if !ast_result.symbols.is_empty() {
                                        let sym_records: Vec<SymbolRecord> = ast_result.symbols
                                            .iter()
                                            .map(|s| SymbolRecord {
                                                id: None,
                                                file_id: fid,
                                                name: s.name.clone(),
                                                kind: s.kind.as_str().to_string(),
                                                line_start: s.line_start,
                                                line_end: s.line_end,
                                                visibility: Some(s.visibility.clone()),
                                                signature: s.signature.clone(),
                                                summary: None,
                                                parent: s.parent.clone(),
                                            })
                                            .collect();
                                        memory.insert_symbols(fid, &sym_records)?;
                                    }

                                    if !ast_result.imports_raw.is_empty() {
                                        memory.insert_raw_imports(fid, &ast_result.imports_raw)?;
                                    }
                                }
                            }
                        }

                        Some(fid)
                    } else if let Some(existing) = memory.get_file_by_path(&rel_path_str)? {
                        existing.id
                    } else {
                        None
                    }
                }
            };

            // Step 2: Match touched symbols
            let mut file_symbols_touched = false;

            if let Some(fid) = file_id {
                let symbols = memory.get_symbols_for_file(fid)?;

                match changed.change_type {
                    ChangeType::Added => {
                        // All symbols in an added file are touched
                        for sym in &symbols {
                            let sid = sym.id.unwrap_or(0);
                            if touched_symbol_ids.insert(sid) {
                                touched_symbols.push(TouchedSymbolInfo {
                                    symbol_id: sym.id,
                                    symbol_name: sym.name.clone(),
                                    symbol_kind: sym.kind.clone(),
                                    file_path: rel_path_str.clone(),
                                    line_start: sym.line_start,
                                    line_end: sym.line_end,
                                    change_type: "added".to_string(),
                                });
                                changes.push(CommitChangeRecord {
                                    id: None,
                                    commit_id: 0,
                                    file_id: Some(fid),
                                    symbol_id: sym.id,
                                    change_type: "added".to_string(),
                                });
                                file_symbols_touched = true;
                            }
                        }
                    }
                    ChangeType::Modified => {
                        let ranges = hunk_lines_by_file.get(&changed.path);
                        if let Some(changed_ranges) = ranges {
                            for sym in &symbols {
                                let overlaps = changed_ranges.iter().any(|&(start, end)| {
                                    start <= sym.line_end && end >= sym.line_start
                                });

                                if overlaps {
                                    let sid = sym.id.unwrap_or(0);
                                    if touched_symbol_ids.insert(sid) {
                                        touched_symbols.push(TouchedSymbolInfo {
                                            symbol_id: sym.id,
                                            symbol_name: sym.name.clone(),
                                            symbol_kind: sym.kind.clone(),
                                            file_path: rel_path_str.clone(),
                                            line_start: sym.line_start,
                                            line_end: sym.line_end,
                                            change_type: "modified".to_string(),
                                        });
                                        changes.push(CommitChangeRecord {
                                            id: None,
                                            commit_id: 0,
                                            file_id: Some(fid),
                                            symbol_id: sym.id,
                                            change_type: "modified".to_string(),
                                        });
                                        file_symbols_touched = true;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // If no individual symbol was recorded for this file, record file-level change
            if !file_symbols_touched {
                changes.push(CommitChangeRecord {
                    id: None,
                    commit_id: 0,
                    file_id,
                    symbol_id: None,
                    change_type: changed.change_type.description().to_string(),
                });
            }
        }

        let risk_level = Self::calculate_risk_level(&changed_files, touched_symbols.len()).to_string();
        let markdown_summary = Self::generate_markdown_summary(
            commit_info,
            &changed_files,
            &touched_symbols,
            &risk_level,
        );

        let commit_record = CommitRecord {
            id: None,
            hash: commit_info.hash.clone(),
            message: commit_info.message.clone(),
            author: Some(format!("{} <{}>", commit_info.author_name, commit_info.author_email)),
            created_at: Some(commit_info.timestamp.clone()),
            summary: Some(markdown_summary.clone()),
            risk_level: Some(risk_level.clone()),
        };

        Ok(CommitAnalysis {
            commit: commit_record,
            changes,
            changed_files_count: changed_files.len(),
            affected_symbols_count: touched_symbols.len(),
            touched_symbols,
            risk_level,
            markdown_summary,
        })
    }

    /// Calculate deterministic risk level based on changed files, paths, and touched symbols.
    pub fn calculate_risk_level(changed_files: &[ChangedFile], touched_symbols_count: usize) -> &'static str {
        let is_critical_config = changed_files.iter().any(|f| {
            let path_str = f.path.to_string_lossy().to_lowercase().replace('\\', "/");
            let file_name = f.path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_lowercase();

            file_name == "cargo.toml"
                || file_name == "cargo.lock"
                || file_name == "package.json"
                || file_name == "pnpm-lock.yaml"
                || file_name == "yarn.lock"
                || file_name.starts_with(".env")
                || file_name == "dockerfile"
                || file_name.contains("docker-compose")
                || path_str.contains(".github/workflows")
                || path_str.contains("/auth/")
                || path_str.contains("/crypto/")
                || path_str.contains("/security/")
                || path_str.contains("/db/")
                || path_str.contains("/migrations/")
        });

        let has_deletions = changed_files.iter().any(|f| f.change_type == ChangeType::Deleted);

        if is_critical_config || has_deletions || changed_files.len() > 8 {
            "high"
        } else if changed_files.len() > 2 || touched_symbols_count > 3 {
            "medium"
        } else {
            "low"
        }
    }

    /// Generate deterministic markdown summary (Level 0 summary, zero LLM tokens).
    pub fn generate_markdown_summary(
        commit_info: &GitCommitInfo,
        changed_files: &[ChangedFile],
        touched_symbols: &[TouchedSymbolInfo],
        risk_level: &str,
    ) -> String {
        let hash_short = if commit_info.hash.len() >= 8 {
            &commit_info.hash[0..8]
        } else {
            &commit_info.hash
        };

        let mut out = String::new();
        out.push_str(&format!("### Commit {}: {}\n", hash_short, commit_info.message));
        out.push_str(&format!(
            "- **Author:** {} <{}>\n- **Date:** {}\n- **Risk Level:** {}\n\n",
            commit_info.author_name, commit_info.author_email, commit_info.timestamp, risk_level
        ));

        out.push_str("#### Impact Overview\n");
        out.push_str(&format!(
            "Commit modified **{}** file(s) and directly affected **{}** symbol(s).\n\n",
            changed_files.len(),
            touched_symbols.len()
        ));

        out.push_str(&format!("#### Changed Files ({})\n", changed_files.len()));
        if changed_files.is_empty() {
            out.push_str("*(No files changed)*\n\n");
        } else {
            for f in changed_files {
                let p = f.path.to_string_lossy().replace('\\', "/");
                out.push_str(&format!("- {} `{}` ({})\n", f.change_type.badge(), p, f.change_type.description()));
            }
            out.push('\n');
        }

        out.push_str(&format!("#### Affected Symbols ({})\n", touched_symbols.len()));
        if touched_symbols.is_empty() {
            out.push_str("*(No AST symbols directly touched by changed hunks)*\n\n");
        } else {
            for sym in touched_symbols {
                out.push_str(&format!(
                    "- `{}` ({} in `{}`:L{}-L{}, {})\n",
                    sym.symbol_name, sym.symbol_kind, sym.file_path, sym.line_start, sym.line_end, sym.change_type
                ));
            }
            out.push('\n');
        }

        out.push_str("#### Recommended Verification\n");
        match risk_level {
            "high" => {
                out.push_str("- [ ] Run `cargo check` & full test suite `cargo test`\n");
                out.push_str("- [ ] Review security-critical & configuration changes\n");
                out.push_str("- [ ] Verify database schema and migration compatibility\n");
            }
            "medium" => {
                out.push_str("- [ ] Run `cargo check`\n");
                out.push_str("- [ ] Run tests for modified modules and dependents\n");
            }
            _ => {
                out.push_str("- [ ] Run `cargo check`\n");
            }
        }

        out
    }
}

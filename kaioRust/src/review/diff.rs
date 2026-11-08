use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::memory::{MemoryEngine, SymbolRecord};
use crate::scanner::git::ChangeType;
use crate::scanner::languages::Language;
use crate::scanner::tree_sitter::AstParser;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewFileSummary {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub added_lines: Vec<String>,
    pub deleted_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDiff {
    pub files: Vec<ReviewFileSummary>,
    pub hunks_by_file: HashMap<String, Vec<DiffHunk>>,
    pub raw_diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BreakingChange {
    pub file_path: String,
    pub line_number: usize,
    pub symbol_name: String,
    pub old_signature: String,
    pub new_signature: Option<String>,
    pub description: String,
    pub affected_callers_count: usize,
}

#[derive(Debug, Clone)]
pub struct AffectedSymbolInfo {
    pub symbol: SymbolRecord,
    pub file_path: String,
    pub is_breaking: bool,
    pub old_signature: String,
    pub new_signature: Option<String>,
    pub breaking_reason: Option<String>,
}

pub struct DiffAnalyzer;

impl DiffAnalyzer {
    /// Parse unified diff text into structured file summaries and hunk line ranges
    pub fn parse_unified_diff(diff_text: &str) -> ParsedDiff {
        let mut files_map: HashMap<String, (usize, usize, ChangeType)> = HashMap::new();
        let mut hunks_by_file: HashMap<String, Vec<DiffHunk>> = HashMap::new();
        let mut current_file: Option<String> = None;
        let mut current_hunk: Option<DiffHunk> = None;

        for line in diff_text.lines() {
            if line.starts_with("diff --git ") {
                if let (Some(cf), Some(hunk)) = (current_file.take(), current_hunk.take()) {
                    hunks_by_file.entry(cf).or_default().push(hunk);
                }
                current_file = None;
            } else if line.starts_with("--- ") {
                // Ignore old file header
            } else if let Some(stripped) = line.strip_prefix("+++ b/") {
                let path = stripped.trim().trim_matches('"').replace('\\', "/");
                current_file = Some(path.clone());
                files_map
                    .entry(path)
                    .or_insert((0, 0, ChangeType::Modified));
            } else if line.starts_with("+++ /dev/null") {
                // Deleted file
            } else if line.starts_with("@@ ") {
                if let (Some(ref cf), Some(hunk)) = (&current_file, current_hunk.take()) {
                    hunks_by_file.entry(cf.clone()).or_default().push(hunk);
                }

                if let Some((old_start, old_count, new_start, new_count)) =
                    Self::parse_hunk_header(line)
                {
                    current_hunk = Some(DiffHunk {
                        old_start,
                        old_count,
                        new_start,
                        new_count,
                        added_lines: Vec::new(),
                        deleted_lines: Vec::new(),
                    });
                }
            } else if let Some(ref mut hunk) = current_hunk {
                if line.starts_with('+') && !line.starts_with("+++") {
                    hunk.added_lines.push(line[1..].to_string());
                    if let Some(ref cf) = current_file {
                        let entry =
                            files_map
                                .entry(cf.clone())
                                .or_insert((0, 0, ChangeType::Modified));
                        entry.0 += 1;
                    }
                } else if line.starts_with('-') && !line.starts_with("---") {
                    hunk.deleted_lines.push(line[1..].to_string());
                    if let Some(ref cf) = current_file {
                        let entry =
                            files_map
                                .entry(cf.clone())
                                .or_insert((0, 0, ChangeType::Modified));
                        entry.1 += 1;
                    }
                }
            }
        }

        if let (Some(cf), Some(hunk)) = (current_file, current_hunk) {
            hunks_by_file.entry(cf).or_default().push(hunk);
        }

        let mut files: Vec<ReviewFileSummary> = files_map
            .into_iter()
            .map(
                |(path, (additions, deletions, change_type))| ReviewFileSummary {
                    path,
                    additions,
                    deletions,
                    change_type,
                },
            )
            .collect();

        files.sort_by(|a, b| a.path.cmp(&b.path));

        ParsedDiff {
            files,
            hunks_by_file,
            raw_diff: diff_text.to_string(),
        }
    }

    fn parse_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
        // Format: @@ -old_start,old_count +new_start,new_count @@
        let at_trimmed = line.strip_prefix("@@ ")?;
        let header_end = at_trimmed.find(" @@")?;
        let header = &at_trimmed[..header_end];

        let mut parts = header.split_whitespace();
        let old_part = parts.next()?.strip_prefix('-')?;
        let new_part = parts.next()?.strip_prefix('+')?;

        let (old_start, old_count) = if let Some((s, c)) = old_part.split_once(',') {
            (s.parse::<usize>().ok()?, c.parse::<usize>().ok()?)
        } else {
            (old_part.parse::<usize>().ok()?, 1)
        };

        let (new_start, new_count) = if let Some((s, c)) = new_part.split_once(',') {
            (s.parse::<usize>().ok()?, c.parse::<usize>().ok()?)
        } else {
            (new_part.parse::<usize>().ok()?, 1)
        };

        Some((old_start, old_count, new_start, new_count))
    }

    /// Match changed diff hunks with stored memory symbols to identify affected and breaking symbols
    pub fn match_symbols(
        repo_root: &Path,
        parsed_diff: &ParsedDiff,
        memory: &MemoryEngine,
    ) -> Result<(Vec<AffectedSymbolInfo>, Vec<BreakingChange>)> {
        let mut affected = Vec::new();
        let mut breaking = Vec::new();

        for file_summary in &parsed_diff.files {
            let file_rec = match memory.get_file_by_path(&file_summary.path)? {
                Some(f) => f,
                None => continue,
            };
            let file_id = match file_rec.id {
                Some(id) => id,
                None => continue,
            };

            let stored_symbols = memory.get_symbols_for_file(file_id)?;
            let hunks = parsed_diff
                .hunks_by_file
                .get(&file_summary.path)
                .cloned()
                .unwrap_or_default();

            // Read current on-disk version of the file if present to parse updated AST symbols
            let current_full_path = repo_root.join(&file_summary.path);
            let fresh_symbols = if current_full_path.exists() {
                if let Ok(source) = std::fs::read_to_string(&current_full_path) {
                    let lang = Language::detect_from_path(&current_full_path);
                    AstParser::parse_source(&source, lang)
                        .ok()
                        .map(|res| res.symbols)
                } else {
                    None
                }
            } else {
                None
            };

            for sym in stored_symbols {
                // Check if symbol overlaps with any modified hunk
                let is_touched = hunks.iter().any(|h| {
                    let h_start = h.new_start;
                    let h_end = if h.new_count == 0 {
                        h_start
                    } else {
                        h_start + h.new_count.saturating_sub(1)
                    };
                    sym.line_start <= h_end && sym.line_end >= h_start
                });

                if !is_touched {
                    continue;
                }

                let is_public = sym
                    .visibility
                    .as_deref()
                    .map(|v| v == "pub" || v == "public")
                    .unwrap_or(false);
                let old_sig = sym.signature.clone().unwrap_or_else(|| sym.name.clone());

                let mut is_breaking = false;
                let mut new_sig = None;
                let mut breaking_reason = None;

                if is_public {
                    if let Some(ref fresh) = fresh_symbols {
                        // Look for the symbol in the newly extracted AST
                        let matching_fresh = fresh.iter().find(|s| {
                            s.name == sym.name
                                && (s.kind.as_str() == sym.kind
                                    || (s.kind.as_str() == "function" && sym.kind == "fn")
                                    || (s.kind.as_str() == "method" && sym.kind == "fn")
                                    || (s.kind.as_str() == "fn" && sym.kind == "function"))
                        });
                        match matching_fresh {
                            Some(fresh_s) => {
                                let f_sig = fresh_s
                                    .signature
                                    .clone()
                                    .unwrap_or_else(|| fresh_s.name.clone());
                                if f_sig != old_sig {
                                    is_breaking = true;
                                    new_sig = Some(f_sig.clone());
                                    breaking_reason = Some(format!(
                                        "Signature changed: `{}` → `{}`",
                                        old_sig, f_sig
                                    ));
                                }
                            }
                            None => {
                                // Symbol was removed or renamed
                                is_breaking = true;
                                breaking_reason = Some(format!(
                                    "Public symbol `{}` ({}) removed or renamed",
                                    sym.name, sym.kind
                                ));
                            }
                        }
                    } else {
                        // File was deleted or couldn't be read: removing public symbols is breaking
                        is_breaking = true;
                        breaking_reason = Some(format!("Public symbol `{}` removed", sym.name));
                    }
                }

                if is_breaking {
                    breaking.push(BreakingChange {
                        file_path: file_summary.path.clone(),
                        line_number: sym.line_start,
                        symbol_name: sym.name.clone(),
                        old_signature: old_sig.clone(),
                        new_signature: new_sig.clone(),
                        description: breaking_reason.clone().unwrap_or_default(),
                        affected_callers_count: 0, // Populated in impact analysis
                    });
                }

                affected.push(AffectedSymbolInfo {
                    symbol: sym,
                    file_path: file_summary.path.clone(),
                    is_breaking,
                    old_signature: old_sig,
                    new_signature: new_sig,
                    breaking_reason,
                });
            }
        }

        Ok((affected, breaking))
    }
}

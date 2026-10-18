use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::docs::frontmatter::DocFrontmatter;
use crate::docs::markdown::{CommitDocEntry, FileDocTemplate, SymbolDocEntry};
use crate::error::{KaiokenError, Result};
use crate::memory::{DocRecord, FileRecord, MemoryEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocGenerateOptions {
    pub level: u8,
    pub force: bool,
    pub stale_only: bool,
    pub no_llm: bool,
    pub limit: Option<usize>,
}

impl Default for DocGenerateOptions {
    fn default() -> Self {
        Self {
            level: 0,
            force: false,
            stale_only: false,
            no_llm: true,
            limit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocGenerationResult {
    pub file_id: i64,
    pub source_path: String,
    pub doc_path: PathBuf,
    pub relative_doc_path: String,
    pub doc_level: u8,
    pub symbols_count: usize,
    pub imports_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocBatchSummary {
    pub total_processed: usize,
    pub generated: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: usize,
}

pub struct DocGenerator {
    repo_root: PathBuf,
    docs_root: PathBuf,
}

impl DocGenerator {
    pub fn new(repo_root: impl Into<PathBuf>, docs_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
            docs_root: docs_root.into(),
        }
    }

    pub fn docs_root(&self) -> &Path {
        &self.docs_root
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Compute the mirrored markdown path in `.agent-docs/` for a given relative source path.
    pub fn mirror_path_for_source(&self, source_relative_path: &Path) -> PathBuf {
        let mut mirrored = self.docs_root.join(source_relative_path);
        if let Some(ext) = source_relative_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if matches!(
                ext_lower.as_str(),
                "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "c" | "cpp" | "h" | "hpp" | "java" | "cs" | "rb" | "php"
            ) {
                mirrored.set_extension("md");
            } else if ext_lower == "md" {
                // Already markdown, keep as is
            } else {
                let filename = source_relative_path.file_name().unwrap_or_default().to_string_lossy();
                mirrored.set_file_name(format!("{}.md", filename));
            }
        } else {
            mirrored.set_extension("md");
        }
        mirrored
    }

    /// Generate documentation for a specific file record.
    pub fn generate_file_doc(
        &self,
        file: &FileRecord,
        memory: &MemoryEngine,
        options: &DocGenerateOptions,
    ) -> Result<DocGenerationResult> {
        let file_id = file.id.ok_or_else(|| {
            KaiokenError::DocError(format!("File {} has no database id", file.path))
        })?;

        let source_rel_path = PathBuf::from(&file.path);
        let target_abs_path = self.mirror_path_for_source(&source_rel_path);
        let rel_doc_path = match target_abs_path.strip_prefix(&self.docs_root) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => target_abs_path.to_string_lossy().replace('\\', "/"),
        };

        // Compute current source hash from disk (source of truth)
        let disk_source_path = self.repo_root.join(&file.path);
        let actual_source_hash = if disk_source_path.exists() {
            if let Ok(bytes) = fs::read(&disk_source_path) {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                Some(format!("{:x}", hasher.finalize()))
            } else {
                file.source_hash.clone()
            }
        } else {
            file.source_hash.clone()
        };

        // Check if existing doc exists and whether we can skip
        let existing_doc = memory.get_doc_by_file_id(file_id)?;
        if !options.force {
            if let Some(ref doc) = existing_doc {
                let is_up_to_date = target_abs_path.exists()
                    && doc.source_hash.as_deref() == actual_source_hash.as_deref()
                    && doc.status == "fresh";

                if is_up_to_date {
                    return Ok(DocGenerationResult {
                        file_id,
                        source_path: file.path.clone(),
                        doc_path: target_abs_path,
                        relative_doc_path: rel_doc_path,
                        doc_level: doc.doc_level,
                        symbols_count: 0,
                        imports_count: 0,
                        status: "skipped_fresh".to_string(),
                    });
                }
            }
        }

        // Fetch AST symbols
        let symbols = memory.get_symbols_for_file(file_id)?;
        let mut public_api = Vec::new();
        let mut internal_symbols = Vec::new();

        for sym in &symbols {
            let entry = SymbolDocEntry {
                name: sym.name.clone(),
                kind: sym.kind.clone(),
                visibility: sym.visibility.clone().unwrap_or_else(|| "private".to_string()),
                signature: sym.signature.clone(),
                line_start: sym.line_start,
                line_end: sym.line_end,
            };
            if entry.visibility.starts_with("pub") {
                public_api.push(entry);
            } else {
                internal_symbols.push(entry);
            }
        }

        // Fetch Raw Imports / Dependencies
        let raw_imports = memory.get_raw_imports_for_file(file_id)?;
        let dependencies: Vec<String> = raw_imports.into_iter().map(|i| i.raw_import).collect();
        let imports_count = dependencies.len();
        let symbols_count = symbols.len();

        // Fetch Recent Commits
        let recent_commits = memory.get_commits_for_file(file_id, 3)?;
        let commit_entries: Vec<CommitDocEntry> = recent_commits
            .into_iter()
            .map(|c| CommitDocEntry {
                hash_short: if c.hash.len() >= 8 { c.hash[0..8].to_string() } else { c.hash },
                message: c.message,
                author: c.author,
                date: c.created_at,
            })
            .collect();

        // Infer Purpose (Level 0: deterministic fact-based heuristics)
        let purpose = file.summary.clone().unwrap_or_else(|| {
            Self::infer_deterministic_purpose(&file.path, file.language.as_deref().unwrap_or("unknown"), &public_api)
        });

        // Risks
        let mut risks = Vec::new();
        if let Some(r) = &file.risk_level {
            risks.push(format!("Risk classification: {}", r));
        }
        if file.size > 100_000 {
            risks.push("Large file size (> 100 KB); consider modular decomposition".to_string());
        }
        if file.parse_status.as_deref() == Some("partial") {
            risks.push("File has syntax errors; AST parsing was partially recovered".to_string());
        }

        let frontmatter = DocFrontmatter {
            source_path: file.path.clone(),
            source_hash: actual_source_hash.clone().unwrap_or_default(),
            doc_level: options.level,
            generated_at: chrono::Utc::now().to_rfc3339(),
            generator: if options.no_llm { "deterministic".to_string() } else { "llm".to_string() },
            status: "fresh".to_string(),
        };

        let used_by = memory.get_used_by_for_file(file_id).unwrap_or_default();

        let template = FileDocTemplate {
            frontmatter,
            file_path: file.path.clone(),
            language: file.language.clone().unwrap_or_else(|| "unknown".to_string()),
            purpose,
            public_api,
            internal_symbols,
            dependencies,
            used_by,
            tests: Vec::new(),
            recent_changes: commit_entries,
            risks,
            notes: None,
        };

        let rendered = template.render();

        // Ensure parent directory exists in .agent-docs/
        if let Some(parent) = target_abs_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write mirrored markdown doc
        fs::write(&target_abs_path, rendered)?;

        // Persist doc record in SQLite
        let doc_record = DocRecord {
            id: None,
            file_id,
            doc_path: rel_doc_path.clone(),
            doc_level: options.level,
            source_hash: actual_source_hash,
            status: "fresh".to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            stale_reason: None,
        };
        memory.upsert_doc(&doc_record)?;

        Ok(DocGenerationResult {
            file_id,
            source_path: file.path.clone(),
            doc_path: target_abs_path,
            relative_doc_path: rel_doc_path,
            doc_level: options.level,
            symbols_count,
            imports_count,
            status: if existing_doc.is_some() { "updated".to_string() } else { "generated".to_string() },
        })
    }

    /// Batch generate docs across all files in repository memory.
    pub fn generate_all_docs(
        &self,
        memory: &MemoryEngine,
        options: &DocGenerateOptions,
    ) -> Result<DocBatchSummary> {
        let files = memory.list_files()?;
        let mut summary = DocBatchSummary::default();

        for file in files {
            if let Some(limit) = options.limit {
                if summary.generated + summary.updated >= limit {
                    break;
                }
            }

            // Filter binary or unparseable files
            if let Some(ref status) = file.parse_status {
                if status == "binary" || status == "too_large" || status == "deleted" {
                    summary.skipped += 1;
                    continue;
                }
            }

            match self.generate_file_doc(&file, memory, options) {
                Ok(res) => {
                    summary.total_processed += 1;
                    match res.status.as_str() {
                        "generated" => summary.generated += 1,
                        "updated" => summary.updated += 1,
                        "skipped_fresh" => summary.skipped += 1,
                        _ => summary.generated += 1,
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to generate doc for {}: {}", file.path, e);
                    summary.errors += 1;
                }
            }
        }

        Ok(summary)
    }

    fn infer_deterministic_purpose(path: &str, language: &str, public_api: &[SymbolDocEntry]) -> String {
        let p = path.replace('\\', "/");
        if p.ends_with("main.rs") {
            "Executable application entry point and CLI runner.".to_string()
        } else if p.ends_with("lib.rs") {
            "Library root exposing modules and public crate interface.".to_string()
        } else if p.ends_with("mod.rs") {
            format!("Module definition and re-exports for `{}` subsystem.", p.trim_end_matches("/mod.rs"))
        } else if p.contains("/db.rs") || p.contains("/database") {
            "Database connection lifecycle, schema migration, and SQL query interface.".to_string()
        } else if p.contains("/config") {
            "Application configuration loading, serialization, and environment handling.".to_string()
        } else if p.contains("/error") {
            "Error taxonomy, typed domain failures, and error conversion implementations.".to_string()
        } else if p.contains("/scanner") {
            "Codebase scanner, filesystem traversal, Git change tracking, and AST extraction.".to_string()
        } else if p.contains("/memory") {
            "Relational codebase memory engine and SQLite synchronization.".to_string()
        } else if p.contains("/cli") {
            "Command-line arguments parser, subcommand dispatch, and terminal output.".to_string()
        } else if !public_api.is_empty() {
            let names: Vec<&str> = public_api.iter().take(3).map(|s| s.name.as_str()).collect();
            format!("{} module providing public symbols: `{}`.", language, names.join("`, `"))
        } else {
            format!("Source file for {} containing internal definitions.", language)
        }
    }
}

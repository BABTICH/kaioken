use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::memory::{FileRecord, MemoryEngine, SymbolRecord};
use crate::scanner::file_inventory::ParseStatus;
use crate::scanner::languages::Language;
use crate::scanner::tree_sitter::AstParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    Updated {
        path: PathBuf,
        symbols_count: usize,
        changed_hash: bool,
        old_hash: Option<String>,
        new_hash: String,
    },
    NewFile {
        path: PathBuf,
        symbols_count: usize,
        new_hash: String,
    },
    Deleted {
        path: PathBuf,
    },
    Unchanged {
        path: PathBuf,
        hash: String,
    },
    Ignored {
        path: PathBuf,
        reason: String,
    },
    Failed {
        path: PathBuf,
        error: String,
    },
}

pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Incrementally synchronize a single file change into SQLite memory.
pub fn sync_path(
    repo_root: &Path,
    raw_path: &Path,
    memory: &MemoryEngine,
    skip_ast: bool,
) -> Result<SyncOutcome> {
    let rel_path = if raw_path.is_absolute() {
        raw_path
            .strip_prefix(repo_root)
            .unwrap_or(raw_path)
            .to_path_buf()
    } else {
        raw_path.to_path_buf()
    };

    let rel_str = rel_path.to_string_lossy().replace('\\', "/");
    let full_path = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        repo_root.join(raw_path)
    };

    // 1. File Deletion Handling
    if !full_path.exists() {
        if let Some(existing) = memory.get_file_by_path(&rel_str)? {
            if let Some(fid) = existing.id {
                let _ = memory.clear_symbols_for_file(fid);
            }
            memory.delete_file(&rel_str)?;
            return Ok(SyncOutcome::Deleted { path: rel_path });
        } else {
            return Ok(SyncOutcome::Ignored {
                path: rel_path,
                reason: "File does not exist on disk and is not in memory".to_string(),
            });
        }
    }

    // 2. Directory Ignore Handling
    if full_path.is_dir() {
        return Ok(SyncOutcome::Ignored {
            path: rel_path,
            reason: "Target path is a directory".to_string(),
        });
    }

    // 3. Read file contents and compute hash
    let bytes = match fs::read(&full_path) {
        Ok(b) => b,
        Err(e) => {
            return Ok(SyncOutcome::Failed {
                path: rel_path,
                error: format!("Failed to read file: {}", e),
            });
        }
    };

    let new_hash = compute_sha256(&bytes);
    let existing_opt = memory.get_file_by_path(&rel_str)?;

    if let Some(existing) = existing_opt {
        // Unchanged check: skip if hash identical
        if existing.source_hash.as_deref() == Some(&new_hash) {
            return Ok(SyncOutcome::Unchanged {
                path: rel_path,
                hash: new_hash,
            });
        }

        // File modified!
        let language = Language::detect_from_path(&full_path);
        let mut symbols_rec = Vec::new();
        let mut parse_status = ParseStatus::Ok;

        if !skip_ast && language.is_ast_supported() {
            if let Ok(utf8) = std::str::from_utf8(&bytes) {
                match AstParser::parse_source(utf8, language) {
                    Ok(res) => {
                        parse_status = res.parse_status;
                        for s in res.symbols {
                            symbols_rec.push(SymbolRecord {
                                id: None,
                                file_id: 0,
                                name: s.name,
                                kind: s.kind.as_str().to_string(),
                                line_start: s.line_start,
                                line_end: s.line_end,
                                visibility: Some(s.visibility),
                                signature: s.signature,
                                summary: None,
                                parent: s.parent,
                            });
                        }
                    }
                    Err(_) => {
                        parse_status = ParseStatus::Partial;
                    }
                }
            } else {
                parse_status = ParseStatus::Failed;
            }
        }

        let mut updated_rec = existing.clone();
        let old_hash = updated_rec.source_hash.clone();
        updated_rec.source_hash = Some(new_hash.clone());
        updated_rec.size = bytes.len();
        updated_rec.parse_status = Some(parse_status.as_str().to_string());
        updated_rec.last_modified = Some(chrono::Utc::now().to_rfc3339());

        let file_id = memory.upsert_file(&updated_rec)?;

        if !skip_ast {
            memory.clear_symbols_for_file(file_id)?;
            if !symbols_rec.is_empty() {
                memory.insert_symbols(file_id, &symbols_rec)?;
            }
        }

        // Staleness cascade
        let reason = format!("File modified on disk: {}", rel_str);
        let _ = memory.mark_doc_status(file_id, "stale", Some(&reason));
        if let Ok(dep_ids) = memory.get_dependent_file_ids(file_id) {
            for dep_id in dep_ids {
                let dep_reason = format!("Dependency '{}' modified on disk", rel_str);
                let _ = memory.mark_doc_status(dep_id, "maybe_stale", Some(&dep_reason));
            }
        }

        Ok(SyncOutcome::Updated {
            path: rel_path,
            symbols_count: symbols_rec.len(),
            changed_hash: true,
            old_hash,
            new_hash,
        })
    } else {
        // New file added!
        let language = Language::detect_from_path(&full_path);
        let mut symbols_rec = Vec::new();
        let mut parse_status = ParseStatus::Ok;

        if !skip_ast && language.is_ast_supported() {
            if let Ok(utf8) = std::str::from_utf8(&bytes) {
                match AstParser::parse_source(utf8, language) {
                    Ok(res) => {
                        parse_status = res.parse_status;
                        for s in res.symbols {
                            symbols_rec.push(SymbolRecord {
                                id: None,
                                file_id: 0,
                                name: s.name,
                                kind: s.kind.as_str().to_string(),
                                line_start: s.line_start,
                                line_end: s.line_end,
                                visibility: Some(s.visibility),
                                signature: s.signature,
                                summary: None,
                                parent: s.parent,
                            });
                        }
                    }
                    Err(_) => {
                        parse_status = ParseStatus::Partial;
                    }
                }
            } else {
                parse_status = ParseStatus::Failed;
            }
        }

        let new_rec = FileRecord {
            id: None,
            path: rel_str.clone(),
            language: Some(language.as_str().to_string()),
            size: bytes.len(),
            source_hash: Some(new_hash.clone()),
            parse_status: Some(parse_status.as_str().to_string()),
            last_commit: None,
            last_modified: Some(chrono::Utc::now().to_rfc3339()),
            summary: None,
            risk_level: None,
            doc_path: None,
            doc_status: None,
        };

        let file_id = memory.upsert_file(&new_rec)?;
        if !skip_ast && !symbols_rec.is_empty() {
            memory.insert_symbols(file_id, &symbols_rec)?;
        }

        Ok(SyncOutcome::NewFile {
            path: rel_path,
            symbols_count: symbols_rec.len(),
            new_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sync_new_file_and_hash_skip() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let mem = MemoryEngine::open_in_memory().unwrap();

        let file_path = repo_root.join("src/lib.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

        // 1. Initial sync -> NewFile
        let outcome1 = sync_path(repo_root, &file_path, &mem, false).unwrap();
        match outcome1 {
            SyncOutcome::NewFile {
                path,
                symbols_count,
                ..
            } => {
                assert_eq!(path, PathBuf::from("src/lib.rs"));
                assert_eq!(symbols_count, 1);
            }
            _ => panic!("Expected NewFile"),
        }

        // Verify stored in memory
        let file_rec = mem.get_file_by_path("src/lib.rs").unwrap().unwrap();
        assert_eq!(file_rec.language.as_deref(), Some("rust"));
        let syms = mem.get_symbols_for_file(file_rec.id.unwrap()).unwrap();
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "add");

        // 2. Sync again without modifying -> Unchanged (hash skip)
        let outcome2 = sync_path(repo_root, &file_path, &mem, false).unwrap();
        match outcome2 {
            SyncOutcome::Unchanged { path, .. } => {
                assert_eq!(path, PathBuf::from("src/lib.rs"));
            }
            _ => panic!("Expected Unchanged"),
        }

        // 3. Modify file content -> Updated
        fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub fn sub(a: i32, b: i32) -> i32 { a - b }").unwrap();
        let outcome3 = sync_path(repo_root, &file_path, &mem, false).unwrap();
        match outcome3 {
            SyncOutcome::Updated {
                path,
                symbols_count,
                changed_hash,
                ..
            } => {
                assert_eq!(path, PathBuf::from("src/lib.rs"));
                assert!(changed_hash);
                assert_eq!(symbols_count, 2);
            }
            _ => panic!("Expected Updated"),
        }

        // 4. Delete file -> Deleted
        fs::remove_file(&file_path).unwrap();
        let outcome4 = sync_path(repo_root, &file_path, &mem, false).unwrap();
        match outcome4 {
            SyncOutcome::Deleted { path } => {
                assert_eq!(path, PathBuf::from("src/lib.rs"));
            }
            _ => panic!("Expected Deleted"),
        }
        assert!(mem.get_file_by_path("src/lib.rs").unwrap().is_none());
    }
}

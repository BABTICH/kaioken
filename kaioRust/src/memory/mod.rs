pub mod db;
pub mod file_memory;
pub mod symbol_memory;
pub mod commit_memory;
pub mod doc_memory;
pub mod graph;
pub mod failure_memory;
pub mod decision_memory;

use std::path::Path;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use crate::error::{KaiokenError, Result};
use crate::scanner::file_inventory::InventorySummary;

pub use db::Database;
pub use file_memory::{FileRecord, RawImportRecord};
pub use symbol_memory::SymbolRecord;
pub use commit_memory::{CommitRecord, CommitChangeRecord, CommitAnalysis, CommitAnalyzer, TouchedSymbolInfo};
pub use doc_memory::DocRecord;
pub use graph::{EdgeRecord, DependencyGraph};
pub use failure_memory::FailureRecord;
pub use decision_memory::DecisionRecord;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    pub files_synced: usize,
    pub symbols_synced: usize,
    pub imports_synced: usize,
}

pub struct MemoryEngine {
    db: Database,
}

impl MemoryEngine {
    pub fn open(db_path: &Path) -> Result<Self> {
        let db = Database::open(db_path)?;
        Ok(Self { db })
    }

    pub fn open_in_memory() -> Result<Self> {
        let db = Database::open_in_memory()?;
        Ok(Self { db })
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    // ==========================================
    // 1. File Memory CRUD
    // ==========================================

    pub fn upsert_file(&self, record: &FileRecord) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO files (path, language, size, source_hash, parse_status, last_commit, last_modified, summary, risk_level, doc_path, doc_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(path) DO UPDATE SET
               language = excluded.language,
               size = excluded.size,
               source_hash = excluded.source_hash,
               parse_status = excluded.parse_status,
               last_commit = COALESCE(excluded.last_commit, files.last_commit),
               last_modified = COALESCE(excluded.last_modified, files.last_modified),
               summary = COALESCE(excluded.summary, files.summary),
               risk_level = COALESCE(excluded.risk_level, files.risk_level),
               doc_path = COALESCE(excluded.doc_path, files.doc_path),
               doc_status = COALESCE(excluded.doc_status, files.doc_status);",
            params![
                record.path,
                record.language,
                record.size as i64,
                record.source_hash,
                record.parse_status,
                record.last_commit,
                record.last_modified,
                record.summary,
                record.risk_level,
                record.doc_path,
                record.doc_status,
            ],
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let id: i64 = conn.query_row(
            "SELECT id FROM files WHERE path = ?1;",
            params![record.path],
            |row| row.get(0),
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(id)
    }

    pub fn get_file_by_path(&self, path: &str) -> Result<Option<FileRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, path, language, size, source_hash, parse_status, last_commit, last_modified, summary, risk_level, doc_path, doc_status
             FROM files WHERE path = ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let file = stmt.query_row(params![path], |row| {
            let size_i64: i64 = row.get(3)?;
            Ok(FileRecord {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                language: row.get(2)?,
                size: size_i64 as usize,
                source_hash: row.get(4)?,
                parse_status: row.get(5)?,
                last_commit: row.get(6)?,
                last_modified: row.get(7)?,
                summary: row.get(8)?,
                risk_level: row.get(9)?,
                doc_path: row.get(10)?,
                doc_status: row.get(11)?,
            })
        }).optional().map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(file)
    }

    pub fn get_file_by_id(&self, id: i64) -> Result<Option<FileRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, path, language, size, source_hash, parse_status, last_commit, last_modified, summary, risk_level, doc_path, doc_status
             FROM files WHERE id = ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let file = stmt.query_row(params![id], |row| {
            let size_i64: i64 = row.get(3)?;
            Ok(FileRecord {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                language: row.get(2)?,
                size: size_i64 as usize,
                source_hash: row.get(4)?,
                parse_status: row.get(5)?,
                last_commit: row.get(6)?,
                last_modified: row.get(7)?,
                summary: row.get(8)?,
                risk_level: row.get(9)?,
                doc_path: row.get(10)?,
                doc_status: row.get(11)?,
            })
        }).optional().map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(file)
    }

    pub fn list_files(&self) -> Result<Vec<FileRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, path, language, size, source_hash, parse_status, last_commit, last_modified, summary, risk_level, doc_path, doc_status
             FROM files ORDER BY path ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let size_i64: i64 = row.get(3)?;
            Ok(FileRecord {
                id: Some(row.get(0)?),
                path: row.get(1)?,
                language: row.get(2)?,
                size: size_i64 as usize,
                source_hash: row.get(4)?,
                parse_status: row.get(5)?,
                last_commit: row.get(6)?,
                last_modified: row.get(7)?,
                summary: row.get(8)?,
                risk_level: row.get(9)?,
                doc_path: row.get(10)?,
                doc_status: row.get(11)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM files WHERE path = ?1;", params![path])
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(())
    }

    pub fn count_files(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM files;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    // ==========================================
    // 2. Symbol Memory CRUD
    // ==========================================

    pub fn insert_symbols(&self, file_id: i64, symbols: &[SymbolRecord]) -> Result<()> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "INSERT INTO symbols (file_id, name, kind, line_start, line_end, visibility, signature, summary, parent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        for sym in symbols {
            stmt.execute(params![
                file_id,
                sym.name,
                sym.kind,
                sym.line_start as i64,
                sym.line_end as i64,
                sym.visibility,
                sym.signature,
                sym.summary,
                sym.parent,
            ]).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn get_symbols_for_file(&self, file_id: i64) -> Result<Vec<SymbolRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, name, kind, line_start, line_end, visibility, signature, summary, parent
             FROM symbols WHERE file_id = ?1 ORDER BY line_start ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![file_id], |row| {
            let start: i64 = row.get(4)?;
            let end: i64 = row.get(5)?;
            Ok(SymbolRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
                line_start: start as usize,
                line_end: end as usize,
                visibility: row.get(6)?,
                signature: row.get(7)?,
                summary: row.get(8)?,
                parent: row.get(9)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn find_symbols_by_name(&self, name: &str) -> Result<Vec<SymbolRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, name, kind, line_start, line_end, visibility, signature, summary, parent
             FROM symbols WHERE name = ?1 ORDER BY id ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![name], |row| {
            let start: i64 = row.get(4)?;
            let end: i64 = row.get(5)?;
            Ok(SymbolRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
                line_start: start as usize,
                line_end: end as usize,
                visibility: row.get(6)?,
                signature: row.get(7)?,
                summary: row.get(8)?,
                parent: row.get(9)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn clear_symbols_for_file(&self, file_id: i64) -> Result<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM symbols WHERE file_id = ?1;", params![file_id])
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(())
    }

    pub fn count_symbols(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM symbols;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    // ==========================================
    // 3. Raw Imports CRUD
    // ==========================================

    pub fn insert_raw_imports(&self, file_id: i64, imports_raw: &[String]) -> Result<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM import_raw WHERE file_id = ?1;", params![file_id])
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut stmt = conn.prepare(
            "INSERT INTO import_raw (file_id, raw_import, status) VALUES (?1, ?2, 'unresolved');",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        for imp in imports_raw {
            stmt.execute(params![file_id, imp])
                .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn get_raw_imports_for_file(&self, file_id: i64) -> Result<Vec<RawImportRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, raw_import, resolved_file_id, resolved_symbol_id, status
             FROM import_raw WHERE file_id = ?1 ORDER BY id ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![file_id], |row| {
            Ok(RawImportRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                raw_import: row.get(2)?,
                resolved_file_id: row.get(3)?,
                resolved_symbol_id: row.get(4)?,
                status: row.get(5)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn count_raw_imports(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM import_raw;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    // ==========================================
    // 4. Edges (Dependency Graph) CRUD
    // ==========================================

    pub fn insert_edge(&self, edge: &EdgeRecord) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO edges (source_symbol_id, target_symbol_id, edge_type) VALUES (?1, ?2, ?3);",
            params![edge.source_symbol_id, edge.target_symbol_id, edge.edge_type],
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    pub fn get_edges_for_source(&self, source_symbol_id: i64) -> Result<Vec<EdgeRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, source_symbol_id, target_symbol_id, edge_type
             FROM edges WHERE source_symbol_id = ?1 ORDER BY id ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![source_symbol_id], |row| {
            Ok(EdgeRecord {
                id: Some(row.get(0)?),
                source_symbol_id: row.get(1)?,
                target_symbol_id: row.get(2)?,
                edge_type: row.get(3)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_edges_for_target(&self, target_symbol_id: i64) -> Result<Vec<EdgeRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, source_symbol_id, target_symbol_id, edge_type
             FROM edges WHERE target_symbol_id = ?1 ORDER BY id ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![target_symbol_id], |row| {
            Ok(EdgeRecord {
                id: Some(row.get(0)?),
                source_symbol_id: row.get(1)?,
                target_symbol_id: row.get(2)?,
                edge_type: row.get(3)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn count_edges(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM edges;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    // ==========================================
    // 5. Commit Memory CRUD
    // ==========================================

    pub fn record_commit(&self, commit: &CommitRecord, changes: &[CommitChangeRecord]) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO commits (hash, message, author, created_at, summary, risk_level)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(hash) DO UPDATE SET
               message = excluded.message,
               author = excluded.author,
               created_at = excluded.created_at,
               summary = excluded.summary,
               risk_level = excluded.risk_level;",
            params![
                commit.hash,
                commit.message,
                commit.author,
                commit.created_at,
                commit.summary,
                commit.risk_level,
            ],
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let commit_id: i64 = conn.query_row(
            "SELECT id FROM commits WHERE hash = ?1;",
            params![commit.hash],
            |row| row.get(0),
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        // Clear previous changes if re-recording this commit
        conn.execute(
            "DELETE FROM commit_changes WHERE commit_id = ?1;",
            params![commit_id],
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut stmt = conn.prepare(
            "INSERT INTO commit_changes (commit_id, file_id, symbol_id, change_type)
             VALUES (?1, ?2, ?3, ?4);",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        for c in changes {
            stmt.execute(params![
                commit_id,
                c.file_id,
                c.symbol_id,
                c.change_type,
            ]).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

            // If file_id is present, update files.last_commit
            if let Some(fid) = c.file_id {
                let _ = conn.execute(
                    "UPDATE files SET last_commit = ?1 WHERE id = ?2;",
                    params![commit.hash, fid],
                );
            }
        }

        // Automatic Staleness Cascade for files touched in the commit
        let mut touched_file_ids = std::collections::HashSet::new();
        for c in changes {
            if let Some(fid) = c.file_id {
                touched_file_ids.insert(fid);
            }
        }

        let short_hash = if commit.hash.len() >= 8 { &commit.hash[0..8] } else { &commit.hash };
        let reason_direct = format!("Modified in commit {}", short_hash);
        for &fid in &touched_file_ids {
            let _ = self.mark_doc_status(fid, "stale", Some(&reason_direct));

            // Direct 1-hop dependents -> maybe_stale
            if let Ok(dep_ids) = self.get_dependent_file_ids(fid) {
                for dep_id in dep_ids {
                    if !touched_file_ids.contains(&dep_id) {
                        let source_name = self.get_file_by_id(fid)
                            .ok().flatten()
                            .map(|f| f.path)
                            .unwrap_or_else(|| format!("file_id {}", fid));
                        let reason_dep = format!("Dependency '{}' modified in commit {}", source_name, short_hash);
                        let _ = self.mark_doc_status(dep_id, "maybe_stale", Some(&reason_dep));
                    }
                }
            }
        }

        Ok(commit_id)
    }

    pub fn record_commit_analysis(&self, analysis: &CommitAnalysis) -> Result<i64> {
        self.record_commit(&analysis.commit, &analysis.changes)
    }

    pub fn get_commit_by_hash(&self, hash: &str) -> Result<Option<CommitRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, hash, message, author, created_at, summary, risk_level
             FROM commits WHERE hash = ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let commit = stmt.query_row(params![hash], |row| {
            Ok(CommitRecord {
                id: Some(row.get(0)?),
                hash: row.get(1)?,
                message: row.get(2)?,
                author: row.get(3)?,
                created_at: row.get(4)?,
                summary: row.get(5)?,
                risk_level: row.get(6)?,
            })
        }).optional().map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(commit)
    }

    pub fn get_commit_by_id(&self, id: i64) -> Result<Option<CommitRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, hash, message, author, created_at, summary, risk_level
             FROM commits WHERE id = ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let commit = stmt.query_row(params![id], |row| {
            Ok(CommitRecord {
                id: Some(row.get(0)?),
                hash: row.get(1)?,
                message: row.get(2)?,
                author: row.get(3)?,
                created_at: row.get(4)?,
                summary: row.get(5)?,
                risk_level: row.get(6)?,
            })
        }).optional().map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(commit)
    }

    pub fn get_recent_commits(&self, limit: usize) -> Result<Vec<CommitRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, hash, message, author, created_at, summary, risk_level
             FROM commits ORDER BY id DESC LIMIT ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(CommitRecord {
                id: Some(row.get(0)?),
                hash: row.get(1)?,
                message: row.get(2)?,
                author: row.get(3)?,
                created_at: row.get(4)?,
                summary: row.get(5)?,
                risk_level: row.get(6)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_changes_for_commit(&self, commit_id: i64) -> Result<Vec<CommitChangeRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, commit_id, file_id, symbol_id, change_type
             FROM commit_changes WHERE commit_id = ?1 ORDER BY id ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![commit_id], |row| {
            Ok(CommitChangeRecord {
                id: Some(row.get(0)?),
                commit_id: row.get(1)?,
                file_id: row.get(2)?,
                symbol_id: row.get(3)?,
                change_type: row.get(4)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn count_commits(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM commits;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    pub fn count_commit_changes(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM commit_changes;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    // ==========================================
    // 6. Documentation CRUD
    // ==========================================

    pub fn upsert_doc(&self, doc: &DocRecord) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO docs (file_id, doc_path, doc_level, source_hash, status, updated_at, stale_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(file_id) DO UPDATE SET
               doc_path = excluded.doc_path,
               doc_level = excluded.doc_level,
               source_hash = excluded.source_hash,
               status = excluded.status,
               updated_at = excluded.updated_at,
               stale_reason = excluded.stale_reason;",
            params![
                doc.file_id,
                doc.doc_path,
                doc.doc_level as i64,
                doc.source_hash,
                doc.status,
                doc.updated_at,
                doc.stale_reason,
            ],
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let id: i64 = conn.query_row(
            "SELECT id FROM docs WHERE file_id = ?1;",
            params![doc.file_id],
            |row| row.get(0),
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        // Update files table metadata
        let _ = conn.execute(
            "UPDATE files SET doc_path = ?1, doc_status = ?2 WHERE id = ?3;",
            params![doc.doc_path, doc.status, doc.file_id],
        );

        Ok(id)
    }

    pub fn get_doc_by_file_id(&self, file_id: i64) -> Result<Option<DocRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, doc_path, doc_level, source_hash, status, updated_at, stale_reason
             FROM docs WHERE file_id = ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let doc = stmt.query_row(params![file_id], |row| {
            let level_i64: i64 = row.get(3)?;
            Ok(DocRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                doc_path: row.get(2)?,
                doc_level: level_i64 as u8,
                source_hash: row.get(4)?,
                status: row.get(5)?,
                updated_at: row.get(6)?,
                stale_reason: row.get(7)?,
            })
        }).optional().map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(doc)
    }

    pub fn get_doc_by_path(&self, doc_path: &str) -> Result<Option<DocRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, doc_path, doc_level, source_hash, status, updated_at, stale_reason
             FROM docs WHERE doc_path = ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let doc = stmt.query_row(params![doc_path], |row| {
            let level_i64: i64 = row.get(3)?;
            Ok(DocRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                doc_path: row.get(2)?,
                doc_level: level_i64 as u8,
                source_hash: row.get(4)?,
                status: row.get(5)?,
                updated_at: row.get(6)?,
                stale_reason: row.get(7)?,
            })
        }).optional().map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        Ok(doc)
    }

    pub fn list_docs(&self) -> Result<Vec<DocRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, doc_path, doc_level, source_hash, status, updated_at, stale_reason
             FROM docs ORDER BY id ASC;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let level_i64: i64 = row.get(3)?;
            Ok(DocRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                doc_path: row.get(2)?,
                doc_level: level_i64 as u8,
                source_hash: row.get(4)?,
                status: row.get(5)?,
                updated_at: row.get(6)?,
                stale_reason: row.get(7)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn count_docs(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM docs;", [], |r| r.get(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(count as usize)
    }

    pub fn count_docs_by_status(&self) -> Result<std::collections::HashMap<String, usize>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM docs GROUP BY status;")
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let status: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((status, count as usize))
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut map = std::collections::HashMap::new();
        for r in rows {
            let (status, count) = r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
            map.insert(status, count);
        }
        Ok(map)
    }

    pub fn get_commits_for_file(&self, file_id: i64, limit: usize) -> Result<Vec<CommitRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.id, c.hash, c.message, c.author, c.created_at, c.summary, c.risk_level
             FROM commits c
             JOIN commit_changes cc ON cc.commit_id = c.id
             WHERE cc.file_id = ?1
             ORDER BY c.id DESC LIMIT ?2;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let rows = stmt.query_map(params![file_id, limit as i64], |row| {
            Ok(CommitRecord {
                id: Some(row.get(0)?),
                hash: row.get(1)?,
                message: row.get(2)?,
                author: row.get(3)?,
                created_at: row.get(4)?,
                summary: row.get(5)?,
                risk_level: row.get(6)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_dependent_file_ids(&self, file_id: i64) -> Result<Vec<i64>> {
        let conn = self.db.connection();
        let mut dependents = std::collections::HashSet::new();

        // 1. Direct import_raw match where resolved_file_id == file_id
        let mut stmt1 = conn.prepare(
            "SELECT DISTINCT file_id FROM import_raw WHERE resolved_file_id = ?1 AND file_id != ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        let rows1 = stmt1.query_map(params![file_id], |r| r.get::<_, i64>(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        for r in rows1 {
            if let Ok(id) = r {
                dependents.insert(id);
            }
        }

        // 2. Edges match where target symbol belongs to file_id
        let mut stmt2 = conn.prepare(
            "SELECT DISTINCT s_src.file_id
             FROM edges e
             JOIN symbols s_src ON e.source_symbol_id = s_src.id
             JOIN symbols s_tgt ON e.target_symbol_id = s_tgt.id
             WHERE s_tgt.file_id = ?1 AND s_src.file_id != ?1;",
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        let rows2 = stmt2.query_map(params![file_id], |r| r.get::<_, i64>(0))
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        for r in rows2 {
            if let Ok(id) = r {
                dependents.insert(id);
            }
        }

        // 3. Module name heuristic for unresolved imports
        if let Ok(Some(file_rec)) = self.get_file_by_id(file_id) {
            let path = std::path::Path::new(&file_rec.path);
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.len() >= 3 && stem != "mod" && stem != "lib" && stem != "main" {
                    let pattern1 = format!("%::{}%", stem);
                    let pattern2 = format!("%{}::%", stem);
                    let mut stmt3 = conn.prepare(
                        "SELECT DISTINCT file_id FROM import_raw 
                         WHERE file_id != ?1 AND (raw_import LIKE ?2 OR raw_import LIKE ?3);",
                    ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
                    let rows3 = stmt3.query_map(params![file_id, pattern1, pattern2], |r| r.get::<_, i64>(0))
                        .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
                    for r in rows3 {
                        if let Ok(id) = r {
                            dependents.insert(id);
                        }
                    }
                }
            }
        }

        let mut res: Vec<i64> = dependents.into_iter().collect();
        res.sort_unstable();
        Ok(res)
    }

    pub fn get_used_by_for_file(&self, file_id: i64) -> Result<Vec<String>> {
        let dep_ids = self.get_dependent_file_ids(file_id)?;
        let mut paths = Vec::new();
        for dep_id in dep_ids {
            if let Ok(Some(f)) = self.get_file_by_id(dep_id) {
                paths.push(f.path);
            }
        }
        paths.sort();
        Ok(paths)
    }

    pub fn mark_doc_status(&self, file_id: i64, status: &str, stale_reason: Option<&str>) -> Result<bool> {
        let conn = self.db.connection();
        let now = chrono::Utc::now().to_rfc3339();

        let affected = conn.execute(
            "UPDATE docs SET status = ?1, stale_reason = ?2, updated_at = ?3 WHERE file_id = ?4;",
            params![status, stale_reason, now, file_id],
        ).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let _ = conn.execute(
            "UPDATE files SET doc_status = ?1 WHERE id = ?2;",
            params![status, file_id],
        );

        if affected == 0 {
            // No doc existed yet: create a placeholder so staleness is recorded
            if let Ok(Some(file_rec)) = self.get_file_by_id(file_id) {
                let default_doc_path = format!("{}.md", file_rec.path.trim_end_matches(".rs"));
                let _ = conn.execute(
                    "INSERT INTO docs (file_id, doc_path, doc_level, source_hash, status, updated_at, stale_reason)
                     VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6)
                     ON CONFLICT(file_id) DO UPDATE SET status = excluded.status, stale_reason = excluded.stale_reason;",
                    params![
                        file_id,
                        default_doc_path,
                        file_rec.source_hash,
                        status,
                        now,
                        stale_reason,
                    ],
                );
            }
            return Ok(false);
        }

        Ok(true)
    }

    pub fn list_docs_by_status(&self, statuses: &[&str]) -> Result<Vec<DocRecord>> {
        let conn = self.db.connection();
        if statuses.is_empty() {
            return self.list_docs();
        }

        let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT id, file_id, doc_path, doc_level, source_hash, status, updated_at, stale_reason
             FROM docs WHERE status IN ({}) ORDER BY id ASC;",
            placeholders
        );

        let mut stmt = conn.prepare(&query).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        let rusqlite_params = rusqlite::params_from_iter(statuses.iter());

        let rows = stmt.query_map(rusqlite_params, |row| {
            let level_i64: i64 = row.get(3)?;
            Ok(DocRecord {
                id: Some(row.get(0)?),
                file_id: row.get(1)?,
                doc_path: row.get(2)?,
                doc_level: level_i64 as u8,
                source_hash: row.get(4)?,
                status: row.get(5)?,
                updated_at: row.get(6)?,
                stale_reason: row.get(7)?,
            })
        }).map_err(|e| KaiokenError::MemoryError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| KaiokenError::MemoryError(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn delete_doc_by_id(&self, doc_id: i64) -> Result<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM docs WHERE id = ?1;", params![doc_id])
            .map_err(|e| KaiokenError::MemoryError(e.to_string()))?;
        Ok(())
    }

    // ==========================================
    // 7. Inventory Synchronization Bridge
    // ==========================================

    pub fn sync_inventory(&self, summary: &InventorySummary) -> Result<SyncStats> {
        let mut files_synced = 0;
        let mut symbols_synced = 0;
        let mut imports_synced = 0;

        for discovered in &summary.files {
            let rel_path_str = discovered.relative_path.to_string_lossy().replace('\\', "/");
            let file_rec = FileRecord {
                id: None,
                path: rel_path_str,
                language: Some(discovered.language.as_str().to_string()),
                size: discovered.size,
                source_hash: discovered.file_hash.clone(),
                parse_status: Some(discovered.parse_status.as_str().to_string()),
                last_commit: None,
                last_modified: discovered.last_modified.clone(),
                summary: None,
                risk_level: None,
                doc_path: None,
                doc_status: Some("unindexed".to_string()),
            };

            let file_id = self.upsert_file(&file_rec)?;
            files_synced += 1;

            // Clear previous symbols and insert freshly extracted symbols
            self.clear_symbols_for_file(file_id)?;
            if !discovered.symbols.is_empty() {
                let symbol_records: Vec<SymbolRecord> = discovered.symbols.iter().map(|s| {
                    SymbolRecord {
                        id: None,
                        file_id,
                        name: s.name.clone(),
                        kind: s.kind.as_str().to_string(),
                        line_start: s.line_start,
                        line_end: s.line_end,
                        visibility: Some(s.visibility.clone()),
                        signature: s.signature.clone(),
                        summary: None,
                        parent: s.parent.clone(),
                    }
                }).collect();

                self.insert_symbols(file_id, &symbol_records)?;
                symbols_synced += symbol_records.len();
            }

            // Sync raw imports
            if !discovered.imports_raw.is_empty() {
                self.insert_raw_imports(file_id, &discovered.imports_raw)?;
                imports_synced += discovered.imports_raw.len();
            }
        }

        Ok(SyncStats {
            files_synced,
            symbols_synced,
            imports_synced,
        })
    }

    // ==========================================
    // Failures Memory
    // ==========================================

    pub fn insert_failure(&self, record: &FailureRecord) -> Result<i64> {
        let conn = self.db.connection();
        let status = record.status.as_deref().unwrap_or("unresolved");
        conn.execute(
            "INSERT INTO failures (error_type, message, file_id, symbol_id, fix_summary, status, resolved_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.error_type,
                record.message,
                record.file_id,
                record.symbol_id,
                record.fix_summary,
                status,
                record.resolved_at,
                record.created_at,
            ],
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to insert failure record: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_failures_for_file(&self, file_id: i64) -> Result<Vec<FailureRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, error_type, message, file_id, symbol_id, fix_summary, status, resolved_at, created_at
                 FROM failures WHERE file_id = ?1 ORDER BY id DESC",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare failures query: {}", e)))?;

        let rows = stmt
            .query_map(params![file_id], |row| {
                Ok(FailureRecord {
                    id: Some(row.get(0)?),
                    error_type: row.get(1)?,
                    message: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    fix_summary: row.get(5)?,
                    status: row.get(6)?,
                    resolved_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query failures: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read failure record: {}", e)))?);
        }
        Ok(results)
    }

    pub fn get_failures_for_symbol(&self, symbol_id: i64) -> Result<Vec<FailureRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, error_type, message, file_id, symbol_id, fix_summary, status, resolved_at, created_at
                 FROM failures WHERE symbol_id = ?1 ORDER BY id DESC",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare failures query: {}", e)))?;

        let rows = stmt
            .query_map(params![symbol_id], |row| {
                Ok(FailureRecord {
                    id: Some(row.get(0)?),
                    error_type: row.get(1)?,
                    message: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    fix_summary: row.get(5)?,
                    status: row.get(6)?,
                    resolved_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query failures: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read failure record: {}", e)))?);
        }
        Ok(results)
    }

    pub fn get_recent_failures(&self, limit: usize) -> Result<Vec<FailureRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, error_type, message, file_id, symbol_id, fix_summary, status, resolved_at, created_at
                 FROM failures ORDER BY id DESC LIMIT ?1",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare recent failures query: {}", e)))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(FailureRecord {
                    id: Some(row.get(0)?),
                    error_type: row.get(1)?,
                    message: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    fix_summary: row.get(5)?,
                    status: row.get(6)?,
                    resolved_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query recent failures: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read failure record: {}", e)))?);
        }
        Ok(results)
    }

    pub fn count_failures(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM failures", [], |row| row.get(0))
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to count failures: {}", e)))?;
        Ok(count as usize)
    }

    pub fn resolve_failure(&self, failure_id: i64, fix_summary: &str) -> Result<()> {
        let conn = self.db.connection();
        let now = chrono::Utc::now().to_rfc3339();
        let rows_affected = conn
            .execute(
                "UPDATE failures 
                 SET status = 'resolved', fix_summary = ?1, resolved_at = ?2 
                 WHERE id = ?3",
                params![fix_summary, now, failure_id],
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to resolve failure record: {}", e)))?;

        if rows_affected == 0 {
            return Err(KaiokenError::MemoryError(format!("Failure record with id {} not found", failure_id)));
        }
        Ok(())
    }

    pub fn get_unresolved_failures(&self) -> Result<Vec<FailureRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, error_type, message, file_id, symbol_id, fix_summary, status, resolved_at, created_at
                 FROM failures WHERE status = 'unresolved' ORDER BY id DESC",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare unresolved failures query: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(FailureRecord {
                    id: Some(row.get(0)?),
                    error_type: row.get(1)?,
                    message: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    fix_summary: row.get(5)?,
                    status: row.get(6)?,
                    resolved_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query unresolved failures: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read failure record: {}", e)))?);
        }
        Ok(results)
    }

    pub fn get_failure_by_id(&self, id: i64) -> Result<Option<FailureRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, error_type, message, file_id, symbol_id, fix_summary, status, resolved_at, created_at
                 FROM failures WHERE id = ?1",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare failure by id query: {}", e)))?;

        let failure = stmt
            .query_row(params![id], |row| {
                Ok(FailureRecord {
                    id: Some(row.get(0)?),
                    error_type: row.get(1)?,
                    message: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    fix_summary: row.get(5)?,
                    status: row.get(6)?,
                    resolved_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .optional()
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query failure by id: {}", e)))?;

        Ok(failure)
    }

    // ==========================================
    // Architectural Decisions (ADRs)
    // ==========================================

    pub fn insert_decision(&self, record: &DecisionRecord) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO decisions (title, description, status, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                record.title,
                record.description,
                record.status,
                record.created_at,
            ],
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to insert decision record: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_decisions(&self, status: Option<&str>) -> Result<Vec<DecisionRecord>> {
        let conn = self.db.connection();
        let mut results = Vec::new();

        if let Some(s) = status {
            let mut stmt = conn
                .prepare("SELECT id, title, description, status, created_at FROM decisions WHERE status = ?1 ORDER BY id DESC")
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare decisions query: {}", e)))?;
            let rows = stmt
                .query_map(params![s], |row| {
                    Ok(DecisionRecord {
                        id: Some(row.get(0)?),
                        title: row.get(1)?,
                        description: row.get(2)?,
                        status: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to query decisions: {}", e)))?;
            for r in rows {
                results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read decision: {}", e)))?);
            }
        } else {
            let mut stmt = conn
                .prepare("SELECT id, title, description, status, created_at FROM decisions ORDER BY id DESC")
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare decisions query: {}", e)))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(DecisionRecord {
                        id: Some(row.get(0)?),
                        title: row.get(1)?,
                        description: row.get(2)?,
                        status: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                })
                .map_err(|e| KaiokenError::MemoryError(format!("Failed to query decisions: {}", e)))?;
            for r in rows {
                results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read decision: {}", e)))?);
            }
        }

        Ok(results)
    }

    pub fn count_decisions(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM decisions", [], |row| row.get(0))
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to count decisions: {}", e)))?;
        Ok(count as usize)
    }

    // ==========================================
    // Agent Runs & Tool Calls
    // ==========================================

    pub fn start_agent_run(&self, task: &str, model: &str) -> Result<i64> {
        let conn = self.db.connection();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO agent_runs (task, status, model, started_at, total_tokens, estimated_cost)
             VALUES (?1, 'running', ?2, ?3, 0, 0.0)",
            params![task, model, now],
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to start agent run: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn finish_agent_run(
        &self,
        run_id: i64,
        status: &str,
        total_tokens: usize,
        estimated_cost: f64,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.db.connection();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE agent_runs
             SET status = ?1, finished_at = ?2, total_tokens = ?3, estimated_cost = ?4, error = ?5
             WHERE id = ?6",
            params![
                status,
                now,
                total_tokens as i64,
                estimated_cost,
                error,
                run_id,
            ],
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to finish agent run: {}", e)))?;

        Ok(())
    }

    pub fn get_agent_run(&self, run_id: i64) -> Result<Option<AgentRunRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, task, status, model, started_at, finished_at, total_tokens, estimated_cost, context_packet_id, error
                 FROM agent_runs WHERE id = ?1",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare agent_runs query: {}", e)))?;

        let res = stmt
            .query_row(params![run_id], |row| {
                let total_tokens_i64: i64 = row.get(6)?;
                Ok(AgentRunRecord {
                    id: Some(row.get(0)?),
                    task: row.get(1)?,
                    status: row.get(2)?,
                    model: row.get(3)?,
                    started_at: row.get(4)?,
                    finished_at: row.get(5)?,
                    total_tokens: total_tokens_i64 as usize,
                    estimated_cost: row.get(7)?,
                    context_packet_id: row.get(8)?,
                    error: row.get(9)?,
                })
            })
            .optional()
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query agent run: {}", e)))?;

        Ok(res)
    }

    pub fn list_agent_runs(&self, limit: usize) -> Result<Vec<AgentRunRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, task, status, model, started_at, finished_at, total_tokens, estimated_cost, context_packet_id, error
                 FROM agent_runs ORDER BY id DESC LIMIT ?1",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare agent_runs list query: {}", e)))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let total_tokens_i64: i64 = row.get(6)?;
                Ok(AgentRunRecord {
                    id: Some(row.get(0)?),
                    task: row.get(1)?,
                    status: row.get(2)?,
                    model: row.get(3)?,
                    started_at: row.get(4)?,
                    finished_at: row.get(5)?,
                    total_tokens: total_tokens_i64 as usize,
                    estimated_cost: row.get(7)?,
                    context_packet_id: row.get(8)?,
                    error: row.get(9)?,
                })
            })
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query agent runs list: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read agent run: {}", e)))?);
        }
        Ok(results)
    }

    pub fn record_tool_call(&self, record: &ToolCallRecord) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO tool_calls (agent_run_id, tool_name, input_json, output_summary, status, duration_ms, error, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.agent_run_id,
                record.tool_name,
                record.input_json,
                record.output_summary,
                record.status,
                record.duration_ms,
                record.error,
                record.created_at,
            ],
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to insert tool_call record: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_tool_calls_for_run(&self, run_id: i64) -> Result<Vec<ToolCallRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, agent_run_id, tool_name, input_json, output_summary, status, duration_ms, error, created_at
                 FROM tool_calls WHERE agent_run_id = ?1 ORDER BY id ASC",
            )
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare tool_calls query: {}", e)))?;

        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(ToolCallRecord {
                    id: Some(row.get(0)?),
                    agent_run_id: row.get(1)?,
                    tool_name: row.get(2)?,
                    input_json: row.get(3)?,
                    output_summary: row.get(4)?,
                    status: row.get(5)?,
                    duration_ms: row.get(6)?,
                    error: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to query tool_calls: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| KaiokenError::MemoryError(format!("Failed to read tool call: {}", e)))?);
        }
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunRecord {
    pub id: Option<i64>,
    pub task: String,
    pub status: String,
    pub model: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub total_tokens: usize,
    pub estimated_cost: f64,
    pub context_packet_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: Option<i64>,
    pub agent_run_id: i64,
    pub tool_name: String,
    pub input_json: String,
    pub output_summary: Option<String>,
    pub status: String,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub created_at: String,
}

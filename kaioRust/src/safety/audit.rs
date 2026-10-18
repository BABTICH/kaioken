use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::error::{KaiokenError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Option<i64>,
    pub actor: String,         // 'agent' | 'user'
    pub action: String,        // 'edit_file' | 'bash' | 'create_file' | 'read_file'
    pub target: String,        // file path or command string
    pub approved: bool,        // true = allowed, false = blocked
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Default)]
pub struct AuditLogger {
    log_file_path: Option<PathBuf>,
    db_path: Option<PathBuf>,
    shared_conn: Option<Arc<Mutex<rusqlite::Connection>>>,
}

impl AuditLogger {
    pub fn new(repo_root: Option<&Path>) -> Self {
        let log_file_path = repo_root.map(|r| r.join(".agent").join("logs").join("audit.log"));
        let db_path = repo_root.map(|r| r.join(".agent").join("memory.sqlite"));
        Self {
            log_file_path,
            db_path,
            shared_conn: None,
        }
    }

    pub fn with_db_path(mut self, db_path: PathBuf) -> Self {
        self.db_path = Some(db_path);
        self
    }

    pub fn with_shared_conn(mut self, conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        self.shared_conn = Some(conn);
        self
    }

    pub fn with_log_file(mut self, log_path: PathBuf) -> Self {
        self.log_file_path = Some(log_path);
        self
    }

    /// Logs an action to both file disk log and SQLite audit_logs table.
    pub fn log(
        &self,
        actor: &str,
        action: &str,
        target: &str,
        approved: bool,
        reason: Option<&str>,
    ) -> Result<()> {
        let entry = AuditLogEntry {
            id: None,
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            approved,
            reason: reason.map(|s| s.to_string()),
            created_at: Utc::now().to_rfc3339(),
        };

        let approved_int = if approved { 1 } else { 0 };

        // 1. Write to SQLite audit_logs table via shared connection or db path
        if let Some(shared) = &self.shared_conn {
            if let Ok(conn) = shared.lock() {
                let _ = conn.execute(
                    "INSERT INTO audit_logs (actor, action, target, approved, reason, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        entry.actor,
                        entry.action,
                        entry.target,
                        approved_int,
                        entry.reason,
                        entry.created_at,
                    ],
                );
            }
        } else if let Some(p) = &self.db_path {
            if p.exists() {
                if let Ok(conn) = rusqlite::Connection::open(p) {
                    let _ = conn.execute(
                        "INSERT INTO audit_logs (actor, action, target, approved, reason, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            entry.actor,
                            entry.action,
                            entry.target,
                            approved_int,
                            entry.reason,
                            entry.created_at,
                        ],
                    );
                }
            }
        }

        // 2. Append to .agent/logs/audit.log
        if let Some(path) = &self.log_file_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                if let Ok(json_line) = serde_json::to_string(&entry) {
                    let _ = writeln!(file, "{}", json_line);
                }
            }
        }

        Ok(())
    }

    /// Retrieves recent audit log entries from SQLite database.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<AuditLogEntry>> {
        let query_fn = |conn: &rusqlite::Connection| -> Result<Vec<AuditLogEntry>> {
            let mut stmt = conn.prepare(
                "SELECT id, actor, action, target, approved, reason, created_at
                 FROM audit_logs ORDER BY id DESC LIMIT ?1",
            ).map_err(|e| KaiokenError::MemoryError(format!("Failed to prepare audit_logs query: {}", e)))?;

            let rows = stmt.query_map(params![limit as i64], |row| {
                let approved_int: i32 = row.get(4)?;
                Ok(AuditLogEntry {
                    id: Some(row.get(0)?),
                    actor: row.get(1)?,
                    action: row.get(2)?,
                    target: row.get(3)?,
                    approved: approved_int == 1,
                    reason: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).map_err(|e| KaiokenError::MemoryError(format!("Failed to query audit_logs: {}", e)))?;

            let mut results = Vec::new();
            for r in rows {
                if let Ok(entry) = r {
                    results.push(entry);
                }
            }
            Ok(results)
        };

        if let Some(shared) = &self.shared_conn {
            let conn = shared.lock().unwrap();
            query_fn(&conn)
        } else if let Some(p) = &self.db_path {
            if p.exists() {
                let conn = rusqlite::Connection::open(p)
                    .map_err(|e| KaiokenError::MemoryError(format!("Failed to open DB: {}", e)))?;
                query_fn(&conn)
            } else {
                Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }
}

use std::path::Path;
use rusqlite::Connection;
use crate::error::{KaiokenError, Result};

pub const SCHEMA_V1: &str = r#"
-- 1. Files
CREATE TABLE IF NOT EXISTS files (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  path TEXT UNIQUE NOT NULL,
  language TEXT,
  size INTEGER NOT NULL,
  source_hash TEXT,
  parse_status TEXT DEFAULT 'ok',
  last_commit TEXT,
  last_modified TEXT,
  summary TEXT,
  risk_level TEXT,
  doc_path TEXT,
  doc_status TEXT
);

-- 2. Symbols
CREATE TABLE IF NOT EXISTS symbols (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  line_start INTEGER NOT NULL,
  line_end INTEGER NOT NULL,
  visibility TEXT,
  signature TEXT,
  summary TEXT,
  parent TEXT,
  FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- 3. Raw Imports
CREATE TABLE IF NOT EXISTS import_raw (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file_id INTEGER NOT NULL,
  raw_import TEXT NOT NULL,
  resolved_file_id INTEGER,
  resolved_symbol_id INTEGER,
  status TEXT DEFAULT 'unresolved',
  FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- 4. Edges (Dependency Graph)
CREATE TABLE IF NOT EXISTS edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_symbol_id INTEGER,
  target_symbol_id INTEGER,
  edge_type TEXT NOT NULL
);

-- 5. Commits
CREATE TABLE IF NOT EXISTS commits (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hash TEXT UNIQUE NOT NULL,
  message TEXT NOT NULL,
  author TEXT,
  created_at TEXT,
  summary TEXT,
  risk_level TEXT
);

-- 6. Commit Changes
CREATE TABLE IF NOT EXISTS commit_changes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  commit_id INTEGER NOT NULL,
  file_id INTEGER,
  symbol_id INTEGER,
  change_type TEXT NOT NULL,
  FOREIGN KEY(commit_id) REFERENCES commits(id) ON DELETE CASCADE
);

-- 7. Documentation
CREATE TABLE IF NOT EXISTS docs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file_id INTEGER NOT NULL,
  doc_path TEXT NOT NULL,
  doc_level INTEGER DEFAULT 1,
  source_hash TEXT,
  status TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  stale_reason TEXT,
  FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- 8. Failures
CREATE TABLE IF NOT EXISTS failures (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  error_type TEXT NOT NULL,
  message TEXT NOT NULL,
  file_id INTEGER,
  symbol_id INTEGER,
  fix_summary TEXT,
  status TEXT DEFAULT 'unresolved',
  resolved_at TEXT,
  created_at TEXT NOT NULL
);

-- 9. Architectural Decisions
CREATE TABLE IF NOT EXISTS decisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL
);

-- 10. Agent Runs
CREATE TABLE IF NOT EXISTS agent_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task TEXT NOT NULL,
  status TEXT NOT NULL,
  model TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  total_tokens INTEGER DEFAULT 0,
  estimated_cost REAL DEFAULT 0.0,
  context_packet_id TEXT,
  error TEXT
);

-- 11. Tool Calls
CREATE TABLE IF NOT EXISTS tool_calls (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  agent_run_id INTEGER NOT NULL,
  tool_name TEXT NOT NULL,
  input_json TEXT NOT NULL,
  output_summary TEXT,
  status TEXT NOT NULL,
  duration_ms INTEGER DEFAULT 0,
  error TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
);

-- 12. Audit Logs
CREATE TABLE IF NOT EXISTS audit_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  target TEXT NOT NULL,
  approved INTEGER NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL
);

-- 13. Retrieval Cache
CREATE TABLE IF NOT EXISTS retrieval_cache (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  query_hash TEXT UNIQUE NOT NULL,
  context_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_language ON files(language);
CREATE INDEX IF NOT EXISTS idx_symbols_file_id ON symbols(file_id);
CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
CREATE INDEX IF NOT EXISTS idx_import_raw_file_id ON import_raw(file_id);
CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_symbol_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_symbol_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(edge_type);
CREATE INDEX IF NOT EXISTS idx_commits_hash ON commits(hash);
CREATE INDEX IF NOT EXISTS idx_commit_changes_commit ON commit_changes(commit_id);
CREATE INDEX IF NOT EXISTS idx_commit_changes_file ON commit_changes(file_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_file_id ON docs(file_id);
CREATE INDEX IF NOT EXISTS idx_failures_file_id ON failures(file_id);
CREATE INDEX IF NOT EXISTS idx_failures_symbol_id ON failures(symbol_id);
CREATE INDEX IF NOT EXISTS idx_retrieval_cache_hash ON retrieval_cache(query_hash);
CREATE INDEX IF NOT EXISTS idx_tool_calls_run ON tool_calls(agent_run_id);
"#;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to open SQLite database at {}: {}", path.display(), e)))?;
        
        Self::apply_pragmas(&conn)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to open in-memory SQLite database: {}", e)))?;
        Self::apply_pragmas(&conn)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn apply_pragmas(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| KaiokenError::MemoryError(format!("Failed to configure SQLite pragmas: {}", e)))?;
        Ok(())
    }

    pub fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA_V1)
            .map_err(|e| KaiokenError::MemoryError(format!("Failed to run schema migrations: {}", e)))?;

        // Ensure unique index on docs(file_id) for ON CONFLICT upsert support
        // (Upgrades legacy non-unique index from older database instances)
        let is_unique: bool = self.conn.query_row(
            "SELECT \"unique\" FROM pragma_index_list('docs') WHERE name = 'idx_docs_file_id'",
            [],
            |r| r.get::<_, i32>(0).map(|u| u == 1),
        ).unwrap_or(false);

        if !is_unique {
            let _ = self.conn.execute("DROP INDEX IF EXISTS idx_docs_file_id;", []);
            self.conn.execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_file_id ON docs(file_id);",
                [],
            ).map_err(|e| KaiokenError::MemoryError(format!("Failed to create unique index on docs(file_id): {}", e)))?;
        }

        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

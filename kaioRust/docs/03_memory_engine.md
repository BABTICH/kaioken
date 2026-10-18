# 03. Memory Engine Layer (Layer 2)

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 3 (Layer 2), Section 4 & Section 15

---

## 1. Purpose & Scope

The **Memory Engine** is the persistent knowledge repository of KAIOKEN Agent. Rather than relying on ephemeral LLM chat history or vector embeddings alone, it stores **7 structured relational memory types** backed by an embedded SQLite database (`.agent/memory.sqlite`).

---

## 2. SQLite Concurrency & Reliability Safety

To guarantee safe concurrent access across the CLI, watch daemons, and background scans:
- **WAL Mode**: `PRAGMA journal_mode=WAL;` allows concurrent readers without blocking writes.
- **Foreign Keys**: `PRAGMA foreign_keys=ON;` ensures referential integrity across symbol and file cascades.
- **Busy Timeout**: `PRAGMA busy_timeout=5000;` prevents database lock errors during write bursts.
- **Single Writer Channel**: Application logic funnels write transactions through a dedicated channel/mutex to avoid lock contention.
- **Secret Redaction**: Sensitive credentials, API tokens, and private keys are stripped before persisting logs, failures, or tool outputs into SQLite.

---

## 3. Seven Core Memory Types

1. **File Memory (`files`)**: File paths, languages, sizes, cryptographic `source_hash`, `parse_status`, summaries, and risk ratings.
2. **Symbol Memory (`symbols`)**: AST functions, structs, classes, visibility, line spans, signatures, and doc summaries.
3. **Dependency Graph Memory (`edges`)**: Directed relationships (`calls`, `imports`, `implements`, `covers_test`).
4. **Change Memory (`commits` & `commit_changes`)**: Commits, touched files/symbols, impact assessments, and recommended tests.
5. **Failure Memory (`failures`)**: Compiler errors, test breakages, error signatures, root causes, and verified fixes.
6. **Decision Memory (`decisions`)**: Architectural Decision Records (ADRs), rationales, alternatives, and consequences.
7. **Documentation Memory (`docs`)**: Path mirrors in `.agent-docs/`, doc levels (0-3), source hashes, and freshness state.

---

## 4. SQLite Relational Schema

```sql
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
  FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- 3. Edges (Dependency Graph)
CREATE TABLE IF NOT EXISTS edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_symbol_id INTEGER,
  target_symbol_id INTEGER,
  edge_type TEXT NOT NULL
);

-- 4. Commits
CREATE TABLE IF NOT EXISTS commits (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  hash TEXT UNIQUE NOT NULL,
  message TEXT NOT NULL,
  author TEXT,
  created_at TEXT,
  summary TEXT,
  risk_level TEXT
);

-- 5. Commit Changes
CREATE TABLE IF NOT EXISTS commit_changes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  commit_id INTEGER NOT NULL,
  file_id INTEGER,
  symbol_id INTEGER,
  change_type TEXT NOT NULL,
  FOREIGN KEY(commit_id) REFERENCES commits(id) ON DELETE CASCADE
);

-- 6. Documentation Freshness
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

-- 7. Failures
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

-- 8. Architectural Decisions
CREATE TABLE IF NOT EXISTS decisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL
);

-- 9. Agent Runs (Execution Observability)
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

-- 10. Tool Calls (Fine-grained Tool Audit)
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

-- 11. Audit Logs (Security & Approval Audit Trail)
CREATE TABLE IF NOT EXISTS audit_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  target TEXT NOT NULL,
  approved INTEGER NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL
);

-- 12. Retrieval Cache (Context Acceleration)
CREATE TABLE IF NOT EXISTS retrieval_cache (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  query_hash TEXT UNIQUE NOT NULL,
  context_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL
);

-- 13. Raw Imports (Unresolved Dependency Buffer)
CREATE TABLE IF NOT EXISTS import_raw (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file_id INTEGER NOT NULL,
  raw_import TEXT NOT NULL,
  resolved_file_id INTEGER,
  resolved_symbol_id INTEGER,
  status TEXT DEFAULT 'unresolved',
  FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE
);
```

---

## 5. Performance Indexes

The following indexes are created to ensure sub-millisecond retrieval performance:

```sql
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_language ON files(language);
CREATE INDEX IF NOT EXISTS idx_symbols_file_id ON symbols(file_id);
CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_symbol_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_symbol_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(edge_type);
CREATE INDEX IF NOT EXISTS idx_commits_hash ON commits(hash);
CREATE INDEX IF NOT EXISTS idx_commit_changes_commit ON commit_changes(commit_id);
CREATE INDEX IF NOT EXISTS idx_commit_changes_file ON commit_changes(file_id);
CREATE INDEX IF NOT EXISTS idx_docs_file_id ON docs(file_id);
CREATE INDEX IF NOT EXISTS idx_failures_file_id ON failures(file_id);
CREATE INDEX IF NOT EXISTS idx_failures_symbol_id ON failures(symbol_id);
CREATE INDEX IF NOT EXISTS idx_retrieval_cache_hash ON retrieval_cache(query_hash);
CREATE INDEX IF NOT EXISTS idx_tool_calls_run ON tool_calls(agent_run_id);
```

---

## 6. Current Implementation Status

- [x] Defined Rust models in `src/memory/`.
- [x] Defined schema migration script in `src/memory/db.rs`.
- [x] Phase 5: SQLite WAL mode, PRAGMAs, and index initialization (`src/memory/db.rs`).
- [x] Phase 5: CRUD persistence for `files`, `symbols`, `import_raw`, and `edges`.
- [ ] Phase 6: Commit understanding persistence.
- [ ] Phase 10: `agent_runs`, `tool_calls`, and `audit_logs` persistence.

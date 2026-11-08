# 14. `kaioken memory explain` & `recent` — Memory Introspection Commands

> **Status**: 🟢 Implemented (Phase 19)
> **CLI Entries**: `kaioken memory explain <path>`, `kaioken memory recent`
> **Source Modules**: `src/cli/memory.rs`, `src/memory/introspection.rs`, `src/memory/mod.rs`
> **Tests**: `tests/phase19_memory_introspection_tests.rs` (8 unit & integration tests)

---

## 1. Purpose & Motivation

These two commands expose the agent's **internal understanding** of the codebase to the developer:

- **`memory explain <path>`**: "What does the agent know about this file?" — Shows the file's AST role, extracted symbols, dependency edges, documentation status, and recent commit risk.
- **`memory recent`**: "What has the agent learned from recent commits?" — Displays a timeline of commit memories with risk levels, impacted symbols, and summaries.

These introspection tools build **developer trust** by making the agent's knowledge transparent and verifiable.

---

## 2. `kaioken memory explain <path>`

### 2.1 What It Shows

Given a file path, `explain` queries SQLite memory and assembles a comprehensive profile:

| Section | Data Source | Description |
|---------|-------------|-------------|
| **File Metadata** | `files` table | Path, language, size, hash, parse status, last scanned |
| **Symbols** | `symbols` table | Functions, structs, enums, traits, methods with visibility |
| **Outgoing Edges** | `edges` table (source) | What this file depends on (imports, calls) |
| **Incoming Edges** | `edges` table (target) | What depends on this file (callers, importers) |
| **Raw Imports** | `import_raw` table | Unresolved import/use statements |
| **Documentation** | `docs` table / `.agent-docs/` | Doc level, status, staleness |
| **Commit History** | `commits` table | Recent commits touching this file, risk levels |
| **Failure Memory** | `failures` table | Past errors associated with this file |

### 2.2 Terminal Output

```text
📖 Memory Explain — src/scanner/tree_sitter.rs
════════════════════════════════════════════════════════════

  📄 File Metadata
  ────────────────
  Language:      Rust
  Size:          13,792 bytes (391 lines)
  SHA-256:       a1b2c3d4...
  Parse Status:  ok
  Last Scanned:  2026-10-18 15:30:00Z

  🔣 Symbols (10 extracted)
  ────────────────────────────
  ├── enum   SymbolKind          (pub)   L9-L22
  ├── impl   SymbolKind          (pub)   L24-L39
  │   └── fn as_str              (pub)   L25-L38
  ├── struct ExtractedSymbol     (pub)   L42-L60
  ├── struct AstExtractResult    (pub)   L62-L70
  ├── fn     extract_symbols     (pub)   L72-L150
  ├── fn     walk_rust_ast       (priv)  L152-L280
  ├── fn     get_node_text       (priv)  L282-L300
  └── fn     detect_visibility   (priv)  L302-L330

  🔗 Dependencies (Outgoing Edges: 4)
  ─────────────────────────────────────
  → src/error.rs           (imports: KaiokenError, Result)
  → src/scanner/languages.rs  (imports: Language)
  → src/scanner/file_inventory.rs  (imports: ParseStatus)
  → tree-sitter (external crate)

  📥 Dependents (Incoming Edges: 3)
  ──────────────────────────────────
  ← src/memory/mod.rs      (calls: extract_symbols)
  ← src/cli/memory.rs      (calls: extract_symbols)
  ← tests/scanner_tests.rs (tests: extract_symbols)

  📝 Documentation
  ─────────────────
  Doc Path:   .agent-docs/src/scanner/tree_sitter.md
  Doc Level:  Level 0 (Facts)
  Status:     current
  Staleness:  ✅ Up to date

  📊 Recent Commits (3)
  ──────────────────────
  • abc1234 (2026-10-18) — "Add TypeAlias and Module symbol kinds"  [Risk: LOW]
  • def5678 (2026-10-17) — "Fix partial parse recovery"             [Risk: MEDIUM]
  • ghi9012 (2026-10-15) — "Initial tree-sitter integration"        [Risk: LOW]

  ⚠️ Failure Memory (1)
  ──────────────────────
  • [RESOLVED] "E0308: mismatched types in detect_visibility" — Fixed in def5678
```

### 2.3 JSON Output (`--json`)

```json
{
  "file": {
    "path": "src/scanner/tree_sitter.rs",
    "language": "rust",
    "size_bytes": 13792,
    "line_count": 391,
    "file_hash": "a1b2c3d4...",
    "parse_status": "ok",
    "last_scanned": "2026-10-18T15:30:00Z"
  },
  "symbols": [...],
  "outgoing_edges": [...],
  "incoming_edges": [...],
  "documentation": { "level": 0, "status": "current" },
  "recent_commits": [...],
  "failures": [...]
}
```

---

## 3. `kaioken memory recent`

### 3.1 What It Shows

Displays the last N commit memories, ordered by recency, with risk assessment and impacted file/symbol summaries.

### 3.2 CLI Args

```rust
#[derive(clap::Args)]
pub struct MemoryRecentArgs {
    /// Number of recent commits to show
    #[arg(long, default_value = "10")]
    pub last: usize,
    /// Filter by minimum risk level (low, medium, high, critical)
    #[arg(long)]
    pub min_risk: Option<String>,
    /// Show only commits touching a specific file
    #[arg(long)]
    pub file: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
```

### 3.3 Terminal Output

```text
🕐 Recent Commit Memories — Last 5
════════════════════════════════════════════════════════════════

  ┌─ abc1234  2026-10-18 15:30:00  [Risk: LOW]
  │  Author:  developer@example.com
  │  Message: "Add TypeAlias and Module symbol kinds"
  │  Files:   2 changed (src/scanner/tree_sitter.rs, src/scanner/languages.rs)
  │  Symbols: +2 added (SymbolKind::TypeAlias, SymbolKind::Module)
  │  Summary: Extended the SymbolKind enum with two new variants for
  │           type aliases and module declarations.
  └──────────────────────────────────────────────────────────

  ┌─ def5678  2026-10-17 12:15:00  [Risk: MEDIUM ⚠️]
  │  Author:  developer@example.com
  │  Message: "Fix partial parse recovery"
  │  Files:   1 changed (src/scanner/tree_sitter.rs)
  │  Symbols: ~1 modified (detect_visibility)
  │  Summary: Fixed type mismatch in visibility detection for
  │           partially-parsed syntax trees. Resolved E0308.
  │  Fixes:   Resolved failure #12 (E0308 mismatched types)
  └──────────────────────────────────────────────────────────

  ... (3 more entries)

═══════════════════════════════════════════════════════════════
  Showing 5 of 127 total commit memories
```

---

## 4. Architecture & Integration Points

### 4.1 Module Layout

```text
src/
├── cli/
│   └── memory.rs   ← EXTEND: Add `Explain` and `Recent` subcommands
├── memory/
│   ├── mod.rs      ← EXTEND: Add query methods for explain/recent data assembly
│   └── commit_memory.rs ← EXISTING: Commit memory storage (reuse)
```

### 4.2 Data Assembly for `explain`

```rust
pub struct FileExplanation {
    pub file: FileRecord,
    pub symbols: Vec<SymbolRecord>,
    pub outgoing_edges: Vec<EdgeRecord>,
    pub incoming_edges: Vec<EdgeRecord>,
    pub raw_imports: Vec<ImportRawRecord>,
    pub documentation: Option<DocRecord>,
    pub recent_commits: Vec<CommitRecord>,
    pub failures: Vec<FailureRecord>,
}

impl MemoryEngine {
    pub fn explain_file(&self, path: &str) -> Result<FileExplanation> {
        let file = self.get_file_by_path(path)?;
        let file_id = file.id.ok_or_else(|| anyhow!("File not in memory"))?;

        Ok(FileExplanation {
            file,
            symbols: self.get_symbols_for_file(file_id)?,
            outgoing_edges: self.get_edges_from_file(file_id)?,
            incoming_edges: self.get_edges_to_file(file_id)?,
            raw_imports: self.get_imports_for_file(file_id)?,
            documentation: self.get_doc_for_path(path).ok(),
            recent_commits: self.get_commits_touching_file(path, 10)?,
            failures: self.get_failures_for_file(path)?,
        })
    }
}
```

### 4.3 Required New Query Methods

| Method | Table | Description |
|--------|-------|-------------|
| `get_edges_from_file(file_id)` | `edges` | Outgoing edges where `source_file_id = file_id` |
| `get_edges_to_file(file_id)` | `edges` | Incoming edges where `target_file_id = file_id` |
| `get_commits_touching_file(path, limit)` | `commits` | Commits where `changed_files` JSON contains path |
| `get_failures_for_file(path)` | `failures` | Failures where `file_path = path` |
| `get_recent_commits(limit, min_risk)` | `commits` | Recent commits ordered by timestamp |

### 4.4 Dependencies

- **Existing**: `MemoryEngine`, `rusqlite`, `chrono`, `serde_json`
- **No new crate dependencies**

---

## 5. Behavioral Rules

1. **Read-only**: These commands never modify the database or filesystem.
2. **Graceful missing data**: If a file isn't in memory, print "File not yet scanned — run `kaioken memory scan` first."
3. **Relative paths**: Accept both relative and absolute paths; normalize to workspace-relative before querying.
4. **Colorized output**: Use ANSI colors for risk levels (green=LOW, yellow=MEDIUM, red=HIGH/CRITICAL).

---

## 6. Testing Strategy

| Test | Description |
|------|-------------|
| `test_explain_full_file` | Insert file with symbols, edges, docs → verify complete output |
| `test_explain_missing_file` | Query non-existent file → graceful error message |
| `test_explain_no_edges` | File with symbols but no edges → partial output works |
| `test_recent_default` | Insert commits → verify last 10 returned |
| `test_recent_min_risk` | Filter by risk → only matching commits shown |
| `test_recent_file_filter` | Filter by file → only commits touching that file |
| `test_explain_json` | Verify JSON schema matches FileExplanation struct |

---

## 7. Open Questions

- [ ] Should `explain` show a visual **dependency graph** (ASCII art or mermaid)?
- [ ] Should `explain` support explaining a **symbol** directly (e.g., `kaioken memory explain src/main.rs::main`)?
- [ ] Should `recent` support a `--diff` flag to show actual code changes alongside the summary?

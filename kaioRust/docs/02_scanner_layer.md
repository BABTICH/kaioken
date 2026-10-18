# 02. Codebase Scanner Layer (Layer 1)

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 3 (Layer 1) & Section 4

---

## 1. Purpose & Scope

The **Codebase Scanner Layer** extracts deterministic structural metadata from the repository without relying on LLM calls. It combines filesystem traversal, git change detection, cryptographic content hashing, and Tree-sitter AST parsing.

**Core Scanner Principle**:
> The scanner must be 100% deterministic, local, and free of LLM calls. It handles broken syntax, huge files, binary blobs, and unsupported languages gracefully without panicking.

```text
Filesystem / Git Diff
        │
        ▼
┌───────────────────────────┐
│     File Inventory        │ -> Detect files, sizes, extensions, ignore patterns
└───────────────────────────┘
        │
        ▼
┌───────────────────────────┐
│ Binary & Size Pre-filters │ -> Skip >1MB, detect binary blobs, verify UTF-8
└───────────────────────────┘
        │
        ▼
┌───────────────────────────┐
│ Content Hash & Language   │ -> Compute SHA-256 file_hash, detect language
└───────────────────────────┘
        │
        ▼
┌───────────────────────────┐
│   Tree-sitter AST Parser  │ -> Extract symbols, imports_raw, exports, error recovery
└───────────────────────────┘
        │
        ▼
Structured JSON / Rust Structs -> Fed to Memory Engine (Layer 2)
```

---

## 2. Robustness & Edge-Case Handling

The scanner enforces production safeguards before parsing:

### 1. Cryptographic File Hashing (`file_hash`)
- Uses `SHA-256` content hashing.
- Allows instant detection of whether file contents changed since the previous scan without re-running AST queries.

### 2. Parse Status States (`parse_status`)
Every discovered file is assigned an explicit parse status:
- `ok`: File parsed cleanly with Tree-sitter; all top-level symbols extracted.
- `partial`: File contained syntax errors, but Tree-sitter error recovery extracted surviving valid symbols.
- `failed`: Unrecoverable parse failure or invalid text encoding.
- `binary`: Binary asset (images, binaries, audio, archives); skipped by AST.
- `ignored`: Matches `.gitignore` or `.agent/config.toml` ignore rules.
- `too_large`: File exceeds maximum size threshold (skipped by AST).

### 3. File Size Boundaries
- **Default Maximum Size**: `1 MB` (configurable via `scanner.max_file_size_bytes`).
- Files larger than this threshold are indexed in `files` with status `too_large`, but bypass Tree-sitter parsing to protect against memory exhaustion.

### 4. Binary & Encoding Detection
- Pre-scans the first 8 KB for null bytes (`0x00`) to detect binary files.
- Strictly parses text as UTF-8. Non-UTF-8 files are marked `failed` or `binary` rather than crashing the scanner.

### 5. Unsupported Languages
- Files with unsupported extensions are inventoried in the `files` table for search and glob discovery, but AST extraction is cleanly skipped.

### 6. Raw Import Tracking (`imports_raw`)
- Because Tree-sitter cannot always resolve relative modules or external crate namespaces without a full compiler, raw import statements are extracted into `imports_raw` for best-effort resolution into `import_raw` database records.

---

## 3. Supported Languages Order

1. **Rust (`.rs`)**: Tree-sitter Rust grammar (functions, structs, traits, enums, impl blocks, visibility).
2. **TypeScript / JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`)**: Classes, functions, exports, imports, interfaces.
3. **Python (`.py`)**: Classes, functions, imports, decorators.

---

## 4. Data Output Contract

The scanner outputs structured JSON conforming to:

```json
{
  "path": "src/auth/session.rs",
  "language": "rust",
  "size": 4200,
  "file_hash": "a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890",
  "parse_status": "ok",
  "symbols": [
    {
      "name": "validate_session",
      "kind": "function",
      "line_start": 45,
      "line_end": 82,
      "visibility": "public",
      "signature": "pub async fn validate_session(token: &str) -> Result<User, AuthError>"
    }
  ],
  "imports": [
    "jsonwebtoken",
    "sqlx",
    "crate::models::User"
  ],
  "imports_raw": [
    "use jsonwebtoken::*;",
    "use sqlx::PgPool;",
    "use crate::models::User;"
  ],
  "exports": [
    "validate_session",
    "create_session"
  ],
  "errors": [],
  "last_commit": "abc1234",
  "last_modified": "2026-06-14T10:15:30Z"
}
```

---

## 5. Current Implementation Status

- [x] Defined scanner interfaces in `src/scanner/mod.rs`.
- [x] Defined `file_inventory.rs`, `languages.rs`, `git.rs`, `diff.rs`, `tree_sitter.rs` module stubs.
- [x] Phase 2: Git diff extraction and commit inspector (`src/scanner/git.rs`).
- [x] Phase 3: Filesystem inventory walk with SHA-256 `file_hash`, binary checks, and size thresholds.
- [x] Phase 4: Tree-sitter symbol, `imports_raw`, and `exports` extraction with error recovery.

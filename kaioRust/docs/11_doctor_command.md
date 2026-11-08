# 11. `kaioken doctor` — Environment Diagnostic Runner

> **Status**: 🟢 Implemented (Phase 16)
> **CLI Entry**: `kaioken doctor`
> **Source Modules**: `src/cli/doctor.rs`, `src/doctor/mod.rs`, `src/doctor/checks.rs`

---

## 1. Purpose & Motivation

`kaioken doctor` is a **single-command environment health check** that validates every prerequisite the agent runtime depends on. It provides a clear pass/fail diagnostic report so developers can immediately resolve issues before running `kaioken task`, `scan`, or `docs` commands.

Without `doctor`, users waste time debugging obscure failures caused by missing tools, invalid API keys, or corrupt databases — problems that could be caught upfront in seconds.

---

## 2. Diagnostic Checks Catalog

The command performs the following ordered checks, each producing a `✅ PASS`, `⚠️ WARN`, or `❌ FAIL` result:

| #  | Check Name              | What It Validates                                           | Severity |
|----|-------------------------|-------------------------------------------------------------|----------|
| 1  | **Git Availability**    | `git --version` exits 0, version ≥ 2.25                    | FAIL     |
| 2  | **Git Repository**      | Current directory is inside a Git working tree              | FAIL     |
| 3  | **Rust Toolchain**      | `rustc --version` and `cargo --version` exit 0              | WARN     |
| 4  | **SQLite Runtime**      | `MemoryEngine::open()` can initialize or open the database  | FAIL     |
| 5  | **SQLite Schema**       | All expected tables exist with correct columns              | FAIL     |
| 6  | **Tree-sitter Rust**    | Can parse a trivial `fn main() {}` snippet                  | FAIL     |
| 7  | **Tree-sitter TS/JS**   | Can parse trivial TS/JS snippets (when multi-lang enabled)  | WARN     |
| 8  | **Tree-sitter Python**  | Can parse trivial Python snippet (when multi-lang enabled)  | WARN     |
| 9  | **Config File**         | `.agent/config.toml` exists and deserializes without error   | WARN     |
| 10 | **API Key (OpenRouter)**| `OPENROUTER_API_KEY` env var is set and non-empty           | WARN     |
| 11 | **Docs Directory**      | `.agent-docs/` directory exists or is creatable             | WARN     |
| 12 | **Lockfile Safety**     | No stale `.agent/scan.lock` present                         | WARN     |
| 13 | **Disk Space**          | Workspace partition has > 100 MB free                       | WARN     |

---

## 3. Output Format

### Terminal Output (Default)

```text
🔍 kaioken doctor — Environment Diagnostic Report
══════════════════════════════════════════════════

  ✅  Git Availability       git 2.44.0
  ✅  Git Repository         d:/project/ai_now_know/kaioRust
  ⚠️  Rust Toolchain         rustc 1.81.0 (not required for all operations)
  ✅  SQLite Runtime          .agent/memory.sqlite (WAL, 3.45.0)
  ✅  SQLite Schema           8/8 tables present
  ✅  Tree-sitter Rust        Parser operational
  ⚠️  Tree-sitter TS/JS      Not yet installed (multi-lang feature)
  ⚠️  Tree-sitter Python     Not yet installed (multi-lang feature)
  ✅  Config File             .agent/config.toml valid
  ⚠️  API Key (OpenRouter)   Not set — agent tasks will be unavailable
  ✅  Docs Directory          .agent-docs/ exists
  ✅  Lockfile Safety         No stale lock
  ✅  Disk Space              42.7 GB free

══════════════════════════════════════════════════
  Result: 9 passed · 4 warnings · 0 failed
  Status: READY (non-agent operations available)
```

### JSON Output (`--json`)

```json
{
  "checks": [
    { "name": "git_availability", "status": "pass", "detail": "git 2.44.0" },
    { "name": "api_key_openrouter", "status": "warn", "detail": "Not set" }
  ],
  "summary": { "pass": 9, "warn": 4, "fail": 0 },
  "overall": "ready_limited"
}
```

---

## 4. Architecture & Integration Points

### 4.1 Module Layout

```text
src/
├── cli/
│   ├── mod.rs        ← Add `Doctor` variant to `Commands` enum
│   └── doctor.rs     ← NEW: Check runner, output formatter
├── doctor/
│   ├── mod.rs        ← NEW: `DoctorEngine` orchestrator
│   └── checks.rs     ← NEW: Individual check implementations
```

### 4.2 Data Structures

```rust
/// Result of a single diagnostic check
pub struct DiagnosticCheck {
    pub name: &'static str,
    pub display_name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
}

pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

/// Complete diagnostic report
pub struct DiagnosticReport {
    pub checks: Vec<DiagnosticCheck>,
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
    pub overall: OverallStatus,
}

pub enum OverallStatus {
    Ready,          // All pass
    ReadyLimited,   // Some warnings, no failures
    NotReady,       // At least one failure
}
```

### 4.3 Dependencies

- **Existing**: `std::process::Command` (for `git --version`, `rustc --version`), `MemoryEngine`, `Config`
- **No new crate dependencies required**

### 4.4 CLI Integration

```rust
// In src/cli/mod.rs — Commands enum
/// Run environment diagnostic checks
Doctor(DoctorArgs),

// In src/cli/doctor.rs
#[derive(clap::Args)]
pub struct DoctorArgs {
    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
    /// Run only specific checks (comma-separated)
    #[arg(long)]
    pub only: Option<String>,
}
```

---

## 5. Behavioral Rules

1. **Non-destructive**: `doctor` never modifies any files, databases, or configuration.
2. **Fast**: All checks should complete in < 2 seconds total.
3. **Offline-capable**: Network checks (API key validation) only verify env var presence, never make HTTP calls.
4. **Exit codes**: Exit 0 if all pass/warn, exit 1 if any fail.
5. **Extensible**: New checks can be added by implementing a `Check` trait and registering in the runner.

---

## 6. Testing Strategy

| Test | Description |
|------|-------------|
| `test_doctor_all_pass` | Mock all dependencies present → expect 0 failures |
| `test_doctor_missing_git` | Remove `git` from PATH → expect FAIL on git checks |
| `test_doctor_no_config` | Run without `.agent/config.toml` → expect WARN |
| `test_doctor_json_output` | Verify JSON output schema matches spec |
| `test_doctor_stale_lock` | Create stale lockfile → expect WARN |

---

## 7. Open Questions

- [ ] Should `doctor` attempt to **auto-fix** simple issues (e.g., create missing `.agent/` dir)?
- [ ] Should we add a `--fix` flag for auto-remediation?
- [ ] Should network connectivity to OpenRouter be checked (adds latency)?

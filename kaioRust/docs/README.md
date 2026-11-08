# KAIOKEN Agent Documentation System

[![CI](https://github.com/BABTICH/kaioken/actions/workflows/ci.yml/badge.svg)](https://github.com/BABTICH/kaioken/actions/workflows/ci.yml)
[![Security Audit](https://github.com/BABTICH/kaioken/actions/workflows/ci.yml/badge.svg?event=schedule)](https://github.com/BABTICH/kaioken/actions/workflows/ci.yml)

> **Vision**: A Rust-based AI coding agent that continuously understands the repository before being asked to write code, powered by persistent structured memory, deterministic AST understanding, and automatic documentation mirroring.

Reference Blueprint: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md)

---

## 1. Architectural Layers & Documentation Index

| Doc File | System Layer / Topic | Blueprint Sections | Status |
|---|---|---|---|
| [00_vision_and_principles.md](00_vision_and_principles.md) | Vision & Core Principles (Rules 1-12) | §§ 1, 19, 20, 27 | 🟢 Done |
| [01_trigger_layer.md](01_trigger_layer.md) | Layer 0: Trigger Layer | § 3 (Layer 0), § 11 | 🟢 Done |
| [02_scanner_layer.md](02_scanner_layer.md) | Layer 1: Codebase Scanner & AST | § 3 (Layer 1), § 4 | 🟢 Done |
| [03_memory_engine.md](03_memory_engine.md) | Layer 2: Memory Engine & SQLite | §§ 3, 4, 15 | 🟢 Done |
| [04_documentation_engine.md](04_documentation_engine.md) | Layer 3: Documentation Mirror Engine | §§ 5, 6 | 🟢 Done |
| [05_retrieval_layer.md](05_retrieval_layer.md) | Layer 4: Context Retrieval Layer | § 7, § 14 | 🟢 Done |
| [06_agent_runtime.md](06_agent_runtime.md) | Layer 5: Agent Runtime & Critic | §§ 8, 9, 10 | 🟢 Done |
| [07_cli_and_config.md](07_cli_and_config.md) | CLI & Configuration (.agent/config.toml) | §§ 16, 18 | 🟢 Done |
| [08_phases_roadmap.md](08_phases_roadmap.md) | 15-Phase Step-by-Step Roadmap | §§ 17, 23, 24, 25 | 🟢 Done |
| [09_safety_and_evaluation.md](09_safety_and_evaluation.md) | Safety, Permissions & Evaluation | §§ 14, 21, 22 | 🟢 Done |
| [10_implementation_notes.md](10_implementation_notes.md) | Production Implementation Notes & Crates | § 13, § 14 | 🟢 Done |
| [11_doctor_command.md](11_doctor_command.md) | `kaioken doctor` — Environment Diagnostics | Future | 🟢 Done |
| [12_hook_install.md](12_hook_install.md) | `kaioken hook install` — Git Hook Installer | Future | 🟢 Done |
| [13_audit_show.md](13_audit_show.md) | `kaioken audit show` — Audit Log Inspector | Future | 🟢 Done |
| [14_memory_explain_recent.md](14_memory_explain_recent.md) | `memory explain` & `recent` — Introspection | Future | 🟢 Done |
| [15_review_command.md](15_review_command.md) | `kaioken review` — Diff Analyzer | Future | 🟢 Done |
| [16_watch_daemon.md](16_watch_daemon.md) | `kaioken watch` — File Watcher Daemon | Future | 🟢 Done |
| [17_multi_language_parsers.md](17_multi_language_parsers.md) | Multi-Language AST Parsers (TS/JS/Python) | Future | 🟢 Done |
| [18_github_actions_ci.md](18_github_actions_ci.md) | GitHub Actions CI/CD Pipeline | Future | 🟢 Done (Phase 23) |
| [19_gpui_desktop_app.md](19_gpui_desktop_app.md) | Native Desktop Cockpit (`wry` + `tao`, historical filename) | Future (P3) | 🟢 Done (Phase 24) |
| [20_interactive_tui.md](20_interactive_tui.md) | Interactive Terminal UI (`ratatui` fallback) | Future (P4) | 🟢 Done (Phase 25) |
| [21_future_roadmap.md](21_future_roadmap.md) | Future Features — Priority & Roadmap | Future | 🔵 Spec |
| [22_claude future idea.md](22_claude%20future%20idea.md) | Future ideas and agent extensions | Future | 🔵 Notes |
| [23_kaio-connect.md](23_kaio-connect.md) | `kaio-connect` design | Future | 🔵 Spec |
| [23.5_kaio-connect_repport.md](23.5_kaio-connect_repport.md) | `kaio-connect` report | Future | 🔵 Report |
| [24_daemon_and_desktop_quality_report.md](24_daemon_and_desktop_quality_report.md) | Daemon & Desktop Quality Audit Report | Audit | 🟢 Published |

---

## 2. High-Level Architecture Diagram

```text
┌────────────────────────────────────────────────────────┐
│                    User / CLI / IDE                    │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│             Layer 5: Agent Runtime Layer               │
│  - Task planning (ReAct loop)                          │
│  - Tool execution (read, edit, bash, tests)            │
│  - Deterministic critic (cargo check / clippy / test)  │
│  - Basic safety guards & audit log recording           │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│            Layer 4: Context Retrieval Layer            │
│  - Token budgeting & priority truncation               │
│  - Exact paths -> Changed files -> Symbols -> Graph    │
│  - Failure memory & architectural decisions lookup     │
│  - Secret redaction & retrieval cache                  │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│              Layer 2: Memory Engine                    │
│  - SQLite storage (.agent/memory.sqlite, WAL mode)     │
│  - Structured relational tables (files, symbols, edges,│
│    commits, docs, failures, decisions, audit_logs)     │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│           Layer 3: Documentation Engine                │
│  - Path-mirrored markdown docs in .agent-docs/         │
│  - Levels 0 (Facts) to 3 (Deep Architectural Context)  │
│  - Frontmatter verification & staleness cascade        │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│         Layer 1: Codebase Scanner Layer                │
│  - Tree-sitter AST symbol extractor                    │
│  - Git diff change analyzer & file hashes (SHA-256)    │
│  - File inventory & language detection                 │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│              Layer 0: Trigger Layer                    │
│  - Non-blocking post-commit hook / git changes         │
│  - Lockfile concurrency safety (.agent/scan.lock)      │
│  - Manual CLI command (scan --changed / scan)          │
│  - Watch mode / Daily cron schedule                    │
└────────────────────────────────────────────────────────┘
```

---

## 3. Cross-Cutting Concerns

The architecture enforces five core cross-cutting concerns across all subsystems:

1. **Safety By Default**: File sandboxing, directory traversal prevention, command allowlists/blocklists, and secret redaction.
2. **Quantitative Evaluation**: Measurable task success rate, compiler pass rate, edit validity, and regression benchmarking.
3. **Comprehensive Observability**: Full traceability of all agent decisions, prompt contexts, tool executions, and file modifications.
4. **Cost & Token Control**: Strict context token budgets, selective LLM invocation, deterministic Level 0 doc generation, and caching.
5. **Immutable Audit Logging**: Every tool call, manual approval, rejection, and safety intercept is recorded to `.agent/logs/audit.log` and SQLite.

> [!IMPORTANT]
> **Safety & Observability are not end-of-project afterthoughts**: Basic safety guards, command blocklists, path sandboxing, and audit logging must be active from **Phase 10** when the coding agent is first introduced, with advanced permission policies expanded in **Phase 14**.

---

## 4. Resolved Missing Pieces

Based on comprehensive architectural review, the following enhancements have been established:
- **Retrieval Layer Specification**: Defined strict 9-step priority order, `ContextPacket` schema, token budgets, and cache invalidation.
- **Safety & Evaluation Specification**: Established permission tiers, path guards, command blocklists, audit logs, and 8 standardized benchmark tasks.
- **Early Safety Integration**: Embedded safety requirements into Phase 10 rather than deferring entirely to Phase 14.
- **Config Schema Modernization**: Added explicit `[limits]`, `[retrieval]`, `[safety]`, `[eval]`, and `[logging]` configuration blocks.
- **SQLite Concurrency & Schema Upgrades**: Specified WAL mode, busy timeouts, foreign key enforcement, and relational tables for `agent_runs`, `tool_calls`, `audit_logs`, `retrieval_cache`, and `import_raw`.

---

## 5. Progress & Active Focus

### Completed Phases
- [x] **Phase 1: Basic CLI Project Skeleton** (clap CLI, config loader, `kaioken memory init` & `status`)
- [x] **Phase 2: Robust Git Change Detection** (commit inspector, diff extractor, working tree change tracker, `.agent/scan.lock`, `kaioken memory scan --changed --no-llm`)
- [x] **Phase 3: File Scanner & Inventory Pre-filters** (workspace directory walk, ignore filtering, SHA-256 `file_hash`, binary detection, 1 MB size boundary, `parse_status`, `kaioken memory scan [--json]`)
- [x] **Phase 4: Tree-sitter AST Symbol Extraction with Error Recovery** (tree-sitter Rust parser, function/struct/enum/trait/impl/method extraction, visibility & exports, error recovery with `parse_status = 'partial'`)
- [x] **Phase 5: SQLite Memory Storage with WAL & Concurrency Safety** (SQLite schema migration, WAL journal mode, busy timeouts, foreign key enforcement, indexing, CRUD persistence for `files`, `symbols`, `import_raw`, `edges`, and `sync_inventory` bridge)
- [x] **Phase 6: Commit Memory Generation & Lockfile Safety** (diff impact analyzer, touched AST symbol detection via `--unified=0`, deterministic Level 0 Markdown summaries, risk assessment, non-blocking `.agent/scan.lock`, `kaioken memory commit` & `kaioken memory hook`)
- [x] **Phase 7: Documentation Mirror Engine & Frontmatter** (path-mirror markdown documentation into `.agent-docs/` matching source tree, YAML provenance frontmatter with `source_hash`, `doc_level`, `status`, deterministic Level 0 fact generation, `kaioken docs generate`, `status`, `verify`)
- [x] **Phase 8: Incremental Doc Updates & Staleness Cascade** (immediate direct staleness marking, 1-hop dependent cascade without transitive runaway, `kaioken docs generate --stale` incremental refreshes with token budget limits, `kaioken docs cascade`, and `kaioken docs verify --clean` orphan pruning)
- [x] **Phase 9: Retrieval Engine Layer** (prioritized, token-budgeted `ContextPacket`, secret redaction, LRU cache, sub-200ms latency tests)
- [x] **Phase 10: Autonomous Coding Agent Runtime** (ReAct loop, tool registry, path guards, command blocklist, streaming events)
- [x] **Phase 11: Deterministic Critic & Failure Memory** (`check`/`clippy`/`test` gates, repair loop, failure records)
- [x] **Phase 12: Memory-Augmented Agent Runtime** (failure warnings, learned decisions, retrieval context)
- [x] **Phase 13: Health Audits & Scheduling** (`kaioken schedule`, daily scan, stale docs, health report)
- [x] **Phase 14: Enterprise Permissions & Safety** (permission modes, approval handler, append-only audit log)
- [x] **Phase 15: Evaluation Suite** (benchmarks, metrics, scorecard)
- [x] **Phase 16: `kaioken doctor`** (environment diagnostics)
- [x] **Phase 17: `kaioken hook install`** (post-commit memory sync)
- [x] **Phase 18: `kaioken audit show`** (security/approval audit viewer)
- [x] **Phase 19: `memory explain` / `memory recent`** (file introspection and commit memories)
- [x] **Phase 20: `kaioken review`** (diff analysis, breaking changes, test gaps, risk score)
- [x] **Phase 21: `kaioken watch`** (debounced watcher + loopback IPC server and event stream)
- [x] **Phase 22: Multi-language AST parsers** (Rust, TypeScript/TSX, JavaScript/JSX, Python)
- [x] **Phase 23: GitHub Actions CI/CD** (Ubuntu/Windows/macOS matrix, fmt, clippy, tests)
- [x] **Phase 24: Native desktop cockpit** (`tao` + `wry` WebView, shared daemon request handler, embedded fallback)
- [x] **Phase 25: Interactive TUI** (`ratatui` dashboard with daemon/offline modes)

### Active Focus
- Harden the daemon/desktop seam: request deadlines (done), loopback-only trust model, and
  draining daemon events into the desktop model before each state push.
- Keep `kaioken desktop` and `kaioken tui` honest: both attempt the local daemon first and
  fall back to embedded/offline behavior with an explicit message.

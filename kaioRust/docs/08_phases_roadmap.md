# 08. 15-Phase Development Roadmap

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 17 & Section 25

To build the system layer-by-layer without rushing, development proceeds across 15 distinct, verifiable phases:

---

## Roadmap Phases

- [x] **Phase 1: Basic CLI Project Skeleton**
  - *Goal*: Cargo crate layout, clap CLI, tracing logging, config loader, `kaioken memory init` & `status`.
  - *DOD*: `kaioken --help`, `kaioken memory init`, `kaioken memory status` run successfully with zero errors.

- [x] **Phase 2: Robust Git Change Detection**
  - *Goal*: Inspect current commit hash, diff, and changed files. Handle dirty working trees, untracked files, fresh repositories with no commits, and merge commits cleanly.
  - *DOD*: `kaioken memory scan --changed` outputs accurate changed files across clean, dirty, or initial repo states.

- [x] **Phase 3: File Scanner & Inventory Pre-filters**
  - *Goal*: Walk workspace directory, filter ignores, identify languages, calculate SHA-256 `file_hash`, detect binary files, enforce 1 MB max size limit, and assign `parse_status`.
  - *DOD*: Scanner inventory lists entire repository structure with cryptographic hashes and status flags in structured JSON.

- [x] **Phase 4: Tree-sitter AST Symbol Extraction with Error Recovery**
  - *Goal*: Tree-sitter extraction of functions, structs, classes, methods, visibility, `exports`, and `imports_raw` for Rust. Gracefully recover from syntax errors with `parse_status = 'partial'`.
  - *DOD*: AST extractor returns structured symbols and raw imports from valid and partially broken source files.

- [x] **Phase 5: SQLite Memory Storage with WAL & Concurrency Safety**
  - *Goal*: SQLite schema migration, `PRAGMA journal_mode=WAL;`, `busy_timeout=5000;`, foreign keys, performance indexes, and CRUD persistence for `files`, `symbols`, `import_raw`, and `edges`.
  - *DOD*: Scanning writes structured records to SQLite; verified with queries and concurrent read/write test.

- [x] **Phase 6: Commit Memory Generation & Lockfile Safety**
  - *Goal*: Compute diff impact, touched symbols, and write markdown commit summary to DB. Include `.agent/scan.lock` concurrency guard and `--no-llm` deterministic fallback.
  - *DOD*: Post-commit hook produces and stores a commit understanding entry without blocking git workflows.

- [x] **Phase 7: Documentation Mirror Engine & Frontmatter**
  - *Goal*: Path-mirror markdown documentation into `.agent-docs/` matching source files. Prepend YAML frontmatter with `source_hash`, `doc_level`, and `status`. Level 0 deterministic docs generated first.
  - *DOD*: Generating docs produces mirrored markdown files (e.g. `.agent-docs/src/main.md`) with valid provenance frontmatter.

- [x] **Phase 8: Incremental Doc Updates & Staleness Cascade**
  - *Goal*: Mark direct changed files stale immediately; mark direct dependents as maybe-stale; avoid cascading transitive updates. Implement `kaioken docs verify` to check broken links and orphaned docs.
  - *DOD*: `kaioken docs generate --stale` refreshes only stale entries within token budget limits.

- [x] **Phase 9: Retrieval Engine Layer (Mandatory Pre-requisite for Agent)**
  - *Goal*: Assemble structured `ContextPacket` prioritizing exact paths -> changed files -> symbols -> graph neighbors -> failures -> docs. Implement token budgeting truncator, secret redaction, and `retrieval_cache`.
  - *DOD*: `kaioken retrieval context <task>` outputs a budgeted, redacted context packet in < 200ms.

- [x] **Phase 10: Basic Coding Agent with Active Safety & Audit Logging**
  - *Goal*: OpenRouter streaming client and ReAct loop with **active safety guards from day one**: path traversal blocking, command blocklist, 100 KB output truncation, 120s timeout, max iterations, and audit logging to `tool_calls` / `audit.log`.
  - *DOD*: Agent autonomously implements a single-file function or test within sandbox boundaries and logs all tool executions.

- [x] **Phase 11: Deterministic Critic & Failure Memory**
  - *Goal*: Automatic execution of `cargo check` / `test` after edit, feeding diagnostics back on failure. Write error signatures to `failures` table and log successful fixes upon self-correction.
  - *DOD*: Agent self-corrects a compiler error without human intervention and records the solution in failure memory.

- [x] **Phase 12: Memory-Augmented Agent Runtime**
  - *Goal*: Mandatory pre-planning retrieval of `ContextPacket`. Agent cites relevant symbol dependencies and past failures in plan, applies edits, and updates `agent_runs` and docs upon completion.
  - *DOD*: Agent cites past repository failures during planning and updates memory state after execution.

- [ ] **Phase 13: Daily Scheduled Scanner & Health Audit**
  - *Goal*: Off-peak scan auditing dead code, missing unit tests, stale docs, and broken graph links.
  - *DOD*: `kaioken schedule --daily` outputs a comprehensive Markdown repository health report.

- [ ] **Phase 14: Advanced Safety & Enterprise Permission System**
  - *Goal*: Full permission modes (`read-only`, `edit-with-approval`, `auto-edit`, `safe-bash`, `full-auto`), interactive approval prompts, `--dry-run`, `--yes`, and secret pattern sanitization.
  - *DOD*: Agent halts and prompts before executing actions outside configured permission policy.

- [ ] **Phase 15: Quantitative Evaluation & Benchmark Suite**
  - *Goal*: Evaluation harness (`kaioken eval run`) testing 8 standardized engineering benchmarks, tracking task success rate, edit validity, token cost, critic iterations, and regression prevention.
  - *DOD*: Benchmark harness executes testbed tasks and emits a quantitative evaluation scorecard.

---

## Strict Phase Gate Rules

To preserve architectural integrity and avoid rushing:
1. **Gate 1 (Retrieval Pre-requisite)**: Development **must not** advance to Phase 10 until Phase 9 (`ContextPacket` generation and token budgeting) is fully implemented and tested.
2. **Gate 2 (Critic Self-Correction)**: Phase 11 cannot be marked complete unless the agent autonomously self-corrects at least one real compiler/type error and records the fix in `failures`.
3. **Gate 3 (End-to-End Loop)**: Phase 12 cannot be marked complete unless the agent retrieves context before planning and updates SQLite memory (`agent_runs`, `tool_calls`) after completion.

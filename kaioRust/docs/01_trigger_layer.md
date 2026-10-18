# 01. Trigger Layer (Layer 0)

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 3 (Layer 0) & Section 11

---

## 1. Purpose & Scope

The **Trigger Layer** determines *when* KAIOKEN Agent executes indexing, memory updates, and documentation synchronization.

**Core Production Constraints**:
- **Non-blocking Execution**: Git operations must never be delayed by slow LLM calls. The post-commit hook performs rapid deterministic parsing; expensive LLM documentation generation is asynchronous or queued.
- **Fail-Safe Operation**: If the trigger or memory update encounters an error, the git commit itself must **never** be failed or aborted. Errors are logged to `.agent/logs/`, leaving repository state clean.
- **Idempotence**: Running scans repeatedly on the same state must produce identical, consistent database records without duplication.

```text
               Git Commit
                   │
                   ▼
       ┌────────────────────────┐
       │   Post-Commit Hook     │
       └────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│  agent memory scan --changed         │
│  - Acquire scan.lock                 │
│  - Inspect commit hash & diff        │
│  - Update deterministic memory       │
│  - Mark docs stale                   │
│  - Queue optional doc regeneration   │
│  - Release scan.lock                 │
└──────────────────────────────────────┘
```

---

## 2. Trigger Types & Flags

1. **Post-Commit Git Hook (Primary Trigger)**:
   - Installed via `kaioken hook install`.
   - Fires automatically after every `git commit`.
   - Runs `kaioken memory scan --changed --no-llm` by default to ensure sub-second completion.
2. **Manual CLI Invocations**:
   - `kaioken memory scan`: Full codebase re-scan.
   - `kaioken memory scan --changed`: Scan only files modified in the latest commit or working tree.
   - `--no-llm`: Flag forcing deterministic memory updates only (AST, Git, Graph) with zero API costs.
3. **File Watcher (`kaioken watch`)**:
   - Monitors filesystem events via debounced notifications. Marks touched files and mirrored docs as `stale` in real time.
4. **Daily Scheduled Scanner (`kaioken schedule --daily`)**:
   - Off-peak deep maintenance: dead code detection, doc freshness audit, graph validation, and repo health summary.

---

## 3. Concurrency Safety & Lockfile Behavior

To prevent race conditions and SQLite database contention when commits happen in rapid succession or concurrent terminal tabs:
- **Lockfile**: Acquired at `.agent/scan.lock` using process ID and timestamp.
- **Concurrency Guard**: If `.agent/scan.lock` exists and is active, secondary scan attempts exit cleanly or wait up to 3 seconds before skipping without blocking the developer.
- **Stale Lock Recovery**: If a lockfile is older than 5 minutes (indicating a previous process crashed), it is automatically cleaned and reclaimed with an audit log notice.

---

## 4. Documentation Regeneration Policy

Documentation synchronization follows a strict tiered approach:
1. **Immediate Staleness Marking**: Any file touched in the commit immediately transitions its `.agent-docs/` record to `status = 'stale'` in SQLite.
2. **Deterministic Regeneration (Level 0/1)**: Raw AST facts (Level 0) and cheap summaries (Level 1) can update immediately if configured.
3. **Deep Regeneration (Level 2/3)**: High-priority, high-risk, or core API files only regenerate when explicitly requested by the user or during scheduled batch runs within token budgets.

---

## 5. Post-Commit Execution Flow

The post-commit sequence runs as follows:

```text
1. Acquire .agent/scan.lock (recovering stale locks if needed)
2. Read current commit HEAD hash and metadata
3. Query git diff for added, modified, renamed, and deleted files
4. Update deterministic memory:
     a. Compute SHA-256 file_hash
     b. Re-parse AST with Tree-sitter (handling syntax errors gracefully)
     c. Update files and symbols tables in SQLite (WAL mode)
     d. Refresh edge relationships and unresolved import_raw entries
5. Mark affected mirrored docs as 'stale' in docs table
6. Queue optional Level 1 summary generation if --no-llm is not set
7. Release .agent/scan.lock and log execution metrics to .agent/logs/
```

---

## 6. Failure Handling & Recovery

- If a scan step throws an exception (e.g., malformed syntax, locked database, IO failure), the error is written to `.agent/logs/trigger_errors.log`.
- SQLite database transactions are rolled back to preserve memory integrity.
- The git commit hook process always exits with status code `0`, ensuring the developer's git workflow remains uninterrupted.

---

## 7. Current Implementation Status

- [x] Defined non-blocking trigger architecture and safety policies.
- [x] Stubbed CLI commands (`memory scan`, `watch`, `schedule`, `hook install`).
- [ ] Phase 2: Git commit hash and changed file detection (`src/scanner/git.rs`).
- [ ] Phase 2: Lockfile implementation (`.agent/scan.lock`) and `--no-llm` flag.
- [ ] Phase 6: Post-commit hook installer script (`kaioken hook install`).

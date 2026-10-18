# 09. Safety, Permissions & Evaluation

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 14, Section 21 & Section 22

---

## 1. Purpose & Scope

Autonomous coding agents possess the ability to read files, execute shell commands, edit code, and interact with remote APIs. Without strict, layered safety guards enforced by default, agents pose immediate risks of data loss, destructive git commands, accidental credential exposure, and runaway execution costs.

**Fundamental Safety Rule**:
> Basic safety guards (path traversal guards, command blocklists, audit logging, timeouts) are active from **Phase 10** when the coding agent begins—not deferred to the end of development. Advanced policy engines and enterprise access controls are matured in **Phase 14**, while quantitative benchmarks run in **Phase 15**.

---

## 2. Permission Modes

KAIOKEN Agent operates under five explicit permission modes configured via `.agent/config.toml` or CLI flags:

| Mode | Reads | Edits | Shell Commands | Intended Use Case |
|---|---|---|---|---|
| `read-only` | Allowed | Blocked | Blocked | Audit, exploration, retrieval context inspection |
| `edit-with-approval` *(Default)* | Allowed | Requires Prompt | Requires Prompt | Standard developer pair programming mode |
| `auto-edit` | Allowed | Allowed | Requires Prompt | Code editing with manual approval for shell tasks |
| `safe-bash` | Allowed | Allowed | Allowed (Allowlist only) | Automated testing, linting, build pipelines |
| `full-auto` | Allowed | Allowed | Allowed | CI/CD sandbox environments with disposable containers |

---

## 3. Filesystem Sandbox

The filesystem guard (`src/safety/path_guard.rs`) enforces strict boundaries on all file operations (`read_file`, `edit_file`, `create_file`):

1. **Workspace Root Restriction**: All target paths are strictly canonicalized and must resolve within the repository root directory.
2. **Path Traversal Block**: Any path attempting directory traversal (`../`, absolute paths outside root, Windows drive traversal) is immediately aborted and recorded as a safety violation.
3. **Protected Directory Quarantine**:
   - Direct writes to `.git/` are blocked (git changes must proceed through git tooling).
   - Direct writes to `.agent/memory.sqlite` by tool calls are blocked (memory writes occur only via the internal `MemoryEngine`).
4. **Secret File Exclusion**: Writes or reads targeting `.env`, `.env.*`, `credentials.json`, `id_rsa`, `*.pem`, or `token` are blocked by default.

---

## 4. Command Safety & Sandboxing

The command guard (`src/safety/command_guard.rs`) intercepts every shell invocation before execution:

### Strict Command Blocklist
The following commands and patterns are unconditionally blocked:
```text
rm -rf / rmdir /s
git push --force / git push -f
git reset --hard
git clean -fd
drop table / truncate
curl * | bash / wget * | bash
npm publish / cargo publish
mkfs / dd / fdisk
```

### Execution Constraints
- **Command Timeout**: Hard ceiling of 120 seconds per command (configurable via `max_command_timeout_seconds`).
- **Output Buffering Limit**: Maximum of 100 KB stdout/stderr returned to the agent loop; excess output is truncated and flushed to `.agent/logs/`.
- **Output Redaction**: All command outputs pass through regex sanitization to strip potential exposed secrets before reaching LLM context.

---

## 5. Approval Flow & Non-Interactive Flags

When an operation requires user consent:
1. The agent presents a structured prompt describing:
   - Target action (e.g. `edit_file: src/auth/session.rs` or `bash: cargo test`)
   - Diff preview or command string
   - Rationale and risk level
2. **CLI Flags**:
   - `--dry-run`: Runs the planning and critic loop, emitting diffs without writing to disk.
   - `--yes` (`-y`): Explicit user consent bypass for CI/automated environments.
3. Every human approval or rejection is recorded in the audit log.

---

## 6. Audit Logging & Observability

Every agent action must be observable, attributable, and auditable. Logging occurs to both `.agent/logs/audit.log` and the SQLite `audit_logs` table:

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  actor TEXT NOT NULL,         -- 'agent' | 'user'
  action TEXT NOT NULL,        -- 'edit_file' | 'bash' | 'create_file'
  target TEXT NOT NULL,        -- file path or command string
  approved INTEGER NOT NULL,   -- 1 = approved, 0 = rejected/blocked
  reason TEXT,                 -- policy rule triggered or user explanation
  created_at TEXT NOT NULL
);
```

---

## 7. Secret Protection & Redaction

The sanitization module (`src/safety/redact.rs`) strips secrets before data enters SQLite logs, ContextPackets, or LLM prompts:
- API Keys: Pattern matching for OpenAI (`sk-...`), GitHub (`ghp_...`), AWS tokens, and OpenRouter keys.
- Private Keys: Interception of PEM/RSA headers (`BEGIN PRIVATE KEY`).
- Environment Variables: Masking values of variables named `*_KEY`, `*_SECRET`, `*_TOKEN`, `*_PASSWORD`.

---

## 8. Quantitative Evaluation Metrics

In **Phase 15**, the agent is evaluated against standardized engineering metrics:

| Metric | Target | Description & Formula |
|---|---|---|
| **Task Success Rate** | > 85% | Percentage of benchmark tasks completed with all tests passing |
| **Edit Validity Rate** | > 95% | Modifications successfully applied without search-replace failure |
| **Compiler First-Pass Rate** | > 75% | Changes compiling on first attempt without critic intervention |
| **Average Critic Iterations** | < 2.0 | Number of compiler/test self-correction cycles required per task |
| **Average Tool Calls** | < 8.0 | Efficiency of tool usage per task |
| **Average Cost per Task** | < $0.05 | Tracked OpenRouter token cost across prompt and completion |
| **Human Interventions** | < 1.0 | Number of manual user overrides or rejections per task |
| **Safety Violation Count** | **0** | Any attempted sandbox breach or blocked command attempt |
| **Documentation Freshness**| > 90% | Percentage of mirrored markdown docs matching source code hashes |

---

## 9. Standardized Benchmark Tasks

The evaluation harness (`src/eval/benchmarks.rs`) executes reproducible tasks against a sandbox testbed:

1. **Add Function**: Implement a new typed helper function with doc comments.
2. **Fix Compiler Error**: Resolve a `mismatched types` or `cannot borrow as mutable` error.
3. **Fix Broken Unit Test**: Diagnose an assertion failure and repair the logic.
4. **Cross-File Symbol Rename**: Rename a public struct across 4 files and ensure compilation.
5. **Add Unit Test Suite**: Write comprehensive edge-case tests for an existing module.
6. **Module Refactor**: Split a 500-line module into clean submodules while preserving public API.
7. **Synchronize Mirrored Docs**: Regenerate stale docs after a signature change.
8. **Dependency Impact Analysis**: Accurately predict the blast radius of modifying a shared utility.

---

## 10. Regression Testing

Every completed bug fix or milestone generates an automated regression test:
- Repositories maintain `.agent/eval/regression_suite.json`.
- Agent modifications must pass existing regression scenarios without token inflation (>20%) or accuracy degradation.

---

## 11. Implementation Modules

The safety and evaluation systems are partitioned into:

```text
src/safety/
├── mod.rs               # SafetyPolicy orchestrator
├── path_guard.rs        # Filesystem sandbox & path traversal filter
├── command_guard.rs     # Shell command blocklist & timeouts
├── approval.rs          # Interactive approval prompts & CLI flag handling
├── audit.rs             # File and SQLite audit log writers
└── redact.rs            # Regex secret & token masking engine

src/eval/
├── mod.rs               # Evaluation suite runner
├── metrics.rs           # Cost, iteration, and pass-rate calculators
└── benchmarks.rs        # 8 standardized benchmark tasks & runners
```

---

## 12. Definition of Done

- **Phase 10 (Basic Safety)**:
  - Command guard intercepts all blocked commands.
  - Path guard blocks access outside repository root.
  - Audit logs record all tool calls to `.agent/logs/audit.log`.
  - Max iteration and timeout limits abort runaway tasks.
- **Phase 14 (Advanced Safety)**:
  - Full permission modes (`read-only` through `full-auto`) operational.
  - Interactive approval prompts with `--dry-run` and `--yes` support.
  - Secret redaction prevents token and credential leakage.
- **Phase 15 (Evaluation Harness)**:
  - `kaioken eval run` executes benchmark tasks.
  - Metrics report outputs pass rate, token cost, and critic iterations.

---

## 13. Current Implementation Status

- [x] Defined permission models and blocked command lists in configuration.
- [x] Defined safety guard architecture and evaluation metrics.
- [x] Phase 10: Implement `src/safety/path_guard.rs` and `command_guard.rs` basic guards.
- [x] Phase 10: Implement `src/safety/audit.rs` tool call logging.
- [x] Phase 14: Implement `src/safety/approval.rs` interactive prompts.
- [x] Phase 14: Implement `src/safety/redact.rs` secret sanitization.
- [x] Phase 15: Implement `src/eval/` benchmark suite and metric reporter.

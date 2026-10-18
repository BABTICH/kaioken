# 06. Agent Runtime Layer (Layer 5)

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Sections 8, 9, 10

---

## 1. Purpose & Scope

The **Agent Runtime Layer** executes autonomous development workflows using a **ReAct + Deterministic Critic** loop. Rather than guessing blindly or hallucinating changes, the agent:
1. Calls **Layer 4 Retrieval** to receive a structured `ContextPacket` before formulating any plan.
2. Selects tools constrained by strict path sandboxing, command blocklists, and output truncation.
3. Submits all file edits to immediate deterministic validation (`cargo check`, linters, tests).
4. Persists execution history, tool metrics, failures, and fixes back into SQLite memory upon completion.

```text
User Task Request
       │
       ▼
[Layer 4: Retrieve ContextPacket]
(Symbols, Files, Past Failures, Docs, ADRs)
       │
       ▼
[Task Planning & Tool Selection]
       │
       ▼
┌──────────────────────────────────────┐
│        ReAct Loop Cycle              │
│  1. Agent chooses tool               │
│  2. Path/Command Guards inspect      │
│  3. Tool executes (with timeout)     │
│  4. Output truncated & logged        │
│  5. Critic evaluates result          │
└──────────────────────────────────────┘
       │
       ├─► [Compiler / Test Failure?] ──► Store Failure in DB & Feed to LLM ──┐
       │                                                                      │
       ▼                                                                      │
[All Deterministic Checks Pass] ◄─────────────────────────────────────────────┘
       │
       ▼
[Update Layer 2 Memory & Layer 3 Docs]
- Record agent_runs & tool_calls
- Record successful fix in failures table
- Mark modified files & docs as stale
```

---

## 2. Pre-Planning Retrieval Requirement

Before the agent loop generates a task plan:
- It **must** invoke `RetrievalEngine::build_context_packet(&task)`.
- The prompt incorporates relevant symbol signatures, file snippets, past failure records, and architectural decisions.
- If the token budget is reached, priority-based truncation ensures compiler errors and core signatures are preserved.

---

## 3. Strict Runtime Limits & Budget Controls

To prevent runaway agent loops, token exhaustion, and frozen processes, the runtime enforces hard limits configured in `.agent/config.toml`:

| Limit Parameter | Default Value | Action on Breach |
|---|---|---|
| `max_iterations` | 20 | Aborts execution; summarizes partial progress |
| `max_tool_calls` | 100 | Halts tool execution; requests human guidance |
| `max_command_timeout_seconds` | 120s | SIGKILL sent to subprocess; returns timeout error |
| `max_tool_output_bytes` | 100 KB | Output truncated; full output written to `.agent/logs/` |
| `max_context_tokens` | 8,000 | Retrieval truncates lower-priority context |
| `max_cost_usd` *(Optional)* | $2.00 | Aborts loop if accumulated token cost exceeds threshold |

---

## 4. Tool Registry & Safety Interception

Every tool call is intercepted by safety guards before execution:

| Tool | Functionality | Interceptor / Safety Check |
|---|---|---|
| `read_file` | Read line range or complete file | `path_guard`: Restricts to workspace; blocks secrets |
| `edit_file` | Precise search-and-replace snippet | `path_guard`: Validates exact string match; blocks `.git/` |
| `create_file`| Create new source file | `path_guard`: Restricts to workspace; prompts if exists |
| `grep_search`| Ripgrep pattern matching | Read-only; respects workspace ignore patterns |
| `glob_search`| Discover files matching glob | Read-only; bounded to max 200 matches |
| `bash` | Execute shell command | `command_guard`: Blocklist (`rm -rf`, `force push`, etc.) |
| `run_tests` | Run test suites | Bounded by timeout; parses exit status and failure stack |

---

## 5. Tool Output Truncation

If a tool emits large output (e.g. a compiler dump with 500 errors or large grep result):
1. The full output is written to `.agent/logs/tools/<tool_call_id>.log`.
2. The agent prompt receives only the first 100 lines and a summary notice:
   ```text
   [Output truncated: 4,500 lines omitted. Full log saved to .agent/logs/tools/142.log]
   ```

---

## 6. Deterministic Critic & Self-Correction

The agent does not prompt the user for validation until deterministic tools confirm the code works:
- **Rust**: `cargo check`, `cargo clippy`, `cargo test`
- **TypeScript**: `tsc --noEmit`, `eslint`, `npm test`
- **Python**: `ruff check`, `mypy`, `pytest`

### Feedback Loop
- If the compiler fails, compiler stderr is fed directly back to the agent:
  *"Compiler error E0308 (mismatched types) on line 45. Fix this error before proceeding."*
- If the fix succeeds on the next iteration, the system logs the error and solution into the `failures` table as a verified fix.

---

## 7. Post-Task Memory & Documentation Synchronization

Upon completing or failing a task, the runtime executes mandatory persistence:
1. **Agent Run Record**: Inserts record into `agent_runs` (task description, duration, model, tokens used, estimated cost).
2. **Tool Calls Log**: Inserts records into `tool_calls` with execution latency and status.
3. **Failure Resolution**: If an earlier compiler error was resolved, update `failures` (`status = 'resolved'`, `fix_summary = '...'`).
4. **Doc Staleness**: Transitions `.agent-docs/` records for touched files to `stale`.

---

## 8. Safe Cancellation & Graceful Interruption

When the developer issues a SIGINT (Ctrl+C) or timeout triggers:
- Running child processes are terminated cleanly.
- The runtime captures all partial edits made up to that point.
- An interruption report is displayed:
  ```text
  ⚠️  Task cancelled by user.
  Modified files:
    - src/auth/session.rs (Partial edit applied)
  Run 'git status' or 'git diff' to review pending changes.
  ```

---

## 9. Current Implementation Status

- [x] Defined agent interfaces in `src/agent/mod.rs`.
- [x] Defined tool interfaces and ReAct loop structure.
- [x] Phase 10: OpenRouter tool calling with runtime limits and safety interceptors.
- [x] Phase 10: Tool call logging to `tool_calls` table and output truncation.
- [x] Phase 11: Deterministic compiler/test critic with failure memory persistence and Gate 2 self-correction loop.
- [ ] Phase 12: Wire Layer 4 `ContextPacket` into pre-planning prompt.

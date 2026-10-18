# 00. Vision & Core Principles

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Sections 1, 19, 20, 27

---

## 1. The Vision

The goal of KAIOKEN Agent is not simply to be an LLM chatbot that edits files, but a **repository-aware software engineering agent**.

Traditional agents start every session with an empty slate or limited context window. KAIOKEN continuously understands:
- What files and modules exist.
- What each function, struct, and class does.
- What depends on what (dependency graph).
- What changed recently and the exact impact.
- What compiler errors, test failures, and bug fixes happened in the past.
- What architectural decisions were made and why.

---

## 2. Core Principles (Rules 1 - 12)

| Rule # | Principle | Description |
|---|---|---|
| **Rule 1** | **Code Is The Source Of Truth** | Docs and memory are derived artifacts. If documentation disagrees with code, trust the code. |
| **Rule 2** | **Do Not Regenerate Everything** | Always use incremental updates. Only re-index and re-document changed files and their immediate impact radius. |
| **Rule 3** | **Use Deterministic Tools First** | AST parsing, ripgrep, git diff, compiler diagnostics, and dependency graphs run first; LLM is reserved for synthesis and planning. |
| **Rule 4** | **Memory Is Context, Not Truth** | Memory can become stale. Verify critical assumptions against the compiler, git status, and unit tests. |
| **Rule 5** | **Keep Context Small** | Send targeted symbols, recent errors, and dependency subsets to the LLM rather than dumping the whole repo. |
| **Rule 6** | **Search-Replace Edits Over Full Rewrites** | Prefer localized search-replace edits with strict validation over blind file rewrites. |
| **Rule 7** | **Every Edit Must Be Verifiable** | Immediately execute `cargo check`, linters, and tests after modifications. |
| **Rule 8** | **Track Failures Systematically** | Record errors, causes, and successful fixes into failure memory to avoid repeat mistakes. |
| **Rule 9** | **Safety By Default** | The agent must not perform destructive operations without explicit consent. Workspace writes and commands must be sandboxed by default. |
| **Rule 10** | **Audit Everything** | Every tool call, manual approval, rejection, failure, and fix must be logged to immutable audit storage. |
| **Rule 11** | **Cost-Aware Intelligence** | Use cheap models for summarization and expensive models for planning/coding only when needed. Retrieval must enforce strict token quotas. |
| **Rule 12** | **Observable Progress** | Every phase must have a verifiable Definition of Done. Agent performance, cost, and accuracy must be quantitatively measured. |

---

## 3. How This Beats Claude Code / Codex

1. **Persistent Repository Memory**: Retains knowledge across reboots, sessions, and git branches in structured SQLite tables.
2. **Persistent Failure Memory**: Remembers previous compiler errors, broken test assertions, and their working fixes so the agent never repeats mistakes.
3. **Change Impact Warnings**: Graph traversal alerts engineers and the planner before edits break downstream dependencies.
4. **Living Mirrored Docs**: Documentation mirrors code 1-to-1 in `.agent-docs/`, keeping architecture notes synchronised.
5. **Repo-Specific Learning Over Time**: Learns repository conventions, preferred error patterns, and flaky test quirks dynamically.
6. **Deterministic Pre-filtering**: Retrieves exact AST symbols and graph paths before relying on vector embeddings.
7. **Cost-Aware Retrieval**: Token budgets and selective Level 0/1 doc tiers prevent runaway API token bills.
8. **Comprehensive Auditability**: Complete visibility into every tool call, diff applied, and approval granted.

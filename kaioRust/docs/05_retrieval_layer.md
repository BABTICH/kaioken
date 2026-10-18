# 05. Context Retrieval Layer (Layer 4)

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 7 & Section 14

---

## 1. Purpose & Scope

The **Context Retrieval Layer** is the intelligent mediation layer between the repository's structured memory (Layer 2) and the autonomous agent runtime (Layer 5). Its job is to assemble a compact, verifiable, high-signal **Context Packet** for any task before the LLM generates a single line of code or plan.

The system adheres to a strict token-budgeted, deterministic-first philosophy:
- Never dump full files or entire repository trees into prompt contexts.
- Resolve exact structural facts (AST, graph, git) before falling back to statistical search.
- Enforce strict token quotas and secret redaction before emitting context.

```text
User Task Request: "Fix session token expiry check"
                         │
                         ▼
┌────────────────────────────────────────────────────────┐
│            Layer 4: Retrieval Engine Pipeline          │
│                                                        │
│  1. Task Query Decomposition & Cache Lookup            │
│  2. Deterministic Structural Retrieval (AST/Graph/Git) │
│  3. Failure Memory & Architectural Decision Lookup     │
│  4. Documentation & Staleness Assessment               │
│  5. Privacy Guard & Secret Redaction                   │
│  6. Token Budget Allocation & Priority Truncation      │
└────────────────────────────────────────────────────────┘
                         │
                         ▼
        Structured, Compact ContextPacket -> Agent
```

---

## 2. Retrieval Priority Order

When gathering context for a coding or planning task, queries must strictly execute in the following priority order:

| Priority | Source | Mechanism | Purpose |
|---|---|---|---|
| **1** | **Exact File Paths** | Filesystem & `files` table | Pinpoint targeted source and test files directly mentioned in task |
| **2** | **Changed Files** | Git diff / staging status | Identify active working changes and immediate blast radius |
| **3** | **AST Symbols** | `symbols` table (Tree-sitter) | Load exact signatures, visibility, and line boundaries for symbols |
| **4** | **Dependency Graph Neighbors** | `edges` table (`calls`, `imports`) | Identify callers, callees, and modules directly affected (Depth 1-2) |
| **5** | **Compiler & Test Failures** | `failures` table | Pull previous errors, causes, and proven fixes for related symbols |
| **6** | **Grep Matches** | ripgrep / regex search | Locate string literals, configuration keys, or macro references |
| **7** | **Glob Matches** | Path glob matching | Discover related sibling files, migrations, or test fixtures |
| **8** | **Mirrored Markdown Docs** | `.agent-docs/` & `docs` table | Extract high-level purpose, documented risks, and API contracts |
| **9** | **Semantic / Vector Search** | Vector cosine similarity | *Fallback only* for conceptual inquiries when exact symbols fail |

---

## 3. Context Packet Schema

The output of Layer 4 is an immutable, structured `ContextPacket` that is serialized into the agent's prompt context:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    /// Normalized summary of the user task and intended goal
    pub task_summary: String,

    /// Exact file paths with targeted line ranges and source snippets
    pub relevant_files: Vec<FileContextSnippet>,

    /// Specific AST symbol definitions, signatures, and locations
    pub relevant_symbols: Vec<SymbolContextSummary>,

    /// Localized sub-graph of callers, callees, and importing modules
    pub dependency_graph_subset: GraphSubset,

    /// Recent commits modifying the relevant files or symbols
    pub recent_related_commits: Vec<CommitSummarySnippet>,

    /// Previous failures, compiler errors, or bug fixes on touched areas
    pub related_failures: Vec<FailureMemorySnippet>,

    /// Warnings if mirrored docs or referenced specs are stale
    pub stale_docs_warnings: Vec<DocStalenessWarning>,

    /// Architectural decisions (ADRs) constraining this area of code
    pub architectural_decisions: Vec<DecisionSnippet>,

    /// Exact verification commands recommended (e.g. `cargo test --test auth`)
    pub recommended_verification_commands: Vec<String>,

    /// Detailed token budgeting report tracking consumption across sections
    pub token_budget_report: TokenBudgetReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContextSnippet {
    pub path: String,
    pub language: String,
    pub line_start: usize,
    pub line_end: usize,
    pub content: String,
    pub source_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolContextSummary {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub signature: String,
    pub visibility: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSubset {
    pub direct_callers: Vec<String>,
    pub direct_callees: Vec<String>,
    pub direct_imports: Vec<String>,
    pub affected_tests: Vec<String>,
}
```

---

## 4. Query Types

The retrieval engine exposes five distinct query primitives:

1. **Path Query**: Resolves a file path to its metadata, language, AST summary, and mirrored doc status.
2. **Symbol Query**: Exact or fuzzy symbol lookup yielding signatures, line spans, and enclosing scopes.
3. **Grep Query**: Deterministic ripgrep invocation scoped to workspace source files (respecting ignore rules).
4. **Graph Impact Query**: Traverses call and import edges to construct a blast-radius report.
5. **Failure Lookup Query**: Queries the `failures` table for historical error signatures matching current files or symbols.

---

## 5. Graph Traversal Rules

To prevent context explosion and infinite recursion during graph traversal:
- **Default Depth**: Traverse direct neighbors only (`depth = 1`).
- **Maximum Depth**: May expand to `depth = 2` only when evaluating blast radius for core public APIs.
- **Symbol Limits**: Maximum of 30 symbols captured per graph expansion.
- **Exclusion Filters**: Graph traversal strictly ignores:
  - Generated files (`*.pb.rs`, `*.gen.ts`, `*.min.js`)
  - Vendored third-party code (`vendor/`, `.reference/`)
  - Artifact directories (`node_modules/`, `target/`, `dist/`, `build/`, `.agent/`)

---

## 6. Failure Memory Lookup

The agent must learn from past mistakes rather than repeating them:
1. When a task touches `src/auth/session.rs` or `validate_session`, query the `failures` table where `file_id` or `symbol_id` matches.
2. Filter for unresolved or recently fixed incidents.
3. Include the root cause, compiler error message, and fix summary in the `ContextPacket`.
4. Prompt format:
   ```text
   PAST FAILURE WARNING:
   Function 'validate_session' previously failed with:
     - Error: TypeError / Null dereference on empty bearer token
     - Root Cause: Missing header guard before slice extraction
     - Fix: Implemented Option<&str> guard in PR #42
   ```

---

## 7. Documentation Lookup & Staleness

1. For each touched file, check `.agent-docs/` for a mirrored markdown document.
2. Inspect the document frontmatter:
   - If `status: fresh` and `source_hash` matches current file content, extract Level 0/1 summary.
   - If `status: stale` or hash mismatch, append a `DocStalenessWarning` to `ContextPacket`.
3. The agent is explicitly informed: *"Doc is stale; rely on source code AST as truth."*

---

## 8. Token Budgeting & Truncation

The retrieval engine enforces strict token budgets defined in `.agent/config.toml`:

| Section | Default Token Budget | Truncation Policy |
|---|---|---|
| **Relevant Files** | 4,000 tokens | Keep active functions/hunks; drop unrelated functions |
| **Relevant Symbols** | 1,500 tokens | Retain signatures and doc comments; drop private helpers |
| **Dependency Graph** | 800 tokens | Limit to direct callers/tests; drop transitive imports |
| **Failure Memory** | 600 tokens | Keep latest 3 relevant failures; drop historical duplicates |
| **Docs & ADRs** | 600 tokens | Extract purpose and risk bullet points only |
| **Total Context Budget** | **8,000 tokens** | Hard ceiling. Priority-based dropping if exceeded |

### Priority-Based Dropping Sequence
If total tokens exceed `max_context_tokens`:
1. Drop semantic/vector search results first.
2. Drop documentation paragraphs (keep raw Level 0 AST facts).
3. Drop transitive commit summaries.
4. Truncate file context snippets to target function boundaries.
5. *Never drop*: Compiler errors, targeted symbol signatures, or safety warnings.

---

## 9. Caching Strategy

To avoid redundant AST parsing and graph traversal during rapid multi-step interactions:
- **Cache Key**: `SHA-256(task_summary + repo_head_commit + active_flags)`
- **Storage**: `retrieval_cache` table in `.agent/memory.sqlite`.
- **Invalidation**:
  - Automatically invalidated when any touched file is modified on disk.
  - Hard TTL expiration: 10 minutes.
  - Cleared on git branch switch or new commit.

---

## 10. Privacy, Redaction & Sandboxing

Prior to inclusion in any `ContextPacket` or LLM dispatch:
1. **Secret Scanning**: Regex filter intercepts and masks API keys (`sk-...`, `ghp_...`, `Bearer ...`), private keys, and passwords.
2. **Environment Variable Exclusion**: `.env`, `.env.local`, and credential files are strictly blocked from retrieval.
3. **Workspace Boundaries**: File paths outside the workspace root are dropped immediately with an audit violation warning.

---

## 11. Implementation Modules

The retrieval subsystem is structured in `kaioRust/src/retrieval/`:

```text
src/retrieval/
├── mod.rs               # RetrievalEngine coordinator & public traits
├── context_packet.rs    # ContextPacket data models and serializes
├── graph_query.rs       # Neighbor traversal and blast radius analysis
├── failure_lookup.rs    # Historical error & fix matching
├── doc_lookup.rs        # Mirrored markdown & staleness validator
├── budget.rs            # Token counter and priority-based truncator
└── cache.rs             # SQLite retrieval_cache store & invalidator
```

---

## 12. Definition of Done (Phase 9)

Phase 9 is complete when:
1. `kaioken retrieval context "task description"` executes and outputs a valid JSON/text `ContextPacket`.
2. Exact file and symbol queries resolve accurately against SQLite memory.
3. Dependency graph queries correctly report callers and affected unit tests.
4. Token budget limits are strictly enforced and reported.
5. All secrets and `.env` values are redacted from output.
6. Retrieval cache stores and invalidates entries accurately across edits.

---

## 13. Current Implementation Status

- [x] Defined retrieval interfaces in `src/retrieval/mod.rs`.
- [x] Defined `ContextPacket` schema and query contracts.
- [x] Phase 9: Implement `src/retrieval/context_packet.rs` and builders.
- [x] Phase 9: Implement `src/retrieval/graph_query.rs` edge traversals.
- [x] Phase 9: Implement `src/retrieval/failure_lookup.rs`.
- [x] Phase 9: Implement `src/retrieval/budget.rs` token truncation.
- [x] Phase 9: Implement `src/retrieval/cache.rs` SQLite cache.

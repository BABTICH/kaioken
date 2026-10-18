
# AI Coding Agent Blueprint

## Project Name

Working name: **KAIOKEN Agent**

## Vision

Build a Rust-based AI coding agent that is stronger than normal chat-based coding agents by adding a persistent codebase memory system, automatic documentation generation, and a coding/build agent on top.

The core idea is:

> The agent should continuously understand the repository before the user asks it to write code.

This system is inspired by the idea of a Hermes-like multi-layer agent, but the first real target is a **coding agent**.

The system should be able to:

- Scan the full codebase.
- Build structured memory about files, symbols, modules, dependencies, and changes.
- Trigger after every commit and update memory incrementally.
- Optionally run daily deep scans.
- Generate deep documentation in markdown, mirroring the source tree.
- Use that memory to plan, edit, test, debug, and build code.
- Become better over time by remembering failures, decisions, and repo structure.

---

# 1. Main Goal

The goal is not just:

> Make an LLM chatbot that edits files.

The goal is:

> Make a repo-aware software engineering agent.

The agent should understand:

```text
What files exist
What each file does
What functions/classes exist
What depends on what
What changed recently
What is risky
What failed before
What tests exist
What decisions were made
What documentation is stale
What should be updated after a change
```

This is how the system can compete with Claude Code / Codex style tools.

---

# 2. High-Level Architecture

The system has multiple layers.

```text
┌────────────────────────────────────────────┐
│              User / CLI / IDE              │
└────────────────────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────┐
│             Agent Runtime Layer            │
│                                            │
│  - task planning                           │
│  - tool execution                          │
│  - code editing                            │
│  - bash execution                          │
│  - test running                            │
│  - self-correction                         │
└────────────────────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────┐
│             Context Retrieval              │
│                                            │
│  - repo memory query                       │
│  - symbol lookup                           │
│  - dependency graph                        │
│  - doc lookup                              │
│  - git history lookup                      │
│  - failure memory lookup                   │
└────────────────────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────┐
│              Memory Engine                 │
│                                            │
│  - file memory                             │
│  - symbol memory                           │
│  - dependency graph                        │
│  - change memory                           │
│  - failure memory                          │
│  - decision memory                         │
└────────────────────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────┐
│           Documentation Engine             │
│                                            │
│  - file docs                               │
│  - class docs                              │
│  - function docs                           │
│  - module docs                             │
│  - architecture docs                       │
└────────────────────────────────────────────┘
                     │
                     ▼
┌────────────────────────────────────────────┐
│            Codebase Understanding          │
│                                            │
│  - tree-sitter AST                         │
│  - git diff                                │
│  - LSP diagnostics                         │
│  - test runner                             │
│  - build runner                            │
│  - dependency analysis                     │
└────────────────────────────────────────────┘
```

---

# 3. Core Layers

## Layer 0: Trigger Layer

This layer decides when the system runs.

### Triggers

The system should run when:

```text
1. A new commit happens
2. A push happens
3. A pull request is opened or updated
4. A daily schedule runs
5. The user manually runs a scan
6. Optional: files are saved/watched
```

### Recommended MVP Trigger

For the MVP, use:

```text
post-commit git hook
manual CLI command
```

Example:

```bash
agent memory scan --changed
```

Later add:

```bash
agent watch
agent schedule --daily
agent ci-hook
```

### Post-Commit Hook Behavior

After every commit:

```text
1. Detect current commit hash
2. Get changed files
3. Get diff
4. Parse changed files
5. Update memory
6. Generate commit understanding
7. Mark stale documentation
8. Optionally regenerate docs for changed files
```

### Daily Scan Behavior

Daily scan should be deeper:

```text
1. Scan whole repository
2. Rebuild stale parts of memory
3. Find missing documentation
4. Find dead code
5. Find risky modules
6. Find duplicate logic
7. Update architecture summary
8. Update dependency graph
9. Detect stale docs
10. Detect broken doc links
```

---

## Layer 1: Codebase Scanner Layer

This layer reads and understands the repository.

It should not rely only on LLMs.

It should use deterministic tools first.

### Scanner Inputs

```text
File system
Git history
AST parser
Language metadata
Dependency manifests
Test files
Build configs
CI configs
README/docs
```

### Scanner Responsibilities

```text
Detect files
Detect languages
Parse AST
Extract symbols
Extract imports/exports
Detect dependencies
Detect test files
Detect config files
Detect changed files from git diff
Detect risky files
Detect stale documentation
```

### Recommended Tools

| Job | Tool |
|---|---|
| AST parsing | Tree-sitter |
| Git operations | git2 or shell git |
| Code search | ripgrep |
| File search | fd or glob |
| Database | SQLite |
| Async runtime | tokio |
| LLM | OpenRouter via async-openai |
| Diff generation | similar |

### Languages To Support First

Start with one or two languages only.

Recommended:

```text
Rust
TypeScript/JavaScript
Python
```

Do not try to support every language immediately.

### Scanner Output

The scanner should output structured data.

Example:

```json
{
  "path": "src/auth/session.rs",
  "language": "rust",
  "size": 4200,
  "symbols": [
    {
      "name": "validate_session",
      "kind": "function",
      "line_start": 45,
      "line_end": 82,
      "visibility": "public"
    }
  ],
  "imports": [
    "jsonwebtoken",
    "sqlx",
    "crate::models::User"
  ],
  "last_commit": "abc123",
  "last_modified": "2026-06-14"
}
```

---

## Layer 2: Memory Engine Layer

This is the most important layer.

The memory engine stores structured knowledge about the repository.

It should not be only embeddings.

It should use:

```text
Structured memory + semantic memory
```

### Memory Types

The memory engine should store:

```text
File Memory
Symbol Memory
Dependency Graph Memory
Change Memory
Failure Memory
Decision Memory
Documentation Memory
```

---

# 4. Memory Types Explained

## 4.1 File Memory

For every file, store:

```text
path
language
size
last_commit
last_modified
purpose
exports
imports
related_files
risk_level
test_coverage
doc_status
```

Example:

```md
File: src/auth/session.rs
Language: Rust
Purpose: Handles user session creation and validation
Exports: create_session, validate_session, expire_session
Imports: jsonwebtoken, sqlx, crate::models::User
Related: src/auth/mod.rs, src/models/user.rs
Risk: High
Tests: tests/auth_session_test.rs
Last Changed: 2026-06-14
```

---

## 4.2 Symbol Memory

For every important symbol:

```text
symbol_name
file_path
line_start
line_end
kind
visibility
parameters
return_type
called_by
calls
tests
recent_changes
summary
```

Example:

```md
Symbol: validate_session
File: src/auth/session.rs
Kind: function
Visibility: public
Called by:
  - src/api/middleware.rs
  - src/api/routes/profile.rs
Calls:
  - decode_jwt
  - get_user_from_db
Summary:
  Validates a session token and returns the authenticated user.
```

---

## 4.3 Dependency Graph Memory

Store relationships:

```text
File A imports File B
Function X calls Function Y
Module M depends on Module N
Test T covers Symbol Z
Route R uses Middleware M
```

This allows impact analysis.

Example:

```text
If validate_session changes:
  - auth_middleware may break
  - profile_route may break
  - session tests should run
```

---

## 4.4 Change Memory

For every commit, store:

```text
commit hash
author
message
changed files
changed symbols
impact
risk
recommended tests
summary
```

Example:

```md
Commit: abc123
Message: fix session expiry bug

Changed files:
- src/auth/session.rs
- tests/auth_session_test.rs

Affected symbols:
- validate_session
- expire_session

Impact:
- Auth middleware may be affected
- Session expiry now tested

Risk:
Medium
```

---

## 4.5 Failure Memory

Store previous failures:

```text
Compiler errors
Test failures
Runtime errors
CI failures
Bugs fixed before
Broken patterns
Bad refactors
```

Example:

```md
Failure: TypeError in validateToken
File: src/auth/session.ts
Cause: token was undefined when request cookie missing
Fix: added null check and fallback session extractor
Related PR: #42
```

This allows the agent to say:

```text
This error looks similar to a previous auth token failure.
Last time it was caused by missing cookie handling.
```

---

## 4.6 Decision Memory

Store architectural decisions.

Example:

```md
Decision: Use JWT for session tokens
Reason: Stateless auth needed for API scalability
Alternatives considered:
- server sessions
- OAuth only
Consequences:
- token revocation is harder
- need refresh token rotation
```

This prevents the agent from suggesting changes that violate project decisions.

---

## 4.7 Documentation Memory

Track documentation state:

```text
doc path
source path
status
updated_at
stale_reason
coverage_level
```

Example:

```md
Doc: .agent-docs/src/auth/session.md
Source: src/auth/session.rs
Status: stale
Reason: validate_session signature changed
Updated At: 2026-06-10
```

---

# 5. Documentation Engine Layer

This is the optional but very powerful second layer.

The goal is:

> When you open a source file, you can also find a matching markdown file with deep documentation.

Example:

```text
src/auth/session.rs
```

has:

```text
.agent-docs/src/auth/session.md
```

---

## Documentation Folder Structure

Mirror the source tree.

Example source tree:

```text
src/
  auth/
    session.rs
    middleware.rs
  api/
    routes/
      user.rs
```

Documentation tree:

```text
.agent-docs/
  src/
    auth/
      session.md
      middleware.md
    api/
      routes/
        user.md
```

---

## File Documentation Template

```md
# src/auth/session.rs

## Purpose

Handles session creation, validation, and expiry.

## Public API

- create_session
- validate_session
- expire_session

## Dependencies

- jsonwebtoken
- sqlx
- crate::models::User

## Used By

- src/api/middleware.rs
- src/api/routes/profile.rs

## Tests

- tests/auth_session_test.rs

## Recent Changes

- 2026-06-14: fixed session expiry bug

## Risks

- Token validation depends on correct clock time
- Missing test for expired refresh token

## Notes

This module should not directly access database models except through repository layer.
```

---

## Function Documentation Template

```md
## validate_session

Location: src/auth/session.rs:45

Signature:

pub async fn validate_session(token: &str) -> Result<User, AuthError>

Purpose:

Validates a session token and returns the authenticated user.

Calls:

- decode_jwt
- get_user_from_db

Called by:

- auth_middleware
- profile_route

Tests:

- tests/auth_session_test.rs::test_valid_session

Known issues:

- Does not handle expired refresh tokens separately
```

---

# 6. Documentation Rules

Do not blindly regenerate all documentation all the time.

That becomes expensive and noisy.

Use incremental documentation.

## Documentation Levels

### Level 0: Raw Facts

Generated from AST/git.

No LLM needed.

```text
file path
symbols
imports
exports
line numbers
```

### Level 1: Basic Summary

Cheap LLM.

```text
This file handles session validation.
```

### Level 2: Detailed Explanation

Stronger LLM.

```text
This module manages authentication sessions using JWT tokens.
It validates tokens, loads users from the database, and returns typed auth errors.
```

### Level 3: Deep Documentation

Generated for important files only.

```text
purpose
public API
dependencies
risks
tests
known issues
architecture notes
recent changes
```

---

## Documentation Priority

Generate deep documentation first for:

```text
Core modules
Recently changed files
High-risk files
Public APIs
Frequently edited files
Files with no docs
Files involved in failures
```

Do not generate Level 3 docs for every file immediately.

---

# 7. Retrieval Engine Layer

This layer gives the coding agent the right context.

It answers questions like:

```text
What files are related to login?
What functions call validate_session?
What changed in the last 10 commits?
What docs exist for auth?
What failures happened in this module before?
What tests cover this function?
What depends on this file?
```

### Retrieval Sources

```text
File memory
Symbol memory
Dependency graph
Documentation
Git history
Failure memory
Decision memory
Embeddings
Grep results
AST search
```

### Retrieval Priority

For coding tasks, use this order:

```text
1. Exact file path
2. Symbol name
3. Grep search
4. Dependency graph
5. Related tests
6. Documentation
7. Git history
8. Semantic search
```

Important:

> For code, AST + grep + dependency graph is usually better than vector search alone.

Use embeddings for:

```text
Conceptual questions
Finding similar failures
Finding related modules
Searching documentation
Searching commit summaries
```

---

# 8. Coding Agent Layer

This layer performs actual development work.

It uses memory and docs as context.

### Agent Capabilities

```text
Implement features
Fix bugs
Refactor code
Write tests
Explain code
Review changes
Run builds
Run tests
Update docs
Update memory
Plan architecture
```

### Agent Tools

The agent should have tools such as:

```text
read_file
edit_file
create_file
list_files
glob_search
grep_search
find_symbol
find_references
run_bash
run_tests
run_linter
git_status
git_diff
git_commit
update_memory
generate_docs
```

---

# 9. Agent Runtime Loop

Use a ReAct + Deterministic Critic loop.

```text
User gives task
   ↓
Agent retrieves memory
   ↓
Agent reads docs/code
   ↓
Agent creates plan
   ↓
Agent chooses tool
   ↓
Rust executes tool
   ↓
Rust validates result
   ↓
If compiler/test fails, send error back
   ↓
Agent fixes issue
   ↓
Repeat until success or stop condition
```

### Example Loop

```text
Task:
Fix expired token bug in login.

Step 1:
Query memory for login, token, session.

Step 2:
Read src/auth/session.rs and src/api/routes/login.rs.

Step 3:
Read docs:
.agent-docs/src/auth/session.md

Step 4:
Plan:
- Handle expired token error
- Add fallback response
- Add test

Step 5:
Edit file.

Step 6:
Run cargo check.

Step 7:
If error, send compiler error back to agent.

Step 8:
Fix error.

Step 9:
Run tests.

Step 10:
Update memory and docs.
```

---

# 10. Deterministic Critic

This is very important.

Do not only trust the LLM.

After code changes, run deterministic checks.

For Rust:

```bash
cargo check
cargo clippy
cargo test
```

For TypeScript:

```bash
tsc --noEmit
eslint
```

For Python:

```bash
ruff check
mypy
pytest
```

Then feed errors back to the agent automatically.

Example:

```text
Tool result:
File edited successfully.

Compiler result:
error[E0308]: mismatched types

Agent instruction:
Fix this compiler error before continuing.
```

This makes the agent much more reliable.

---

# 11. Memory Update Flow After Commit

This is the main automated loop.

```text
git commit
   ↓
post-commit hook
   ↓
get changed files
   ↓
parse diff
   ↓
parse changed files with tree-sitter
   ↓
extract symbols
   ↓
update file memory
   ↓
update symbol memory
   ↓
update dependency graph
   ↓
detect affected symbols/files
   ↓
generate commit summary
   ↓
mark stale docs
   ↓
update high-priority docs
   ↓
update embeddings if needed
```

---

# 12. Daily Scan Flow

```text
daily schedule
   ↓
scan repository
   ↓
validate memory integrity
   ↓
find stale docs
   ↓
find missing docs
   ↓
find dead code
   ↓
find duplicated logic
   ↓
find risky modules
   ↓
find missing tests
   ↓
update architecture summary
   ↓
write daily report
```

Example daily report:

```md
# Daily Repo Report

## Stale Docs
- src/auth/session.rs
- src/api/routes/login.rs

## Missing Tests
- src/auth/refresh.rs

## High Churn Files
- src/api/middleware.rs

## Potential Risks
- validate_session has no test for expired tokens
- login route handles errors inconsistently

## Recommended Tasks
- Add test for expired refresh token
- Regenerate auth docs
- Normalize API error responses
```

---

# 13. Recommended Tech Stack

## Language

```text
Rust
```

## Core Rust Crates

```text
tokio
clap
serde
serde_json
anyhow
thiserror
tracing
```

## LLM

```text
OpenRouter
async-openai
reqwest
eventsource-stream
```

## Code Understanding

```text
tree-sitter
tree-sitter-rust
tree-sitter-typescript
tree-sitter-python
tree-sitter-javascript
```

## Git

```text
git2
```

## Database

```text
rusqlite
```

Later:

```text
sqlx
duckdb
vector database
```

## Search

```text
ripgrep
grep-regex
grep-searcher
```

## Diff

```text
similar
```

## Terminal UI

```text
ratatui
crossterm
```

---

# 14. Recommended Folder Structure

Agent source code:

```text
repo-mind-agent/
  Cargo.toml
  src/
    main.rs
    cli/
      mod.rs
      memory.rs
      docs.rs
      task.rs
    memory/
      mod.rs
      db.rs
      file_memory.rs
      symbol_memory.rs
      commit_memory.rs
      graph.rs
      failure_memory.rs
      decision_memory.rs
    docs/
      mod.rs
      generator.rs
      markdown.rs
      stale.rs
    scanner/
      mod.rs
      git.rs
      diff.rs
      tree_sitter.rs
      languages.rs
      file_inventory.rs
    retrieval/
      mod.rs
      query.rs
      graph.rs
      embeddings.rs
    agent/
      mod.rs
      loop.rs
      planner.rs
      tools/
        mod.rs
        read_file.rs
        edit_file.rs
        create_file.rs
        bash.rs
        grep.rs
        glob.rs
        git.rs
        tests.rs
    llm/
      mod.rs
      openrouter.rs
      prompts.rs
      models.rs
```

Inside the target repository:

```text
target-repo/
  .agent/
    config.toml
    memory.sqlite
    cache/
    logs/
  .agent-docs/
    src/
      auth/
        session.md
```

---

# 15. Database Schema

Start with SQLite.

## files

```sql
CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  language TEXT,
  size INTEGER,
  last_commit TEXT,
  last_modified TEXT,
  summary TEXT,
  risk_level TEXT,
  doc_path TEXT,
  doc_status TEXT
);
```

## symbols

```sql
CREATE TABLE symbols (
  id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  line_start INTEGER,
  line_end INTEGER,
  visibility TEXT,
  signature TEXT,
  summary TEXT,
  FOREIGN KEY(file_id) REFERENCES files(id)
);
```

## edges

```sql
CREATE TABLE edges (
  id INTEGER PRIMARY KEY,
  source_symbol_id INTEGER,
  target_symbol_id INTEGER,
  edge_type TEXT
);
```

## commits

```sql
CREATE TABLE commits (
  id INTEGER PRIMARY KEY,
  hash TEXT UNIQUE NOT NULL,
  message TEXT,
  author TEXT,
  created_at TEXT,
  summary TEXT,
  risk_level TEXT
);
```

## commit_changes

```sql
CREATE TABLE commit_changes (
  id INTEGER PRIMARY KEY,
  commit_id INTEGER NOT NULL,
  file_id INTEGER,
  symbol_id INTEGER,
  change_type TEXT
);
```

## docs

```sql
CREATE TABLE docs (
  id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL,
  doc_path TEXT NOT NULL,
  status TEXT,
  updated_at TEXT,
  stale_reason TEXT
);
```

## failures

```sql
CREATE TABLE failures (
  id INTEGER PRIMARY KEY,
  error_type TEXT,
  message TEXT,
  file_id INTEGER,
  symbol_id INTEGER,
  fix_summary TEXT,
  created_at TEXT
);
```

## decisions

```sql
CREATE TABLE decisions (
  id INTEGER PRIMARY KEY,
  title TEXT,
  description TEXT,
  status TEXT,
  created_at TEXT
);
```

---

# 16. Commands To Build

## Memory Commands

```bash
agent memory init
agent memory scan
agent memory scan --changed
agent memory status
agent memory explain <path>
agent memory impact <path>
agent memory recent
```

## Docs Commands

```bash
agent docs generate <path>
agent docs generate --changed
agent docs generate --stale
agent docs verify
```

## Agent Commands

```bash
agent task "fix login bug"
agent plan "add rate limiting"
agent review
agent test
agent build
```

## Watch/Scheduler Commands

```bash
agent watch
agent schedule --daily
agent hook install
```

---

# 17. Build Order

Do not build everything at once.

Build in this exact order.

---

## Phase 1: Basic CLI Project

Goal:

Create the Rust CLI skeleton.

Deliverables:

```text
Cargo project
clap CLI
basic commands
logging with tracing
error handling with anyhow
```

Commands:

```bash
agent --help
agent memory init
agent memory status
```

Definition of done:

```text
The CLI runs.
Commands are parsed.
Basic config file can be created.
```

---

## Phase 2: Git Change Detection

Goal:

Detect changed files from the latest commit.

Deliverables:

```text
Get current commit hash
Get changed files
Get diff
Print changed files
```

Definition of done:

```text
Running agent memory scan --changed shows files changed in last commit.
```

---

## Phase 3: File Scanner

Goal:

Scan files and detect language/metadata.

Deliverables:

```text
File inventory
Language detection
File size
Last modified
Last commit
```

Definition of done:

```text
Agent can list repository files with metadata.
```

---

## Phase 4: AST Symbol Extraction

Goal:

Parse code using Tree-sitter.

Deliverables:

```text
Extract functions
Extract classes
Extract structs
Extract methods
Extract imports/exports where possible
```

Definition of done:

```text
Agent can extract symbols from supported files.
```

---

## Phase 5: SQLite Memory Storage

Goal:

Store scanner output in SQLite.

Deliverables:

```text
files table
symbols table
commits table
commit_changes table
```

Definition of done:

```text
Scanning a repo writes structured memory into SQLite.
```

---

## Phase 6: Commit Memory

Goal:

Generate a commit understanding after each commit.

Deliverables:

```text
Commit summary
Changed files
Changed symbols
Affected files
Risk estimate
Recommended tests
```

Definition of done:

```text
After commit, agent produces a markdown commit memory and saves it to DB.
```

Example output:

```md
## Commit Understanding

Commit: a1b2c3d
Message: refactor session validation

Changed Files:
- src/auth/session.rs
- src/api/middleware.rs

Changed Symbols:
- validate_session
- auth_middleware

Impact:
- session validation now returns AuthError instead of bool
- middleware updated to handle new error type

Potential Risks:
- error handling may be incomplete in protected routes
```

---

## Phase 7: Documentation Mirror

Goal:

Generate markdown docs for source files.

Deliverables:

```text
.agent-docs folder
path-mirrored docs
file docs
symbol docs
```

Definition of done:

```text
For src/auth/session.rs, agent generates .agent-docs/src/auth/session.md.
```

---

## Phase 8: Incremental Doc Updates

Goal:

Avoid regenerating all docs.

Deliverables:

```text
Stale doc detection
Changed-file doc update
Doc freshness metadata
```

Definition of done:

```text
Only changed or affected docs are regenerated.
```

---

## Phase 9: Retrieval Layer

Goal:

Query memory before coding.

Deliverables:

```text
Query by file path
Query by symbol name
Query related files
Query recent changes
Query failures
```

Definition of done:

```text
Agent can retrieve relevant memory for a task.
```

---

## Phase 10: Basic Coding Agent

Goal:

Let the LLM edit code using tools.

Deliverables:

```text
OpenRouter client
streaming responses
tool calling
read_file
edit_file
grep_search
glob_search
bash tool
```

Definition of done:

```text
Agent can complete a simple task like adding a function or fixing a small bug.
```

---

## Phase 11: Deterministic Feedback

Goal:

Make the agent self-correct.

Deliverables:

```text
Run compiler after edit
Run linter after edit
Run tests after edit
Send failures back to agent
Retry limits
```

Definition of done:

```text
Agent can fix a compile error automatically.
```

---

## Phase 12: Agent Uses Memory

Goal:

Connect coding agent to memory engine.

Deliverables:

```text
Before task, query memory
Read related docs
Read related symbols
Use failure memory
Use dependency graph
```

Definition of done:

```text
Agent explains where it got context from and uses repo memory during tasks.
```

---

## Phase 13: Daily Scanner

Goal:

Run deeper analysis on schedule.

Deliverables:

```text
Daily repo report
Stale doc detection
Missing doc detection
Dead code hints
Missing test hints
Architecture summary
```

Definition of done:

```text
Daily scan produces a useful report.
```

---

## Phase 14: Safety and Permissions

Goal:

Prevent dangerous actions.

Deliverables:

```text
Permission levels
Dangerous command blocklist
File path restrictions
Secret filtering
Approval prompts
```

Definition of done:

```text
Agent cannot run destructive commands without explicit approval.
```

---

## Phase 15: Evaluation Suite

Goal:

Measure if the agent is actually good.

Deliverables:

```text
Task benchmark folder
success/failure tracking
token usage tracking
cost tracking
time tracking
edit validity tracking
test pass rate
```

Definition of done:

```text
Every major change to the agent can be tested against real tasks.
```

---

# 18. Configuration File

Use:

```text
.agent/config.toml
```

Example:

```toml
[project]
name = "my-project"
languages = ["rust", "typescript"]

[memory]
database = ".agent/memory.sqlite"
update_on_commit = true
daily_scan = true

[docs]
output = ".agent-docs"
auto_update_changed = true
deep_docs_for_high_risk = true

[llm]
provider = "openrouter"
planner_model = "anthropic/claude-sonnet-4"
coder_model = "anthropic/claude-sonnet-4"
summarizer_model = "openai/gpt-4o-mini"
max_tokens = 8000
temperature = 0.2

[permissions]
auto_edit = false
auto_bash = false
allowed_commands = [
  "cargo check",
  "cargo test",
  "npm test",
  "pytest",
]
blocked_commands = [
  "rm -rf",
  "git push --force",
  "git reset --hard",
]

[scanner]
ignore = [
  "node_modules",
  "target",
  "dist",
  "build",
  ".git",
]
```

---

# 19. Important Design Rules

## Rule 1: Code Is The Source Of Truth

Docs and memory are derived artifacts.

If docs disagree with code, trust code.

---

## Rule 2: Do Not Regenerate Everything

Use incremental updates.

Only regenerate:

```text
changed files
affected files
stale docs
high-priority files
```

---

## Rule 3: Use Deterministic Tools First

Before asking the LLM, use:

```text
AST
grep
git
compiler
tests
dependency graph
```

Use LLM for:

```text
explanation
planning
code generation
summarization
```

---

## Rule 4: Memory Is Context, Not Truth

Memory can be stale.

Always verify with:

```text
current code
compiler
tests
git status
```

---

## Rule 5: Keep Context Small

Do not dump the whole repo into the prompt.

Send:

```text
relevant files
relevant symbols
recent errors
dependency info
docs summaries
```

---

## Rule 6: Prefer Search-Replace Edits

Do not rewrite whole files unless necessary.

Use:

```text
old_text
new_text
```

with exact validation.

---

## Rule 7: Every Edit Must Be Verifiable

After editing, run:

```text
compiler
linter
tests
syntax check
```

---

## Rule 8: Track Failures

Failure memory is one of the biggest advantages.

Store:

```text
error message
root cause
fix
related files
related symbols
```

---

# 20. How This Beats Claude Code / Codex

This system can win in these areas:

## 1. Long-Term Repo Memory

Claude Code usually starts with limited session memory.

Your agent can remember:

```text
project structure
past changes
past bugs
past decisions
```

---

## 2. Change Impact Analysis

Your agent can say:

```text
If you change this function, these files may break.
```

---

## 3. Documentation That Lives With Code

Your agent can provide:

```text
file-by-file
class-by-class
function-by-function
```

documentation in predictable paths.

---

## 4. Project-Specific Learning

Your agent can learn:

```text
This repo uses this error pattern.
This repo avoids this dependency.
This module is unstable.
This test always fails in CI.
```

---

## 5. Better Context Selection

Instead of guessing, the agent uses:

```text
memory
docs
graph
history
failures
```

---

# 21. Risks And Solutions

## Risk 1: Docs Become Stale

Solution:

```text
Mark docs stale automatically.
Regenerate only when needed.
Show doc freshness timestamp.
```

---

## Risk 2: Too Expensive

Solution:

```text
Use cheap models for summaries.
Use strong models only for planning/editing.
Update only changed files.
Cache aggressively.
```

---

## Risk 3: Hallucinated Documentation

Solution:

```text
Generate docs from AST facts first.
LLM only explains facts.
Do not invent APIs.
Validate symbol names against tree-sitter.
```

---

## Risk 4: Memory Becomes Too Large

Solution:

```text
Summarize.
Prune old events.
Store raw logs separately.
Keep active memory small.
```

---

## Risk 5: Agent Trusts Bad Memory

Solution:

```text
Memory is context, not truth.
Always verify with code/compiler/tests.
```

---

## Risk 6: Dangerous Commands

Solution:

```text
Permission system.
Blocked command list.
Approval prompts.
Sandbox for bash.
```

---

# 22. Evaluation Metrics

Track these metrics:

```text
Task success rate
Edit validity rate
Compiler pass rate
Test pass rate
Average steps per task
Average tokens per task
Average cost per task
Average time per task
Human interventions required
Number of corrupted files
Number of stale docs
Doc freshness percentage
Memory retrieval accuracy
```

---

# 23. Definition Of MVP

The MVP is complete when:

```text
1. The CLI can scan a repository.
2. It stores file and symbol memory in SQLite.
3. It detects changed files after a commit.
4. It generates a commit understanding.
5. It generates markdown docs for changed files.
6. It can answer questions using repo memory.
7. It can perform basic code edits.
8. It can run compiler/tests and feed errors back.
9. It updates memory after successful changes.
```

---

# 24. Definition Of Advanced Version

The advanced version is complete when:

```text
1. It has full dependency graph memory.
2. It has failure memory and decision memory.
3. It has daily deep scanning.
4. It has path-mirrored deep documentation.
5. It has LSP integration.
6. It has cross-file refactoring.
7. It has permission/safety system.
8. It has session persistence.
9. It has benchmark eval suite.
10. It can autonomously fix bugs and pass tests.
```

---

# 25. Final Recommended Order

Build in this order:

```text
1. Rust CLI skeleton
2. Git change detection
3. File scanner
4. Tree-sitter symbol extraction
5. SQLite memory storage
6. Commit memory generation
7. Documentation mirror
8. Incremental doc updates
9. Retrieval/query layer
10. Basic ReAct coding agent
11. Deterministic compiler/test feedback
12. Agent uses memory/docs
13. Daily scanner/report
14. Safety/permissions
15. Eval suite
```

---

# 26. First Practical Milestone

Your first real milestone should be:

```text
After every commit, generate a structured commit understanding and update file/symbol memory.
```

Do not build the full coding agent first.

Build the memory engine first.

If the memory engine is good, the coding agent becomes much stronger.

If the memory engine is bad, the coding agent will just be another unstable LLM wrapper.

---

# 27. Core Principle

The core principle of the whole project is:

> Build a system that understands the repository continuously, documents it intelligently, and uses that understanding to make coding actions safer and more accurate.

That is the path to building something that can genuinely compete with Claude Code / Codex.
```
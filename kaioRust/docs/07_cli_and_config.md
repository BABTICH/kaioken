# 07. CLI & Configuration

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 16 & Section 18

---

## 1. Global CLI Flags

The CLI supports global flags across all subcommands:
- `--json`: Output results formatted as structured JSON for automation or IDE integration.
- `--dry-run`: Plan tasks, simulate diffs, and inspect tools without writing changes to disk.
- `--yes` (`-y`): Bypass interactive approval prompts for CI/CD or non-interactive environments.
- `-v, --verbose`: Increase logging output verbosity (`-v` info, `-vv` debug, `-vvv` trace).

---

## 2. CLI Commands Tree

```text
kaioken
├── doctor              # Diagnose environment, SQLite, Git, Tree-sitter, API keys
├── memory
│   ├── init            # Initialize .agent/ folder, config.toml, and database
│   ├── status          # Display active configuration and database readiness
│   ├── scan            # Scan repository and extract structured metadata
│   │   ├── --changed   # Scan only files modified in latest commit
│   │   └── --no-llm    # Deterministic scan only (zero API cost)
│   ├── query <target>  # Query files, symbols, and edges directly from SQLite
│   ├── explain <path>  # Explain file architecture from memory
│   ├── impact <target> # Analyze dependency blast radius of symbol or file
│   └── recent          # Display recent commit memories
├── retrieval
│   └── context <task>  # Generate and print the ContextPacket for a task
├── docs
│   ├── generate [path] # Generate markdown docs into .agent-docs/
│   │   ├── --changed   # Regenerate docs for changed files only
│   │   └── --stale     # Regenerate docs marked as stale
│   └── verify          # Audit doc freshness and detect broken links/orphans
├── task <prompt>       # Execute autonomous coding task with ReAct loop
├── plan <prompt>       # Generate execution plan without editing code
├── review              # Review recent git diff against repository memory
├── test                # Run deterministic test critic
├── build               # Run deterministic compiler check
├── watch               # Start filesystem watcher daemon
├── schedule --daily    # Run scheduled deep repository maintenance
├── audit
│   └── show            # Display recent security and tool audit logs
├── eval
│   └── run             # Execute benchmark evaluation suite
└── hook
    └── install         # Install git post-commit hook
```

---

## 3. Production Configuration Schema (`.agent/config.toml`)

```toml
[project]
name = "kaioken-project"
languages = ["rust", "typescript"]

[limits]
max_iterations = 20
max_tool_calls = 100
max_command_timeout_seconds = 120
max_tool_output_bytes = 100000
max_context_tokens = 8000

[retrieval]
max_files = 20
max_symbols = 50
max_failures = 10
max_docs = 10
graph_depth = 1
use_vector_fallback = false

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

[safety]
mode = "edit-with-approval"
restrict_to_workspace = true
block_path_traversal = true
redact_secrets = true
audit_log = ".agent/logs/audit.log"

[permissions]
require_approval_for_bash = true
allowed_read_commands = [
  "git status",
  "git diff",
  "git log",
]
allowed_test_commands = [
  "cargo check",
  "cargo test",
  "npm test",
  "pytest",
]
blocked_commands = [
  "rm -rf",
  "rmdir /s",
  "git push --force",
  "git reset --hard",
  "git clean -fd",
  "drop table",
  "curl * | bash",
  "wget * | bash",
  "npm publish",
  "cargo publish",
]

[eval]
output_dir = ".agent/eval"
run_smoke_tests = true

[logging]
level = "info"
directory = ".agent/logs"

[scanner]
max_file_size_bytes = 1048576  # 1 MB threshold
ignore = [
  "node_modules",
  "target",
  "dist",
  "build",
  ".git",
  ".agent",
]
```

---

## 4. Current Implementation Status

- [x] Basic `Config` serialization in `src/config/mod.rs`.
- [x] `kaioken memory init` and `kaioken memory status` operational in Phase 1.
- [ ] Phase 2: Implement `--no-llm` flag and `memory scan --changed`.
- [ ] Phase 9: Implement `kaioken retrieval context <task>`.
- [ ] Phase 10: Implement `kaioken doctor` and `kaioken audit show`.
- [ ] Phase 15: Implement `kaioken eval run`.

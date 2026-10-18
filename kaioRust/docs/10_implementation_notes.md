# 10. Implementation Notes & Technical Stack

> **Reference**: [AGENT_BLUEPRINT.md](file:///d:/project/ai_now_know/AGENT_BLUEPRINT.md) - Section 13 & Section 14

This document defines the recommended Rust crates, technical dependencies, and architectural implementation practices for building KAIOKEN Agent.

---

## 1. Recommended Rust Crates by Domain

### Core Foundation & Runtime
- **`tokio`**: Multi-threaded async runtime with `full` feature set for concurrent task execution, timeouts, and process management.
- **`clap`**: Declarative CLI parsing using `derive` and environment variable overrides.
- **`serde` & `serde_json`**: Fast data serialization and deserialization for tool arguments and JSON outputs.
- **`toml`**: Strongly typed parser and serializer for `.agent/config.toml`.
- **`anyhow` & `thiserror`**: Production error handling (`thiserror` for structured domain errors in library, `anyhow` for top-level CLI application binaries).
- **`tracing` & `tracing-subscriber`**: Structured diagnostic telemetry, log filtering, and per-tool execution tracing.

### Code Understanding & AST
- **`tree-sitter`**: Incremental parser generating concrete syntax trees with robust error recovery.
- **`tree-sitter-rust`**: Rust grammar support.
- **`tree-sitter-typescript`**: TypeScript / TSX grammar support.
- **`tree-sitter-python`**: Python grammar support.
- **`tree-sitter-javascript`**: JavaScript / JSX grammar support.

### Git Operations
- **`git2`**: Native libgit2 bindings for high-performance commit inspection, diff hunk extraction, and status queries without spawning shell subprocesses.

### Storage & Database
- **`rusqlite`**: Embedded SQLite driver with `bundled` feature. Preferred for zero-dependency embedded deployment, fast query execution, WAL concurrency, and offline operation.

### Search & Text Analysis
- **`grep-regex` & `grep-searcher`**: Fast in-process ripgrep engine components.
- Alternative: Shell out directly to installed `rg` binary with structured JSON streaming.

### Diff Calculation
- **`similar`**: High-performance character, word, and unified line diff generator for generating search-and-replace patches and validation.

### LLM Communication & Streaming
- **`async-openai`**: OpenAI-compatible client library configured with OpenRouter base URL (`https://openrouter.ai/api/v1`).
- **`reqwest`**: Low-level async HTTP client for custom header forwarding and proxy support.
- **`eventsource-stream`**: Server-Sent Events (SSE) streaming parser for real-time token rendering.

### Terminal User Interface (TUI)
- **`ratatui` & `crossterm`**: Terminal interface framework for interactive diff viewing, approval dialogs, and progress bars.

---

## 2. Architectural Implementation Guidance

### 1. OpenAI-Compatible OpenRouter Client
- OpenRouter supports the standard OpenAI `/chat/completions` REST interface.
- Set base URL to `https://openrouter.ai/api/v1`.
- Forward standard headers: `HTTP-Referer: https://kaioken.dev` and `X-Title: KAIOKEN Agent`.
- Support model routing:
  - Planner & Coder: `anthropic/claude-sonnet-4` or `anthropic/claude-3.7-sonnet`
  - Summarizer: `openai/gpt-4o-mini` or `anthropic/claude-3.5-haiku`

### 2. Structured Tool Call Schemas
- Define tool schemas using standard JSON Schema definitions.
- Tools must parse arguments into strongly typed Rust structs with default fallbacks.
- Never allow unvalidated arbitrary strings directly into shell executions.

### 3. Streaming and Progress Observability
- Stream tool and reasoning tokens where possible to eliminate latency perception.
- Show active step indicators: `[1/3] Parsing AST...`, `[2/3] Querying memory...`, `[3/3] Compiling...`.

### 4. Exponential Backoff & Transient Fault Handling
- Network calls to LLM APIs must implement exponential backoff (e.g. 1s, 2s, 4s, 8s) on HTTP 429 (Rate Limit) and 5xx server errors.
- Never crash the agent loop due to a temporary network blip.

### 5. Cost Tracking & SQLite Persistence
- Extract token counts (`prompt_tokens`, `completion_tokens`) from API responses.
- Compute estimated cost based on active model pricing.
- Persist every agent session into the `agent_runs` table and individual tool latencies into `tool_calls`.

### 6. Strict Independence of Deterministic Layers
- Layer 1 (Scanner), Layer 2 (Memory), and Layer 4 (Retrieval) must run completely independently of LLM availability.
- A developer without internet access or an API key can still scan repositories, inspect symbols, query dependency graphs, and verify docs locally.

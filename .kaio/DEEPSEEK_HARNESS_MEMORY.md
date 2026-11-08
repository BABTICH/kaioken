# DeepSeek Harness (DSH) Architecture Memory & Kaioken Blueprint

This document captures the complete architectural analysis of **DeepSeek Harness (`dsh`)** located at [.kaio/deepseek-harness](deepseek-harness) and provides the integration blueprint for building the **KAIOKEN Agent** on top of it.

---

## 1. Executive Summary & Philosophy

DeepSeek Harness is an enterprise-grade, extensible AI agent runtime created by DeepSeek AI. It is built on two non-negotiable principles:

1. **"Everything is a Plugin" (Powered by Cordis)**:
   - There is **no privileged core**. The agent loop, tool execution pipeline, prompt assembler, LLM adapters, session storage, and even the default model selector are all unprivileged plugins registered into a shared runtime container.
   - Features attach services (`ctx.<key>`), register typed events, and inject reversible effects (`ctx.effect()`, `ctx.on()`) that clean up cleanly when unmounted.
2. **"Model-Visible Means Logged" (Durable Event Sourcing)**:
   - The session log is the single source of truth. Any prompt string, tool output, system context, user message, or assistant chunk that reaches or comes from the model is recorded as an immutable event in an append-only JSONL log (`session.vN.jsonl`).
   - Context is folded over this event log, allowing deterministic pause, resume, replay, and migration.

---

## 2. Core Architecture: The Cordis Plugin Framework

DSH vendors Cordis under `@deepseek-ai/cordis` (in `vendor/`). 

### 2.1 The Context (`ctx`) & Dependency Injection
- Every plugin is an object implementing `apply(ctx: Context, config: Config)` or a subclass of `Service`.
- Plugins declare requirements via `inject: ['llm', 'tools', 'sessions']`. The Cordis loader delays mounting until all injected services are active.
- Services claim stable keys on `ctx` (e.g., `ctx.tools`, `ctx.llm`, `ctx.agentLoop`).

### 2.2 Event Dispatch Modes
Cordis provides five strict dispatch modes used throughout the agent loop:
| Mode | Behavior | Use Case in DSH |
| :--- | :--- | :--- |
| **`waterfall`** | Middleware onion `(...args, next)`. Listeners **must** call `next()` to delegate or return early to short-circuit. | `system-prompt/assemble`, `agent/pre-step`, `agent/request`, `llm/stream` |
| **`serial`** | Awaited sequential execution in registration order. | `agent/turn-stopping`, shutdown cleanup |
| **`parallel`** | `Promise.all` across all registered listeners concurrently. | Broad notification & async event fan-out |
| **`bail`** | Sequential execution stopping at the first non-`undefined` return. | Permission checks, title resolvers |
| **`emit`** | Fire-and-forget, non-awaited observation in registration order. | `session/event`, telemetry, live UI streaming |

### 2.3 Declarative Composition (Profiles & Bundles)
- **Profile**: A named top-level configuration (`web`, `headless`, `sdk`, `acp`) defined in `package.json` and a `cordis.patch.yml`.
- **Bundle**: A distributable collection of plugin configurations (e.g., `@deepseek-ai/dsh-base`).
- **Overlays & Patches**: `cordis.patch.yml` modifies rows by `id`, allowing surgical overrides of any service without modifying source code.

---

## 3. Agent Turn & Step Lifecycle

Execution separates **durable session events** (`session/event`) from **transient live coordination** (`agent/*`):

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant Inbox as Agent Inbox
    participant Loop as ctx.agentLoop
    participant Prompt as ctx.systemPrompt
    participant PreStep as agent/pre-step
    participant LLM as ctx.llm
    participant Tools as ctx.tools
    participant Log as ctx.sessions

    User->>Inbox: Post message (agent/inbox/inserted)
    Inbox->>Loop: Wake up -> turn/start
    loop Step Loop (until no tool calls or turn finishes)
        Loop->>Prompt: system-prompt/assemble (waterfall)
        Loop->>PreStep: agent/pre-step (waterfall: compaction, guards)
        Loop->>Loop: step/start -> agent/request (waterfall)
        Loop->>Log: Synchronous admission (system/message, user/message)
        Loop->>LLM: llm/stream (waterfall)
        LLM-->>Loop: Stream chunks -> agent/assistant-stream
        Loop->>Log: assistant/message
        opt Model emitted tool_calls
            Loop->>Tools: tools/pre-execute -> execute -> post-execute
            Tools->>Log: tool/result
        end
        Loop->>Loop: step/end
    end
    Loop->>Loop: agent/turn-stopping (serial) -> turn/end
```

---

## 4. Subsystem & Capability Seam Map

DSH strictly enforces the **Capability Seam Pattern**:
* **Service Definition**: Declares the abstract TypeScript interface.
* **Service Provider**: Concrete implementation registered onto `ctx.<key>`.
* **Consumer**: Uses the service via static typing.

```
packages/
├── core/
│   ├── agent/               -> ctx.agents (Agent handle, lifecycle bus)
│   ├── agent-loop/          -> ctx.agentLoop (Default turn/step driver)
│   ├── agent-default-model/ -> ctx.agentDefaultModel (Fallback model selector)
│   ├── session/             -> ctx.sessions (In-memory store + append-only session log)
│   ├── system-prompt/       -> ctx.systemPrompt (Prompt sections & schema assembly)
│   ├── tools/               -> ctx.tools (Tool registry & execution pipeline)
│   └── scope/               -> Isolation primitives for agent contexts
├── llm/
│   ├── llm/                 -> ctx.llm (Provider-neutral streaming seam)
│   ├── llm-deepseek/        -> Official DeepSeek API adapter
│   ├── llm-pi-ai/           -> Universal multi-model adapter (Claude, OpenAI, Ollama, etc.)
│   ├── llm-retry/           -> Step-boundary exponential retries
│   └── token-meter/         -> ctx.tokenMeter (Heuristic & API token accounting)
├── session/
│   ├── session-persistence/ -> ctx.sessionPersistence (Durable storage seam)
│   ├── session-persistence-jsonl/ -> JSONL generation files with Zstandard support
│   ├── session-projection/  -> ctx.sessionProjections (State folds over session events)
│   ├── session-title/       -> ctx.sessionTitle (Deterministic / LLM session titling)
│   └── session-format-*/    -> Versioned migration decoders (v0 -> v1 -> v2 -> v3 -> v4)
├── session-query/           -> ctx.sessionQuery (SQLite full-text search across sessions)
├── subagent/
│   ├── subagent/            -> ctx.subagents (Delegation registry & discovery)
│   ├── subagent-spawn-in-process/ -> Clean isolated child session
│   ├── subagent-fork-in-process/  -> Child seeded with parent conversation history
│   ├── subagent-acp/        -> Agent Client Protocol child runner
│   ├── subagent-dsh-sdk/    -> Out-of-process TS SDK child runner
│   └── tool-subagent/       -> Model-facing subagent tool
├── workflow/
│   ├── workflow/            -> ctx.workflowEngine (Scripted orchestration seam)
│   ├── workflow-ptc/        -> Sandboxed Node process runtime (Programmatic Tool Calling)
│   ├── tool-workflow/       -> Model-facing workflow orchestration tool
│   └── tool-ralph/          -> Model-facing iterative fresh-agent loop
├── skill/
│   ├── skill/               -> ctx.skills (Instruction catalog registry)
│   ├── skill-filesystem/    -> Discovers skills from repo / user dirs
│   └── tool-skill/          -> Model-facing skill loader tool
├── fs/                      -> ctx.fs (fs-local, fs-sandbox with write fencing)
├── shell/                   -> ctx.shell (bash on POSIX, pwsh on Win32)
├── subprocess/              -> ctx.subprocess (Process-tree tracking)
└── sandbox/                 -> ctx.sandbox (Windows ACL tokens / Linux bwrap / Seatbelt)
```

---

## 5. Blueprint: Rebuilding Kaioken on DeepSeek Harness

The Kaioken vision ([AGENT_BLUEPRINT.md](../AGENT_BLUEPRINT.md)) requires:
1. Continuous codebase understanding & structured repo memory (symbols, AST, dependencies, changes).
2. Auto-generated deep documentation mirroring the tree.
3. Proactive context injection before the model runs tasks.
4. Autonomous plan-edit-test-debug loops.

Here is how Kaioken maps cleanly onto DeepSeek Harness:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        KAIOKEN AGENT SUITE                             │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ @deepseek-ai/dsh-base (The Rock-Solid DSH Spine)                      │
│ - Cordis container, execution loop, durable JSONL session logging     │
│ - Sandboxed filesystem, PowerShell/Bash execution, model streaming    │
└────────────────────────────────────────────────────────────────────────┘
        │                           │                           │
        ▼                           ▼                           ▼
┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
│  Kaioken Memory  │        │ Kaioken Context  │        │ Kaioken Workflows│
│  Capability Seam │        │ Waterfall Hooks  │        │ & Subagents      │
│(ctx.kaiokenMemory)│       │ (Pre-Step Influx)│        │ (Specialists)    │
└──────────────────┘        └──────────────────┘        └──────────────────┘
```

### 5.1 Step 1: Create `@kaioken/memory` Service Seam
Create a Cordis Service defining the memory interface:
```typescript
import { Context, Service } from '@deepseek-ai/cordis';

export interface KaiokenMemoryService {
  searchSymbols(query: string): Promise<SymbolMatch[]>;
  getFileContext(path: string): Promise<FileMemoryContext>;
  getDependencyGraph(target: string): Promise<DependencyNode[]>;
  getFailureMemory(area: string): Promise<HistoricalFailure[]>;
  triggerIncrementalScan(changedFiles: string[]): Promise<void>;
}

export class KaiokenMemory extends Service {
  static inject = ['fs', 'subprocess'];
  constructor(ctx: Context) {
    super(ctx, 'kaiokenMemory', true);
  }
}
```
* Can be implemented via Rust FFI (using `@deepseek-ai/node-addon-system` or `napi-rs` pointing to `kaioRust`) or a SQLite vector/symbol database.

### 5.2 Step 2: Inject Context via `system-prompt/assemble` Waterfall
Instead of monkey-patching the agent loop, Kaioken registers a waterfall listener on prompt assembly:
```typescript
ctx.on('system-prompt/assemble', async (sections, next) => {
  const activeFiles = ctx.kaiokenMemory.getActiveContextFiles();
  const symbolContext = await ctx.kaiokenMemory.getRelevantSymbols(activeFiles);
  
  sections.push({
    id: 'kaioken-repo-intelligence',
    priority: 150,
    text: `## Repository Intelligence\n${symbolContext}`,
  });
  
  return next();
});
```

### 5.3 Step 3: Proactive Pre-Step Failure Memory Hook
Use `agent/pre-step` waterfall to inspect planned actions and inject warnings if touching risky files:
```typescript
ctx.on('agent/pre-step', async (stepState, next) => {
  const failures = await ctx.kaiokenMemory.getFailureMemory(stepState.currentFocus);
  if (failures.length > 0) {
    stepState.advisoryNotes.push(`Warning: Similar changes failed in past commits: ${failures[0].summary}`);
  }
  return next();
});
```

### 5.4 Step 4: Register Kaioken Model-Facing Tools
Expose tools via `ctx.tools`:
- `kaioken_query_memory`: Query symbol graphs, caller hierarchies, and module roles.
- `kaioken_inspect_docs`: Access generated repo wiki/markdown documentation.
- `kaioken_record_decision`: Store architectural decisions directly into repo memory.

### 5.5 Step 5: Continuous Repo Scanning via Background Jobs
Hook into `packages/jobs/` or git file watchers:
- On commit / file change: Fire a background task via `ctx.jobs` to update AST symbols and documentation without freezing the interactive conversation.

### 5.6 Step 6: Create the `@kaioken/bundle`
Create a custom profile patch (`kaioken.patch.yml`):
```yaml
- insert:
    - id: kaioken-memory
      name: '@kaioken/memory'
      config:
        dbPath: '.kaio/memory.db'
    - id: kaioken-tools
      name: '@kaioken/tools'
    - id: kaioken-prompt-enhancer
      name: '@kaioken/prompt-enhancer'
```
Mount it as a profile:
```sh
pnpm dsh --profile kaioken "Refactor auth middleware and update tests"
```

---

## 6. Key Takeaways & Recommendations

1. **Do not reinvent the harness**: DSH already handles sandboxing, process execution, cross-platform PTYs, event-sourced persistence, token management, and model streaming with 100% test coverage.
2. **Focus 100% of Kaioken on Repository Understanding**: Put all effort into the AST indexer, symbol graphs, persistent codebase memory, and automatic documentation generator.
3. **Plug cleanly into DSH seams**: Everything Kaioken needs can be achieved by adding 1 Service (`ctx.kaiokenMemory`), 2 Waterfall listeners (`system-prompt/assemble`, `agent/pre-step`), and 3 Tools (`ctx.tools`).

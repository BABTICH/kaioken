# DeepSeek Harness (DSH) — Kaioken Rebuild Memory

> **Purpose**: Complete architectural reference for rebuilding the Kaioken agent framework on top of `deepseek-harness`. Generated 2026-09-30.

---

## 1. What DeepSeek Harness Is

**DeepSeek Harness (`dsh`)** is an open-source, everything-is-a-plugin agent harness built by DeepSeek AI.

- **Philosophy**: _No privileged core._ Every major system (model adapter, tool registry, session log, agent loop, UI) is a Cordis plugin. Plugins are composable, reversible, and swappable from configuration.
- **Framework**: Built on a vendored copy of **Cordis** re-scoped to `@deepseek-ai/cordis`.
- **Distribution**: Published as `@deepseek-ai/dsh` on npm. Run with `npx @deepseek-ai/dsh web`.
- **Profiles**: Named runtime compositions: `web`, `headless`, `sdk`, `sdk-minimal`, `acp`.

---

## 2. Core Architecture — Cordis Plugin System

### How Cordis Works

A Cordis **Context** (`ctx`) is a shared service registry. Plugins declare what they need via `inject: [...]` and contribute services, events, and effects.

```typescript
class MyService extends Service {
  static inject = ['sessions', 'tools']
  constructor(ctx: Context) {
    super(ctx, 'myService')  // registers as ctx.myService
    ctx.effect(() => {
      return () => { /* cleanup disposer */ }
    })
  }
}
ctx.plugin(MyService)
```

### Cordis Event Dispatch Modes

| Mode | Behavior |
|------|----------|
| `emit` | Fire-and-forget, non-awaited |
| `waterfall` | Around-middleware — each listener wraps the chain, MUST call `next()` |
| `parallel` | All listeners awaited in parallel |
| `serial` | Listeners awaited sequentially |
| `bail` | Sequential until first non-`undefined` return |

> **Critical**: `waterfall` is the primary extension mechanism. `agent/request`, `system-prompt/assemble`, `llm/stream` are all waterfalls.

### Profile → Bundle → Plugin Composition

```
Profile (cordis.yml)
  └─ dsh-base (bundle)
       ├─ core/session, core/tools, core/agent, core/agent-loop
       ├─ llm/llm + llm-deepseek
       └─ ... more plugins
  └─ [Kaioken bundle]
       └─ kaioken-* plugins
```

---

## 3. Core Service Map (ctx.*)

| Service Key | Package | Role |
|---|---|---|
| `ctx.sessions` | `@deepseek-ai/dsh-session` | Append-only session event log, active session store |
| `ctx.systemPrompt` | `@deepseek-ai/dsh-system-prompt` | Collects prompt sections via waterfall |
| `ctx.tools` | `@deepseek-ai/dsh-tools` | Tool registration, scoped registry, guarded execution pipeline |
| `ctx.agents` | `@deepseek-ai/dsh-agent` | Agent interface, live agent registry, `agent/*` event bus |
| `ctx.agentLoop` | `@deepseek-ai/dsh-agent-loop` | Default driver implementing the Agent interface |
| `ctx.llm` | `@deepseek-ai/dsh-llm` | Provider-neutral streaming LLM interface |
| `ctx.sessionPersistence` | `@deepseek-ai/dsh-session-persistence` | Durable log storage seam |
| `ctx.subagents` | `@deepseek-ai/dsh-subagent` | Delegated agent runner registry |
| `ctx.workflowEngine` | `@deepseek-ai/dsh-workflow` | JS orchestration script executor |
| `ctx.skills` | `@deepseek-ai/dsh-skill` | Skill catalog/loader for task instructions |
| `ctx.fs` | `@deepseek-ai/dsh-fs` | Filesystem seam (local/sandbox/SSH backends) |
| `ctx.subprocess` | `@deepseek-ai/dsh-subprocess` | Child-process seam |
| `ctx.shell` | `@deepseek-ai/dsh-shell` | Bash/pwsh execution seam |
| `ctx.sandbox` | `@deepseek-ai/dsh-sandbox` | Process confinement (bwrap/Landlock/Seatbelt) |
| `ctx.sessionQuery` | `@deepseek-ai/dsh-session-query` | Session browsing, semantic search, FTS |
| `ctx.credentials` | `@deepseek-ai/dsh-credentials` | Credential reference + env/.env provider |
| `ctx.settings` | `@deepseek-ai/dsh-settings` | User settings seam |
| `ctx.web` | `@deepseek-ai/dsh-web` | Web search/fetch seam |
| `ctx.webhookRuntime` | `@deepseek-ai/dsh-webhook` | Verified external event ingress |

---

## 4. Agent Turn & Step Lifecycle

```
User Message → Agent Inbox → Driver wakes → turn/start
  ↓
[For each step]:
  system-prompt/assemble (waterfall)
  agent/pre-step (waterfall)          ← compaction hooks here
  step/start
  agent/request (waterfall)           ← request mutation hooks here
  llm/stream (waterfall)              ← model streaming
    └→ agent/assistant-stream         ← UI streaming events
  assistant/message (logged to session)
  [For each tool call]:
    tool/call
    tools/pre-execute (waterfall)
    tools/execute (waterfall)
    tools/post-execute (waterfall)
    tool/result (logged)
  step/end
  ↓
agent/turn-stopping (serial) → turn/end → agent/status: idle
```

### Two-Tier Event System

| Tier | Events | Persisted? | Use |
|------|--------|-----------|-----|
| **Session Events** | `session/event` | YES — JSONL log | Ground truth, replay, SDK projection |
| **Agent Events** | `agent/*` | NO — in-memory | Live UI, real-time coordination |

> **Key Rule**: Anything reaching a model request MUST be reconstructable from the session log.

---

## 5. Session System

- **Format**: JSONL append-only log — `session.v4.jsonl` (current generation)
- **Migration chain**: v0 → v1 → v2 → v3 → v4 (on full open)
- **SQLite**: `session-query` backend — monotonic `SCHEMA_VERSION`

| Package | Role |
|---|---|
| `@deepseek-ai/dsh-session-persistence` | Persistence seam |
| `@deepseek-ai/dsh-session-persistence-jsonl` | JSONL backend (default) |
| `@deepseek-ai/dsh-session-query` | Logical corpus, bounded reads, lineage, semantic/FTS |
| `@deepseek-ai/dsh-session-query-sqlite` | SQLite FTS backend |
| `@deepseek-ai/dsh-session-projection` | Derive/project session views |

- `SESSION_FORMAT_VERSION` bumps only for structural format changes
- `ignorable: true` envelope flag allows unknown event types to pass through

---

## 6. LLM Layer

```
ctx.llm  (Service Definition — @deepseek-ai/dsh-llm)
  ↑ providers:
  ├─ @deepseek-ai/dsh-llm-deepseek    (DeepSeek API)
  ├─ @deepseek-ai/dsh-llm-pi-ai       (Pi AI)
  └─ @deepseek-ai/dsh-llm-replay      (keyless replay for testing)
```

- `llm/stream` is a **waterfall** — Kaioken can wrap or intercept model calls
- `agent/assistant-stream` delivers streaming chunks to UI consumers
- Token metering via `@deepseek-ai/dsh-token-meter`

**To add a new LLM provider for Kaioken:**
1. Create package implementing the `LLMService` seam
2. `ctx.plugin(KaiokenLLMProvider)`
3. Mount in bundle's `cordis.yml`

---

## 7. Tool System

```typescript
ctx.tools.register({
  name: 'my_tool',
  description: '...',
  inputSchema: { type: 'object', properties: { ... } },
  async execute(input, context) { ... }
})
```

**Pipeline:**
```
tool/call → tools/pre-execute (waterfall) → tools/execute (waterfall)
          → tools/post-execute (waterfall) → tool/result
```

**Kaioken hooks:**
- `tools/pre-execute`: permission checks, logging
- `tools/post-execute`: transform/wrap results
- `system-prompt/assemble`: inject tool instructions

---

## 8. Subagent / Workflow System

```
ctx.subagents  (Service Definition — @deepseek-ai/dsh-subagent)
  ↑ providers:
  ├─ subagent-spawn-in-process   (same process, new context scope)
  ├─ subagent-fork-in-process    (new process, IPC)
  ├─ subagent-dsh-sdk            (out-of-process via JSON-RPC)
  ├─ subagent-acp                (automation-only ACP protocol)
  ├─ subagent-codex              (Codex bridge)
  └─ subagent-claude-code        (Claude Code bridge)
```

**Workflow Engine:**
- **PTC (Plugin-Triggered Code)**: model writes JS orchestration scripts
- Runs in sandboxed Node via `@deepseek-ai/dsh-ptc-runtime`
- Tools: `tool-workflow`, `tool-ralph` (loop-based orchestration)

---

## 9. Skills System

```
ctx.skills  (Service Definition — @deepseek-ai/dsh-skill)
  ↑ providers:
  ├─ @deepseek-ai/dsh-skill-filesystem   (SKILL.md files on disk)
  ├─ @deepseek-ai/dsh-skill-office       (Office document skills)
  └─ @deepseek-ai/dsh-skill-badge        (badge-based catalog)
```

Skills are directories with `SKILL.md` (YAML frontmatter + markdown instructions) — same format as Kaioken's own skills.

---

## 10. Capability Seam Pattern

Every extensible capability in DSH follows the 3-role split:

```
Service Definition  →  interface (TypeScript abstract class / interface)
Service Provider    →  concrete implementation plugin
Consumer            →  uses ctx.capability without knowing the provider
```

| Seam | Interface Package | Default Provider |
|---|---|---|
| LLM | `dsh-llm` | `dsh-llm-deepseek` |
| Session Persistence | `dsh-session-persistence` | `dsh-session-persistence-jsonl` |
| Filesystem | `dsh-fs` | `dsh-fs-local` |
| Subprocess | `dsh-subprocess` | `dsh-subprocess-local` |
| Sandbox | `dsh-sandbox` | platform-specific |
| Shell | `dsh-shell` | `dsh-bash-local` |
| Subagents | `dsh-subagent` | `dsh-subagent-spawn-in-process` |
| Skills | `dsh-skill` | `dsh-skill-filesystem` |
| Web | `dsh-web` | `dsh-web-fetch-http` |
| Credentials | `dsh-credentials` | `dsh-credentials-local` |
| Session Query | `dsh-session-query` | `dsh-session-query-sqlite` |

---

## 11. App Layer

| App | Path | Entry |
|---|---|---|
| CLI | `apps/cli` | `pnpm dsh <profile> <task>` |
| Web server + GUI | `apps/web` | `pnpm dsh web` → `http://127.0.0.1:3080` |
| Desktop (Electron) | `apps/desktop` | `pnpm dsh desktop` |
| Desktop host | `apps/desktop-host` | Electron host side |

- Server-side UI: `packages/host/` — services, directory picking, launch, plugin inventory
- Browser-side UI: `packages/client/` — GUI shell, wire protocol, `ui-*` plugins

---

## 12. Key Technical Conventions

| Convention | Detail |
|---|---|
| Module system | ESM everywhere (`"type": "module"`) |
| TypeScript | `strict: true`, `noImplicitAny`, all exports JSDoc'd |
| Branding | Cross-boundary IDs use `Branded<B>` from `dsh-brand` |
| Imports | Package names across packages, `.ts` for local relative |
| Effects | All registrations via `ctx.effect()` / `ctx.on()` — reversible |
| Config | `cordis.yml` — `!!js` allowed only in `config` and `disabled` |
| No hardcoded tunables | Deployment-varying config in validated `Config` fields |
| Closed unions | End in `assertNever` |
| Waterfall listeners | MUST call `next()` or they short-circuit the chain |
| Testing | vitest; coverage gate per-file 100%; snapshot tests for model output |
| Package naming | `@deepseek-ai/dsh-<name>`, ESM, `workspace:*` deps |

---

## 13. Vendored Cordis (`vendor/`)

DSH vendors Cordis under `@deepseek-ai` scope with custom patches:
- Lazy Loader config interpolation after injection resolution
- Reentrant fiber disposal / error containment
- Volatile config diffing (live updates without remount)
- Windows path normalization + ESM/CJS compat fixes

Key vendored packages:
- `@deepseek-ai/cordis` — core plugin framework
- `@deepseek-ai/cosmokit` — utility library
- `@deepseek-ai/schemastery` — JSON Schema validation
- `@deepseek-ai/cordis-plugin-loader` — dynamic plugin loading

---

## 14. Kaioken Integration Points

### Strategy: Kaioken as a DSH Bundle/Profile

Build Kaioken as a **DSH bundle** layered on top of `dsh-base`:

```yaml
# kaioken.bundle.yml
plugins:
  - name: kaioken-identity        # custom identity/persona
  - name: kaioken-system-prompt   # Kaioken-specific prompt sections
  - name: kaioken-tools           # Kaioken-specific tools
  - name: kaioken-ui              # custom UI slots (via packages/client)
  - name: kaioken-skill-provider  # Kaioken skill catalog
  - name: kaioken-llm-adapter     # if using non-DeepSeek LLM
```

### Primary Hook-In Points for Kaioken

1. **System Prompt** — `system-prompt/assemble` waterfall → `ctx.systemPrompt.contribute()`
2. **Tool Registration** — `ctx.tools.register()` for all Kaioken tools
3. **Agent Request** — `agent/request` waterfall → intercept/modify model requests
4. **Agent Loop Replacement** — swap `dsh-agent-loop` for Kaioken-specific loop
5. **LLM Provider** — `ctx.llm` seam → plug in different model or wrap DeepSeek
6. **Session Events** — `session/event` → subscribe for Kaioken logging/telemetry
7. **Skill Loading** — `ctx.skills` seam → `dsh-skill-filesystem` pointed at Kaioken skills dir
8. **UI Slots** — `packages/client` `ui-*` plugins → register custom React components
9. **Subagents** — `ctx.subagents` seam → register Kaioken-specific delegation runners
10. **Preset Composition** — `packages/preset/` → define per-session agent compositions

### Package Naming for Kaioken

- Use `@kaioken/dsh-*` or `kaioken-*` for Kaioken-specific plugins
- Depend on `@deepseek-ai/cordis` in `peerDependencies`
- Use `workspace:*` for internal Kaioken workspace deps

---

## 15. Quick Start

```bash
# From npm
npx @deepseek-ai/dsh web

# From source
git clone https://github.com/deepseek-ai/deepseek-harness.git
cd deepseek-harness
pnpm install && pnpm run build
export DEEPSEEK_API_KEY=sk-...
pnpm dsh web

# With Kaioken bundle
pnpm dsh --profile kaioken "task"
```

### Dev Commands

```bash
pnpm run dev:web              # live-reload web + client bundles
pnpm run test                 # unit tests
pnpm run typecheck            # TypeScript check
pnpm run build                # full build
pnpm run lint                 # oxlint
pnpm dsh --profile headless "task"  # headless run
```

---

## References

| Doc | Path |
|---|---|
| Root AGENTS.md | `d:\project\ai_now_know\deepseek-harness\AGENTS.md` |
| Architecture | `d:\project\ai_now_know\deepseek-harness\docs\architecture.md` |
| Capability Seams | `d:\project\ai_now_know\deepseek-harness\docs\capability-seams.md` |
| Agent Lifecycle | `d:\project\ai_now_know\deepseek-harness\docs\agent-lifecycle.md` |
| Cordis Primer | `d:\project\ai_now_know\deepseek-harness\docs\cordis-primer.md` |
| Package Map | `d:\project\ai_now_know\deepseek-harness\packages\README.md` |
| Tool Catalog | `d:\project\ai_now_know\deepseek-harness\docs\tool-catalog.md` |
| Config Catalog | `d:\project\ai_now_know\deepseek-harness\docs\config-catalog.md` |
| Persistence Catalog | `d:\project\ai_now_know\deepseek-harness\docs\persistence-catalog.md` |

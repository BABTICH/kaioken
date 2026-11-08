# Kaioken Desktop Studio — Architecture Reference

## Process Model

Kaioken runs three isolated processes with strict boundaries.

```
┌──────────────────────────────────────────────────────────────┐
│ Renderer Process                                             │
│   React 19 · Nanostores · CodeMirror · Xterm · Mermaid      │
│   NO Node.js · NO fs · NO child_process                     │
└──────────────────────┬───────────────────────────────────────┘
                       │ window.api (contextBridge)
                       │ sandboxed ipcRenderer
┌──────────────────────▼───────────────────────────────────────┐
│ Preload Process                                              │
│   Typed API surface · ipcRenderer.invoke wrappers            │
└──────────────────────┬───────────────────────────────────────┘
                       │ ipcMain.handle / ipcMain.on
┌──────────────────────▼───────────────────────────────────────┐
│ Electron Main Process                                        │
│   Security Gate · IPC Controllers · PTY Manager              │
│   Approval Gateway · Workspace Guard · Provider Vault        │
│   Diagnostics · Updater · Crash Reporter                     │
└──────────────────────┬───────────────────────────────────────┘
                       │ utilityProcess.fork + MessagePort
┌──────────────────────▼───────────────────────────────────────┐
│ Kaiopi Utility Process                                       │
│   Session Orchestrator · Tool Runtime · Kaioken Workers      │
│   AST Indexing · BM25 Search · Verification · Git Ops       │
└──────────────────────────────────────────────────────────────┘
```

### Why Three Processes?

1. **Crash isolation** — A Kaiopi engine crash never takes down the UI.
2. **CPU isolation** — Heavy indexing never blocks window responsiveness.
3. **Security** — The renderer never touches Node.js, filesystem, or secrets.
4. **Restart semantics** — The engine can be killed and respawned cleanly.

---

## IPC Contract Surface

All IPC is strictly typed and validated with Zod.

### Control Plane (invoke/response)

| Channel | Direction | Purpose |
| --- | --- | --- |
| `window:minimize` | renderer → main | Window control |
| `agent:createSession` | renderer → main | Create agent session |
| `agent:sendMessage` | renderer → main | Send user message |
| `pty:create` | renderer → main | Spawn terminal |
| `workspace:openDialog` | renderer → main | Open folder picker |
| `tools:approval:resolve` | renderer → main | Resolve approval |
| `updater:check` | renderer → main | Check for updates |
| `diagnostics:getState` | renderer → main | Get diagnostics |

### Data Plane (streaming)

| Channel | Direction | Purpose |
| --- | --- | --- |
| `agent:event` | main → renderer | Batched agent events |
| `pty:output` | main → renderer | Terminal output |
| `updater:state` | main → renderer | Updater state changes |
| `diagnostics:state` | main → renderer | Diagnostics updates |

### Batching Strategy

Agent events are coalesced in the utility process:

- Token deltas merged per message ID
- Batch window: 16ms
- Max batch size: 256 events / 256KB
- Flow control via renderer ACK with `busyLevel` backpressure

---

## Security Model

### Renderer Isolation

```ts
contextIsolation: true
nodeIntegration: false
sandbox: true
webSecurity: true
```

### Content Security Policy

Injected via `session.defaultSession.webRequest.onHeadersReceived`:

```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
connect-src 'self' ws: wss: https://api.openai.com https://api.anthropic.com;
object-src 'none';
base-uri 'self';
form-action 'none'
```

### Workspace Sandbox

All filesystem access resolves against the trusted root:

```ts
assertInsideWorkspace(targetPath)
// → realpath containment check
// → rejects ../ traversal
// → rejects symlink escapes
```

### Destructive Approval Gateway

```
Agent proposes destructive tool
  → Engine pauses
  → Main stores pending request + starts timeout
  → Renderer shows ApprovalModal (focus: Deny)
  → User resolves OR timeout auto-denies
  → Main forwards decision to engine
  → Audit log entry written
```

---

## State Management

Renderer state is partitioned by update frequency:

| Store | Update Frequency | Purpose |
| --- | --- | --- |
| `uiStore` | Low | Active surface, panels, modals |
| `sessionStore` | Low | Session lifecycle |
| `agentStreamStore` | High | Streaming text (external cache) |
| `messageStore` | Medium | Message metadata |
| `toolStore` | Medium | Tool call state machine |
| `approvalStore` | Low | Pending approval |
| `terminalStore` | High | Terminal output (xterm direct) |
| `explorerStore` | Low | File tree |
| `diffStore` | Low | Diff review |
| `graphStore` | Low | Codemap |
| `wikiStore` | Low | Wiki pages |
| `cardsStore` | Low | Knowledge cards |
| `costStore` | Low | Cost ledger |
| `updaterStore` | Low | Update state |
| `diagnosticsStore` | Low | Certification |

High-frequency streaming text uses an external mutable cache with `useSyncExternalStore` to avoid React re-render storms.

---

## Key Data Types

All shared types live in `src/shared/types/`:

- `agent.ts` — Session, message, tool, approval events
- `pty.ts` — Terminal session and output
- `workspace.ts` — File tree, search, watch
- `tools.ts` — Approval requests, audit entries
- `diff.ts` — Hunks, decisions, patches
- `graph.ts` — Nodes, edges, layout
- `wiki.ts` — Pages, outlines
- `cards.ts` — Knowledge cards
- `cost.ts` — Ledger entries, budgets
- `updater.ts` — Update state, channels
- `diagnostics.ts` — Metrics, certification

---

## Build Pipeline

```
src/main/       → electron-vite → out/main/
src/preload/    → electron-vite → out/preload/
src/renderer/   → electron-vite → out/renderer/
src/engine/     → electron-vite → out/engine-entry.js
```

Packaging via `electron-builder`:

```
out/ + node_modules/ → asar → dist/
  ├── Kaioken-Setup-x.y.z.exe      (Windows NSIS)
  ├── Kaioken-x.y.z.dmg            (macOS DMG)
  ├── Kaioken-x.y.z.AppImage       (Linux AppImage)
  └── latest.yml / latest-mac.yml  (auto-update feeds)
```

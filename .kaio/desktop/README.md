<div align="center">

# KAIOKEN — Desktop Studio

**Grounded agent runtime with deterministic truth, hard verification, and desktop-grade safety.**

`your product label` · `v0.8.1` · `electron 44 · react 19 · typescript 5.8`

</div>

---

## What is Kaioken Desktop Studio?

Kaioken Desktop Studio is a secure, high-performance desktop application that hosts the **Kaiopi** autonomous agent runtime. It combines a grounded knowledge engine, a deterministic truth layer, and a hardened Electron shell into a single developer-facing workspace.

Every agent action is grounded in repository facts. Every destructive operation requires explicit human approval. Every session is persisted, replayable, and auditable.

---

## Core Capabilities

| Capability | Description |
| --- | --- |
| **Agent Stream** | Real-time streaming conversation with tool cards and citations |
| **Destructive Approval** | Human-in-the-loop gateway with timeout auto-deny |
| **Diff Inspector** | Per-hunk accept/reject with CodeMirror side-by-side view |
| **Codemap Graph** | Force-directed symbol and dependency visualization |
| **Wiki & Cards** | Generated repository documentation and atomic knowledge cards |
| **Cost Ledger** | Token usage, spend charts, budget gauges, multiplier history |
| **Research Surface** | Corpus search with source citation dossier |
| **Terminal** | Integrated PTY with batching and flow control |

---

## Quick Start

### Prerequisites

- Node.js 20+
- npm 10+
- Git

### Install

```bash
git clone https://github.com/babtix/kaioken-desktop.git
cd kaioken-desktop
npm install
```

### Develop

```bash
npm run dev
```

### Typecheck

```bash
npm run typecheck
```

### Test

```bash
npm run test:e2e
```

### Build & Package

```bash
npm run build
npx electron-builder --publish never
```

### Release

```bash
npm run release
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│ Renderer Process (React 19)                         │
│   Surfaces · Stores · Components · Markdown         │
└────────────────────┬────────────────────────────────┘
                     │ contextBridge + sandboxed IPC
┌────────────────────▼────────────────────────────────┐
│ Electron Main Process                               │
│   Security Gate · PTY Manager · Approval Gateway    │
│   Diagnostics · Updater · Workspace Guard           │
└────────────────────┬────────────────────────────────┘
                     │ utilityProcess + MessagePort
┌────────────────────▼────────────────────────────────┐
│ Kaiopi Utility Process                              │
│   Session Orchestrator · Tool Runtime · Kaioken     │
│   Index Workers · Verification · Git Ops            │
└─────────────────────────────────────────────────────┘
```

Three isolated processes. Crash containment. Strict IPC contracts. Zero Node.js exposure in the renderer.

→ Full architecture reference: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)

---

## Design System

Kaioken uses a strict technical-precision aesthetic:

- **Dark-first** ink backgrounds with hairline grid separation
- **Monochrome** with a single red accent (`#ff3b1f`) and ember secondary (`#ff9500`)
- **Terminal aesthetic** — Geist Mono for all UI chrome
- **Zero border-radius** on controls
- **No glows** in production

→ Design reference: [`design/design.md`](design/design.md)
→ Standalone CSS: [`design/kaioken-design-system.css`](design/kaioken-design-system.css)

---

## Project Structure

```
kaioken-desktop/
├── src/
│   ├── main/              # Electron main process
│   │   ├── app/           # Window, security, updater, diagnostics
│   │   ├── ipc/           # IPC handler registration
│   │   ├── engine/        # Kaiopi supervisor
│   │   ├── pty/           # Terminal manager
│   │   ├── workspace/     # Filesystem guard
│   │   ├── tools/         # Approval gateway
│   │   └── settings/      # Provider vault
│   ├── preload/           # Sandboxed contextBridge API
│   ├── renderer/          # React 19 UI
│   │   ├── components/    # All surfaces and panels
│   │   ├── stores/        # Nanostores state
│   │   ├── lib/           # Markdown, graph, virtualization
│   │   └── hooks/         # Session, terminal, hotkeys
│   ├── shared/            # Cross-process types and schemas
│   └── engine/            # Kaiopi utility process entry
├── design/                # Design system reference
├── docs/                  # Documentation
├── tests/                 # E2E tests
├── scripts/               # Build and release scripts
└── .github/workflows/     # CI/CD pipelines
```

---

## Documentation

| Document | Description |
| --- | --- |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Process model, IPC contracts, data flow |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | Local setup, scripts, debugging |
| [`docs/release-runbook.md`](docs/release-runbook.md) | Release procedure and signing |
| [`docs/release-secrets.md`](docs/release-secrets.md) | CI/CD secret configuration |
| [`docs/regression-matrix.md`](docs/regression-matrix.md) | Test coverage matrix |

---

## CLI Companion

Kaioken ships with a lightweight CLI for session inspection and app control.

```bash
npx kaioken --help
npx kaioken info
npx kaioken open
npx kaioken sessions
npx kaioken clear-cache
```

→ CLI reference: [`docs/cli.md`](docs/cli.md)

---

## License

`L0-NC 2.0.1` — See [`LICENSE`](LICENSE) for details.

---

<div align="center">

**Built with technical precision.**

`all systems nominal`

</div>

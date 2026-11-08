# Kaioken Desktop Studio — Developer Guide

## Local Setup

### 1. Clone and Install

```bash
git clone https://github.com/babtix/kaioken-desktop.git
cd kaioken-desktop
npm install
```

The `postinstall` hook runs `electron-builder install-app-deps` to rebuild native modules (node-pty) against the Electron version.

### 2. Verify Installation

```bash
npm run typecheck
```

Expected: No TypeScript errors.

### 3. Start Development

```bash
npm run dev
```

This launches Electron with the Vite dev server. Hot reload is enabled for the renderer. Main process changes require restart.

---

## Available Scripts

| Script | Description |
| --- | --- |
| `npm run dev` | Start dev server + Electron |
| `npm run build` | Build all processes |
| `npm run typecheck` | Run TypeScript checks |
| `npm run test:e2e` | Run Playwright E2E tests |
| `npm run test:e2e:ci` | Build + run E2E |
| `npm run dist:win` | Build Windows installer |
| `npm run dist:mac` | Build macOS DMG |
| `npm run dist:linux` | Build Linux AppImage |
| `npm run release` | Interactive release pipeline |

---

## Project Layout

```
src/
├── main/               # Electron main process
│   ├── index.ts        # Entry point
│   ├── app/            # Window, security, updater, diagnostics
│   ├── ipc/            # IPC handler registration
│   ├── engine/         # Kaiopi utility process supervisor
│   ├── pty/            # Terminal PTY manager
│   ├── workspace/      # Filesystem guard
│   ├── tools/          # Approval gateway
│   └── settings/       # Provider vault
│
├── preload/            # Sandboxed preload scripts
│   ├── index.ts        # contextBridge API surface
│   └── [domain].ts     # Per-domain API wrappers
│
├── renderer/           # React 19 UI
│   ├── main.tsx        # React entry
│   ├── app/            # App shell, router, hotkeys
│   ├── components/     # All UI components
│   │   ├── shell/      # AppShell, TitleBar, NavRail
│   │   ├── chat/       # Agent conversation
│   │   ├── approval/   # Destructive approval modal
│   │   ├── diff/       # Diff inspector
│   │   ├── graph/      # Codemap
│   │   ├── wiki/       # Wiki surface
│   │   ├── cards/      # Knowledge cards
│   │   ├── cost/       # Cost ledger
│   │   ├── research/   # Research surface
│   │   ├── settings/   # Settings panels
│   │   ├── terminal/   # Terminal drawer
│   │   ├── explorer/   # File tree
│   │   └── system/     # Error boundary, recovery
│   ├── stores/         # Nanostores state
│   ├── lib/            # Markdown, graph, virtualization
│   ├── hooks/          # Custom React hooks
│   └── styles/         # Tailwind CSS
│
├── shared/             # Cross-process code
│   ├── types/          # TypeScript interfaces
│   ├── schema/         # Zod validation schemas
│   └── config/         # Shared configuration
│
└── engine/             # Kaiopi utility process
    ├── entry.ts        # Utility process entry
    ├── session/        # Session service
    └── streaming/      # Debug stream generator
```

---

## Debugging

### Renderer DevTools

Open with `Ctrl+Shift+I` in the app window.

### Main Process Logs

Main process logs appear in the terminal where `npm run dev` is running.

### Utility Process Logs

Kaiopi engine logs are prefixed with `[KaiopiEngine]` and piped through the secret redactor.

### IPC Debugging

Add temporary logging in `src/main/ipc/[domain].ts`:

```ts
handleIpc("agent:createSession", Schema, async (input) => {
  console.log("[IPC] createSession", input);
  // ...
});
```

---

## Common Tasks

### Add a New Surface

1. Create component in `src/renderer/components/[name]/`
2. Add store in `src/renderer/stores/[name].ts`
3. Register in `src/renderer/app/router.tsx`
4. Add nav item in `src/renderer/components/shell/NavRail.tsx`
5. Add hotkey in `src/renderer/hooks/useGlobalHotkeys.ts`

### Add a New IPC Channel

1. Define types in `src/shared/types/[domain].ts`
2. Define Zod schema in `src/shared/schema/[domain].zod.ts`
3. Create handler in `src/main/ipc/[domain].ts`
4. Register in `src/main/ipc/index.ts`
5. Add preload wrapper in `src/preload/[domain].ts`
6. Expose in `src/preload/index.ts`

### Add a New Tool

1. Define tool in Kaiopi engine tool runtime
2. Add tool card rendering in `src/renderer/components/chat/ToolCard.tsx`
3. If destructive, wire through approval gateway

---

## Code Style

- **TypeScript strict mode** — no `any` unless unavoidable
- **Zod validation** — all IPC payloads validated
- **Nanostores** — no Redux, no Context for global state
- **Functional components** — no class components
- **Explicit returns** — no implicit undefined

---

## Troubleshooting

### node-pty build fails

```bash
npm rebuild node-pty
# or
npx electron-builder install-app-deps
```

### Port already in use

```bash
# Kill existing Electron processes
taskkill /F /IM electron.exe    # Windows
pkill -f electron               # macOS/Linux
```

### Blank window on Linux

Install required system libraries:

```bash
sudo apt-get install -y libgtk-3-0 libnss3 libasound2 libgbm1
```

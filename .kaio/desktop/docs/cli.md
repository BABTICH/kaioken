# Kaioken CLI Companion

A lightweight command-line tool for inspecting and controlling Kaioken Desktop Studio.

## Installation

The CLI is bundled with the project. Run it via `npx`:

```bash
npx kaioken <command>
```

Or install globally:

```bash
npm link
kaioken <command>
```

## Commands

### `info`

Show application and environment information.

```bash
npx kaioken info
```

Output:
```
  Name:        kaioken-desktop
  Version:     0.8.1
  App ID:      com.kaioken.desktop-studio
  Electron:    ^44.0.0
  React:       ^19.2.0
  App Data:    /home/user/.config/kaioken-desktop
```

### `open`

Launch the desktop application.

```bash
npx kaioken open
```

### `sessions`

List persisted Kaiopi session data.

```bash
npx kaioken sessions
```

Output:
```
  Persisted Sessions:
  ─────────────────────────────────────
  sessions.json                    12.4 KB  2026-01-15
```

### `clear-cache`

Clear cached application data (GPU cache, code cache).

```bash
npx kaioken clear-cache
```

### `help`

Show available commands.

```bash
npx kaioken help
```

## Platform Data Locations

| Platform | Path |
| --- | --- |
| Windows | `%APPDATA%\Kaioken Desktop Studio\` |
| macOS | `~/Library/Application Support/Kaioken Desktop Studio/` |
| Linux | `~/.config/kaioken-desktop/` |

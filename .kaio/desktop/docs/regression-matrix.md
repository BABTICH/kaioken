# Kaioken Desktop Studio — Regression Matrix

Purpose:
This matrix defines the required regression coverage for every Kaioken Desktop Studio release.

Legend:

- `AUTO` — covered by Playwright smoke test.
- `MANUAL` — must be manually verified before release.
- `GATE` — release-blocking.

---

## 1. Boot and Shell

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| App boot | `[data-app-shell]` appears | AUTO | 1.4 / 7.1 | GATE |
| Frameless window | Titlebar controls work | MANUAL | 1.3 | GATE |
| Hotkeys | `Ctrl+1` through `Ctrl+7` switch surfaces | AUTO | 1.4 / 7.1 | GATE |
| Command palette | `Ctrl+K` opens palette | MANUAL | 3.3 | YES |
| Inspector toggle | `Ctrl+B` toggles inspector | MANUAL | 3.1 | YES |
| Terminal drawer | ``Ctrl+` `` toggles terminal drawer | MANUAL | 3.2 | YES |

---

## 2. Workspace and Filesystem

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Workspace open | Native folder dialog opens trusted root | MANUAL | 3.1 | GATE |
| Path sandbox | `../../` requests are denied | MANUAL | 3.1 | GATE |
| Symlink escape | Symlink outside workspace is denied | MANUAL | 3.1 | GATE |
| File tree | Tree expands and collapses | MANUAL | 3.1 | YES |
| Git badges | Modified/untracked files show badges | MANUAL | 3.1 | NO |
| Quick open | `Ctrl+P` finds files | MANUAL | 3.3 | YES |

---

## 3. Terminal

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| PTY creation | Terminal starts in workspace root | MANUAL | 3.2 | GATE |
| Workspace guard | Terminal refuses non-workspace cwd | MANUAL | 3.2 | GATE |
| Fast output | Large output does not freeze UI | MANUAL | 3.2 | YES |
| Resize | Terminal resizes with drawer/window | MANUAL | 3.2 | YES |
| Close | Closing terminal kills PTY | MANUAL | 3.2 | YES |

---

## 4. Agent Session and Streaming

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Session create | Mock session can be created | MANUAL | 2.3 | GATE |
| Streaming | Message deltas render smoothly | MANUAL | 2.2 / 4.1 | GATE |
| Tool cards | Tool lifecycle renders | MANUAL | 4.1 | YES |
| Session restore | Last session restores after reload | MANUAL | 2.5 | YES |
| Event replay | Connect replays retained history | MANUAL | 2.5 | YES |

---

## 5. Destructive Approval

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Approval modal | Modal appears for destructive tool | MANUAL | 4.3 | GATE |
| Safe default | Deny is focused by default | MANUAL | 4.3 | GATE |
| Enter denies | Enter/Esc deny approval | MANUAL | 4.3 | GATE |
| Timeout | Approval auto-denies after timeout | MANUAL | 4.3 | GATE |
| Audit log | Decision is recorded | MANUAL | 4.3 | YES |

---

## 6. Diff Inspector

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Mock diff | Mock diff loads | MANUAL | 5.1 | YES |
| Hunk accept | Accepted hunks appear in final patch | MANUAL | 5.1 | YES |
| Hunk reject | Rejected hunks are omitted | MANUAL | 5.1 | YES |
| CodeMirror | Hunk panes render read-only | MANUAL | 5.1 | YES |

---

## 7. Knowledge Surfaces

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Research | Example query returns mock results | AUTO | 5.6 / 7.1 | YES |
| Wiki | Mock wiki pages load | AUTO | 5.3 / 7.1 | YES |
| Codemap | Graph SVG renders | AUTO | 5.2 / 7.1 | YES |
| Cards | Mock cards load | AUTO | 5.4 / 7.1 | YES |
| Cost | Spend chart renders | AUTO | 5.5 / 7.1 | YES |
| Release notes | Release notes modal opens | MANUAL | 6.3 | YES |

---

## 8. Security and Diagnostics

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Context isolation | Renderer cannot access Node primitives | MANUAL | 6.1 | GATE |
| CSP | Strict CSP headers present | MANUAL | 6.1 | GATE |
| Secret redaction | API keys are redacted from logs | MANUAL | 6.1 | GATE |
| Crash reporter | Crash reports persist | MANUAL | 6.2 | YES |
| Session recovery | Recovery surface appears after crash | MANUAL | 6.2 | YES |
| Certification | Certification panel runs checks | MANUAL | 6.4 | GATE |
| Startup budget | Startup time is measured | AUTO/MANUAL | 6.4 | YES |
| Memory guardrails | Warnings appear for high memory | MANUAL | 6.4 | YES |

---

## 9. Packaging and Release

| Area | Check | Type | Phase | Gate |
| --- | --- | --- | --- | --- |
| Build | `npm run build` succeeds | AUTO | 6.1 | GATE |
| Dist | `npm run dist:*` produces artifact | MANUAL | 6.1 | GATE |
| Installed boot | Installed app boots | MANUAL | 7.1 | GATE |
| Updater channel | Channel persists | MANUAL | 6.3 | YES |
| Update mock flow | Check/download/install flow works | MANUAL | 6.3 | YES |
| Release checklist | Runbook completed | MANUAL | 7.1 | GATE |

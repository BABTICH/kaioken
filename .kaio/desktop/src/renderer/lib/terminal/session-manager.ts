// src/renderer/lib/terminal/session-manager.ts
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import "@xterm/xterm/css/xterm.css";

interface ManagedTerminal {
  id: string;
  term: Terminal;
  fit: FitAddon;
  container: HTMLDivElement;
}

const terminals = new Map<string, ManagedTerminal>();

const pendingWrites = new Map<
  string,
  Array<{
    data: string;
    cb?: () => void;
  }>
>();

let host: HTMLElement | null = null;

export function setTerminalHost(el: HTMLElement | null): void {
  host = el;
  if (host) {
    for (const [, entry] of terminals) {
      if (!entry.container.parentElement) {
        host.appendChild(entry.container);
      }
    }
  }
}

function createContainer(): HTMLDivElement {
  const el = document.createElement("div");

  el.style.width = "100%";
  el.style.height = "100%";
  el.style.background = "var(--bg-sunk)";
  el.style.overflow = "hidden";

  return el;
}

function flushPending(sessionId: string): void {
  const pending = pendingWrites.get(sessionId);

  if (!pending) {
    return;
  }

  pendingWrites.delete(sessionId);

  const terminal = terminals.get(sessionId);

  if (!terminal) {
    for (const item of pending) {
      item.cb?.();
    }
    return;
  }

  for (const item of pending) {
    terminal.term.write(item.data, item.cb);
  }
}

export function getTerminalTheme(theme: "dark" | "light") {
  if (theme === "light") {
    return {
      background: "#fafaf9",
      foreground: "#141416",
      cursor: "#b81600",
      cursorAccent: "#fafaf9",
      selectionBackground: "rgba(184, 22, 0, 0.2)"
    };
  }
  return {
    background: "#08080a",
    foreground: "#f5f5f6",
    cursor: "#ff3b1f",
    cursorAccent: "#08080a",
    selectionBackground: "rgba(255, 59, 31, 0.25)"
  };
}

export function updateTerminalsTheme(theme: "dark" | "light"): void {
  const terminalTheme = getTerminalTheme(theme);
  for (const [, entry] of terminals) {
    entry.term.options.theme = terminalTheme;
  }
}

export function createTerminal(sessionId: string): void {
  if (terminals.has(sessionId)) {
    return;
  }

  const container = createContainer();

  const currentTheme =
    typeof document !== "undefined" &&
    document.documentElement.getAttribute("data-theme") === "light"
      ? "light"
      : "dark";

  const term = new Terminal({
    cursorBlink: true,
    fontFamily:
      "'Geist Mono', ui-monospace, 'SF Mono', Menlo, monospace",
    fontSize: 12,
    lineHeight: 1.35,
    scrollback: 10_000,
    allowProposedApi: true,
    theme: getTerminalTheme(currentTheme)
  });

  const fit = new FitAddon();
  term.loadAddon(fit);

  term.open(container);

  try {
    const webgl = new WebglAddon();

    webgl.onContextLoss(() => {
      webgl.dispose();
    });

    term.loadAddon(webgl);
  } catch {
    // Fall back to the default canvas renderer.
  }

  term.onData((data) => {
    window.api.pty.write(sessionId, data);
  });

  term.onResize(({ cols, rows }) => {
    void window.api.pty
      .resize(sessionId, cols, rows)
      .catch(() => {
        // Ignore resize races during close.
      });
  });

  if (host) {
    host.appendChild(container);
  }

  terminals.set(sessionId, {
    id: sessionId,
    term,
    fit,
    container
  });

  flushPending(sessionId);
}

export function attachTerminal(
  sessionId: string,
  target: HTMLElement
): void {
  const terminal = terminals.get(sessionId);

  if (!terminal) {
    return;
  }

  target.innerHTML = "";
  target.appendChild(terminal.container);

  requestAnimationFrame(() => {
    fitTerminal(sessionId);

    try {
      terminal.term.focus();
    } catch {
      // Ignore focus failures.
    }

    void window.api.pty
      .resize(sessionId, terminal.term.cols, terminal.term.rows)
      .catch(() => {
        // Ignore resize races during attach.
      });
  });
}

export function detachTerminal(sessionId: string): void {
  const terminal = terminals.get(sessionId);

  if (!terminal || !host) {
    return;
  }

  host.appendChild(terminal.container);
}

export function fitTerminal(sessionId: string): void {
  const terminal = terminals.get(sessionId);

  if (!terminal) {
    return;
  }

  if (
    terminal.container.clientWidth === 0 ||
    terminal.container.clientHeight === 0
  ) {
    return;
  }

  try {
    terminal.fit.fit();
  } catch {
    // Ignore fit errors during layout transitions.
  }
}

export function writeTerminal(
  sessionId: string,
  data: string,
  cb?: () => void
): void {
  const terminal = terminals.get(sessionId);

  if (!terminal) {
    const pending = pendingWrites.get(sessionId) ?? [];

    pending.push({
      data,
      cb
    });

    if (pending.length > 500) {
      const dropped = pending.shift();
      dropped?.cb?.();
    }

    pendingWrites.set(sessionId, pending);
    return;
  }

  terminal.term.write(data, cb);
}

export function disposeTerminal(sessionId: string): void {
  const terminal = terminals.get(sessionId);

  pendingWrites.delete(sessionId);

  if (!terminal) {
    return;
  }

  try {
    terminal.term.dispose();
  } catch {
    // Ignore dispose errors.
  }

  terminal.container.remove();
  terminals.delete(sessionId);
}

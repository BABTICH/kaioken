// src/main/pty/manager.ts
import * as pty from "node-pty";
import { randomUUID } from "node:crypto";
import path from "node:path";
import fs from "node:fs";
import type { BrowserWindow } from "electron";
import {
  assertInsideWorkspace,
  getTrustedRoot
} from "../workspace/guard";
import type {
  PtyCreateOptions,
  PtyCreateResult,
  PtyExit,
  PtyOutput
} from "@shared/types/pty";

const OUTPUT_FLUSH_MS = 16;
const OUTPUT_FLUSH_BYTES = 128 * 1024;
const FLOW_HIGH_WATER_BYTES = 1024 * 1024;
const FLOW_LOW_WATER_BYTES = 256 * 1024;

interface PtySession {
  id: string;
  proc: pty.IPty;
  shell: string;
  cwd: string;
  seq: number;
  buffer: string[];
  bufferBytes: number;
  outstandingBytes: number;
  paused: boolean;
  flushTimer?: NodeJS.Timeout;
}

const sessions = new Map<string, PtySession>();

let getWindow: () => BrowserWindow | null = () => null;

export function setPtyWindowGetter(
  getter: () => BrowserWindow | null
): void {
  getWindow = getter;
}

function defaultShell(): string {
  if (process.platform === "win32") {
    return "powershell.exe";
  }

  return process.env.SHELL || "/bin/bash";
}

function sanitizeShell(input?: string): string {
  if (!input) {
    return defaultShell();
  }

  const base = path.basename(input);

  return base || defaultShell();
}

function resolveCwd(cwd?: string): string {
  const root = getTrustedRoot();

  if (!root) {
    if (cwd && fs.existsSync(cwd)) {
      try {
        const stat = fs.statSync(cwd);
        if (stat.isDirectory()) {
          return path.resolve(cwd);
        }
      } catch {
        // Fall back below
      }
    }

    const fallback =
      process.env.USERPROFILE ||
      process.env.HOME ||
      process.cwd();

    if (fs.existsSync(fallback)) {
      return path.resolve(fallback);
    }

    return process.cwd();
  }

  const resolved = assertInsideWorkspace(cwd ?? ".");

  if (!fs.existsSync(resolved)) {
    throw new Error("PATH_DENIED");
  }

  const stat = fs.statSync(resolved);

  if (!stat.isDirectory()) {
    throw new Error("PATH_DENIED");
  }

  return resolved;
}

function buildEnv(): Record<string, string> {
  const env: Record<string, string> = {};

  for (const [key, value] of Object.entries(process.env)) {
    if (typeof value === "string") {
      env[key] = value;
    }
  }

  env.TERM = "xterm-256color";
  env.COLORTERM = "truecolor";
  env.KAIOKEN_DESKTOP = "1";

  return env;
}

function scheduleFlush(session: PtySession): void {
  if (session.flushTimer) {
    return;
  }

  session.flushTimer = setTimeout(() => {
    flushOutput(session);
  }, OUTPUT_FLUSH_MS);
}

function flushOutput(session: PtySession): void {
  if (session.flushTimer) {
    clearTimeout(session.flushTimer);
    session.flushTimer = undefined;
  }

  if (session.buffer.length === 0 || session.bufferBytes === 0) {
    session.buffer = [];
    session.bufferBytes = 0;
    return;
  }

  const data = session.buffer.join("");
  const bytes = session.bufferBytes;

  session.buffer = [];
  session.bufferBytes = 0;

  const win = getWindow();

  if (!win || win.isDestroyed()) {
    return;
  }

  session.seq += 1;
  session.outstandingBytes += bytes;

  const payload: PtyOutput = {
    sessionId: session.id,
    seq: session.seq,
    data,
    bytes
  };

  win.webContents.send("pty:output", payload);

  if (
    session.outstandingBytes > FLOW_HIGH_WATER_BYTES &&
    !session.paused
  ) {
    session.paused = true;

    const proc = session.proc as unknown as {
      pause?: () => void;
    };

    proc.pause?.();
  }
}

function appendOutput(session: PtySession, data: string): void {
  session.buffer.push(data);
  session.bufferBytes += Buffer.byteLength(data, "utf8");

  if (session.bufferBytes >= OUTPUT_FLUSH_BYTES) {
    flushOutput(session);
  } else {
    scheduleFlush(session);
  }
}

export function createPtySession(
  opts: PtyCreateOptions
): PtyCreateResult {
  const shell = sanitizeShell(opts.shell);
  const cwd = resolveCwd(opts.cwd);

  let proc: pty.IPty;
  let activeShell = shell;

  try {
    proc = pty.spawn(shell, [], {
      name: "xterm-256color",
      cols: opts.cols,
      rows: opts.rows,
      cwd,
      env: buildEnv(),
      useConpty: process.platform === "win32"
    });
  } catch (spawnError) {
    if (process.platform === "win32" && shell !== "cmd.exe") {
      activeShell = process.env.COMSPEC || "cmd.exe";
      proc = pty.spawn(activeShell, [], {
        name: "xterm-256color",
        cols: opts.cols,
        rows: opts.rows,
        cwd,
        env: buildEnv(),
        useConpty: false
      });
    } else {
      throw spawnError;
    }
  }

  const id = randomUUID();

  const session: PtySession = {
    id,
    proc,
    shell: activeShell,
    cwd,
    seq: 0,
    buffer: [],
    bufferBytes: 0,
    outstandingBytes: 0,
    paused: false
  };

  sessions.set(id, session);

  proc.onData((data) => {
    appendOutput(session, data);
  });

  proc.onExit(({ exitCode, signal }) => {
    if (session.flushTimer) {
      clearTimeout(session.flushTimer);
      session.flushTimer = undefined;
    }

    sessions.delete(id);

    const win = getWindow();

    if (win && !win.isDestroyed()) {
      const payload: PtyExit = {
        sessionId: id,
        exitCode,
        signal: signal != null ? String(signal) : undefined
      };

      win.webContents.send("pty:exit", payload);
    }
  });

  return {
    sessionId: id,
    pid: proc.pid,
    shell,
    cwd
  };
}

export function writePtySession(
  sessionId: string,
  data: string
): void {
  const session = sessions.get(sessionId);

  if (!session) {
    return;
  }

  session.proc.write(data);
}

export function resizePtySession(
  sessionId: string,
  cols: number,
  rows: number
): void {
  const session = sessions.get(sessionId);

  if (!session) {
    throw new Error("PTY_SESSION_NOT_FOUND");
  }

  session.proc.resize(cols, rows);
}

export function closePtySession(sessionId: string): void {
  const session = sessions.get(sessionId);

  if (!session) {
    return;
  }

  if (session.flushTimer) {
    clearTimeout(session.flushTimer);
    session.flushTimer = undefined;
  }

  try {
    session.proc.kill();
  } catch (error) {
    console.error("[PTY] Failed to kill session", error);
  }

  sessions.delete(sessionId);
}

export function ackPtySession(
  sessionId: string,
  bytes: number
): void {
  const session = sessions.get(sessionId);

  if (!session) {
    return;
  }

  session.outstandingBytes = Math.max(
    0,
    session.outstandingBytes - bytes
  );

  if (
    session.paused &&
    session.outstandingBytes < FLOW_LOW_WATER_BYTES
  ) {
    session.paused = false;

    const proc = session.proc as unknown as {
      resume?: () => void;
    };

    proc.resume?.();
  }
}

export function disposeAllPtySessions(): void {
  for (const sessionId of [...sessions.keys()]) {
    closePtySession(sessionId);
  }
}

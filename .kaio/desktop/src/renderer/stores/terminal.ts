// src/renderer/stores/terminal.ts
import { atom } from "nanostores";
import type { PtyExit } from "@shared/types/pty";
import {
  createTerminal as createManagedTerminal,
  disposeTerminal
} from "@renderer/lib/terminal/session-manager";

export interface TerminalSessionMeta {
  id: string;
  pid: number;
  shell: string;
  cwd: string;
  running: boolean;
  createdAt: number;
}

export interface TerminalState {
  sessions: TerminalSessionMeta[];
  activeId: string | null;
  creating: boolean;
  error: string | null;
}

const initialState: TerminalState = {
  sessions: [],
  activeId: null,
  creating: false,
  error: null
};

export const terminalStore = atom<TerminalState>(initialState);

function set(patch: Partial<TerminalState>): void {
  terminalStore.set({
    ...terminalStore.get(),
    ...patch
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export async function createTerminal(): Promise<void> {
  const state = terminalStore.get();

  if (state.creating) {
    return;
  }

  set({
    creating: true,
    error: null
  });

  try {
    const result = await window.api.pty.create({
      cols: 80,
      rows: 24
    });

    const meta: TerminalSessionMeta = {
      id: result.sessionId,
      pid: result.pid,
      shell: result.shell,
      cwd: result.cwd,
      running: true,
      createdAt: Date.now()
    };

    createManagedTerminal(result.sessionId);

    set({
      sessions: [...terminalStore.get().sessions, meta],
      activeId: result.sessionId,
      creating: false
    });
  } catch (error) {
    set({
      creating: false,
      error: errorMessage(error)
    });
  }
}

export function setActiveTerminal(sessionId: string): void {
  set({
    activeId: sessionId
  });
}

export async function closeActiveTerminal(): Promise<void> {
  const state = terminalStore.get();

  if (!state.activeId) {
    return;
  }

  const sessionId = state.activeId;

  try {
    await window.api.pty.close(sessionId);
  } catch {
    // Ignore close races with process exit.
  }

  removeTerminal(sessionId);
}

export function removeTerminal(sessionId: string): void {
  disposeTerminal(sessionId);

  const state = terminalStore.get();

  const sessions = state.sessions.filter(
    (session) => session.id !== sessionId
  );

  const activeId =
    state.activeId === sessionId
      ? sessions[0]?.id ?? null
      : state.activeId;

  set({
    sessions,
    activeId
  });
}

export function markTerminalExited(info: PtyExit): void {
  const state = terminalStore.get();

  set({
    sessions: state.sessions.map((session) => {
      if (session.id !== info.sessionId) {
        return session;
      }

      return {
        ...session,
        running: false
      };
    })
  });
}

// src/renderer/stores/session.ts
import { atom } from "nanostores";
import type {
  AgentSessionInfo,
  AgentSessionSummary,
  CreateAgentSessionInput
} from "@shared/types/agent";
import {
  agentStreamDebug,
  resetAgentStreamDebug
} from "./agent-events";
import { ingestAgentEnvelope } from "@renderer/lib/event-batcher";
import { resetMessages } from "./messages";
import { resetTools } from "./tools";

export type AgentConnectionState =
  | "idle"
  | "connecting"
  | "connected"
  | "error";

export interface SessionStoreState {
  sessions: AgentSessionSummary[];
  activeSession: AgentSessionInfo | null;
  connectionState: AgentConnectionState;
  error: string | null;
}

const initialState: SessionStoreState = {
  sessions: [],
  activeSession: null,
  connectionState: "idle",
  error: null
};

const ACTIVE_SESSION_KEY = "kaioken.desktop.activeSessionId";

export const sessionStore = atom<SessionStoreState>(initialState);

function set(patch: Partial<SessionStoreState>): void {
  sessionStore.set({
    ...sessionStore.get(),
    ...patch
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function rememberActiveSession(sessionId: string): void {
  try {
    localStorage.setItem(ACTIVE_SESSION_KEY, sessionId);
  } catch {
    // Ignore storage failures.
  }
}

function readRememberedActiveSession(): string | null {
  try {
    return localStorage.getItem(ACTIVE_SESSION_KEY);
  } catch {
    return null;
  }
}

export async function refreshSessions(): Promise<void> {
  try {
    const sessions = await window.api.agent.listSessions();

    set({ sessions });
  } catch (error) {
    console.error("[sessionStore] Failed to refresh sessions", error);
  }
}

export async function connectActiveSession(): Promise<void> {
  const { activeSession } = sessionStore.get();

  if (!activeSession) {
    return;
  }

  set({
    connectionState: "connecting",
    error: null
  });

  try {
    const afterCursor =
      agentStreamDebug.get().lastCursor ?? undefined;

    const connection = await window.api.agent.connect(
      activeSession.sessionId,
      afterCursor
    );

    for (const envelope of connection.replay) {
      ingestAgentEnvelope(envelope);
    }

    set({
      connectionState: "connected"
    });
  } catch (error) {
    set({
      connectionState: "error",
      error: errorMessage(error)
    });
  }
}

export async function createSession(
  input?: Partial<CreateAgentSessionInput>
): Promise<void> {
  set({
    connectionState: "connecting",
    error: null
  });

  try {
    resetAgentStreamDebug();
    resetMessages();
    resetTools();

    const sessionInfo = await window.api.agent.createSession({
      workspaceRoot: input?.workspaceRoot ?? "mock://workspace",
      providerId: input?.providerId ?? "mock",
      modelId: input?.modelId ?? "mock-stream",
      multiplier: input?.multiplier ?? 1
    });

    set({
      activeSession: sessionInfo
    });

    rememberActiveSession(sessionInfo.sessionId);

    await refreshSessions();
    await connectActiveSession();
  } catch (error) {
    set({
      connectionState: "error",
      error: errorMessage(error)
    });
  }
}

export async function restoreSession(
  sessionId: string
): Promise<void> {
  set({
    connectionState: "connecting",
    error: null
  });

  try {
    resetAgentStreamDebug();
    resetMessages();
    resetTools();

    const sessionInfo =
      await window.api.agent.restoreSession(sessionId);

    set({
      activeSession: sessionInfo
    });

    rememberActiveSession(sessionInfo.sessionId);

    await connectActiveSession();
  } catch (error) {
    set({
      connectionState: "error",
      error: errorMessage(error)
    });
  }
}

export async function restoreLastActiveSession(): Promise<void> {
  const { activeSession } = sessionStore.get();

  if (activeSession) {
    return;
  }

  const remembered = readRememberedActiveSession();

  if (!remembered) {
    return;
  }

  let { sessions } = sessionStore.get();
  if (sessions.length === 0) {
    await refreshSessions();
    sessions = sessionStore.get().sessions;
  }

  const exists = sessions.some(
    (session) => session.sessionId === remembered
  );

  if (!exists) {
    return;
  }

  await restoreSession(remembered);
}

export async function sendTestMessage(
  text = "Build Kaioken Desktop Studio with strict process isolation."
): Promise<void> {
  const { activeSession } = sessionStore.get();

  if (!activeSession) {
    return;
  }

  try {
    await window.api.agent.sendMessage({
      sessionId: activeSession.sessionId,
      text
    });
  } catch (error) {
    set({
      error: errorMessage(error)
    });
  }
}

export async function stopActiveSession(): Promise<void> {
  const { activeSession } = sessionStore.get();

  if (!activeSession) {
    return;
  }

  try {
    await window.api.agent.stopSession(
      activeSession.sessionId,
      "user_stop"
    );

    await refreshSessions();
  } catch (error) {
    set({
      error: errorMessage(error)
    });
  }
}

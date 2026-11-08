// src/renderer/stores/agent-events.ts
import { atom } from "nanostores";
import type {
  AgentEvent,
  AgentEventEnvelope
} from "@shared/types/agent";
import { applyMessageEvent } from "./messages";
import { applyToolEvent } from "./tools";

export interface AgentStreamDebugState {
  activeSessionId: string | null;
  status: "idle" | "ready" | "running" | "stopped" | "error";
  lastSeq: number;
  lastCursor: string | null;
  envelopes: number;
  events: number;
  tokens: number;
  bytes: number;
  lastEventAt: number | null;
  busyLevel: 0 | 1 | 2;
}

const initialState: AgentStreamDebugState = {
  activeSessionId: null,
  status: "idle",
  lastSeq: 0,
  lastCursor: null,
  envelopes: 0,
  events: 0,
  tokens: 0,
  bytes: 0,
  lastEventAt: null,
  busyLevel: 0
};

export const agentStreamDebug =
  atom<AgentStreamDebugState>(initialState);

let ackTimer: number | undefined;
let unackedBytes = 0;
let lastSessionId: string | null = null;
let lastSeq = 0;

export function resetAgentStreamDebug(): void {
  if (ackTimer) {
    window.clearTimeout(ackTimer);
    ackTimer = undefined;
  }

  unackedBytes = 0;
  lastSessionId = null;
  lastSeq = 0;

  agentStreamDebug.set({ ...initialState });
}

export function trackEnvelope(
  envelope: AgentEventEnvelope
): void {
  const current = agentStreamDebug.get();

  lastSessionId = envelope.sessionId;
  lastSeq = envelope.seq;

  const bytes = JSON.stringify(envelope).length;
  unackedBytes += bytes;

  agentStreamDebug.set({
    ...current,
    activeSessionId: envelope.sessionId,
    lastSeq: envelope.seq,
    lastCursor: envelope.cursor,
    envelopes: current.envelopes + 1,
    events: current.events + envelope.events.length,
    bytes: current.bytes + bytes,
    lastEventAt: envelope.emittedAt
  });

  scheduleAck();
}

export function applyAgentEvents(events: AgentEvent[]): void {
  const current = agentStreamDebug.get();

  let tokens = current.tokens;
  let status = current.status;

  for (const event of events) {
    applyMessageEvent(event);
    applyToolEvent(event);

    if (event.type === "message.delta") {
      tokens += event.deltaTokens ?? 0;
    }

    if (event.type === "session.status") {
      if (event.status === "ready") {
        status = "ready";
      } else if (event.status === "running") {
        status = "running";
      } else if (event.status === "stopped") {
        status = "stopped";
      } else if (event.status === "error") {
        status = "error";
      }
    }
  }

  agentStreamDebug.set({
    ...current,
    tokens,
    status
  });
}

function scheduleAck(): void {
  if (ackTimer) {
    return;
  }

  ackTimer = window.setTimeout(() => {
    ackTimer = undefined;

    if (lastSessionId && lastSeq > 0 && unackedBytes > 0) {
      window.api.agent.ackEvents({
        sessionId: lastSessionId,
        lastSeq,
        receivedBytes: unackedBytes,
        busyLevel: agentStreamDebug.get().busyLevel
      });

      unackedBytes = 0;
    }
  }, 250);
}

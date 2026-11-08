// src/preload/agent.ts
import { ipcRenderer } from "electron";
import { invokeIpc } from "./invoke";
import type {
  AgentApi,
  AgentConnectionInfo,
  AgentEventEnvelope,
  AgentMessageInput,
  AgentSessionInfo,
  AgentSessionSummary,
  CreateAgentSessionInput,
  DebugStreamOptions,
  EventAck,
  SendMessageResult,
  StartDebugStreamResult,
  Unsubscribe
} from "@shared/types/agent";

const eventListeners = new Set<(envelope: AgentEventEnvelope) => void>();

ipcRenderer.on("agent:event", (_event, envelope: AgentEventEnvelope) => {
  for (const listener of eventListeners) {
    listener(envelope);
  }
});

export const agentApi: AgentApi = {
  createSession(
    input: CreateAgentSessionInput
  ): Promise<AgentSessionInfo> {
    return invokeIpc<AgentSessionInfo>("agent:createSession", input);
  },

  listSessions(): Promise<AgentSessionSummary[]> {
    return invokeIpc<AgentSessionSummary[]>("agent:listSessions");
  },

  restoreSession(sessionId: string): Promise<AgentSessionInfo> {
    return invokeIpc<AgentSessionInfo>("agent:restoreSession", sessionId);
  },

  sendMessage(input: AgentMessageInput): Promise<SendMessageResult> {
    return invokeIpc<SendMessageResult>("agent:sendMessage", input);
  },

  stopSession(sessionId: string, reason?: string): Promise<void> {
    return invokeIpc<void>("agent:stopSession", {
      sessionId,
      reason
    });
  },

  connect(
    sessionId: string,
    afterCursor?: string
  ): Promise<AgentConnectionInfo> {
    return invokeIpc<AgentConnectionInfo>("agent:connect", {
      sessionId,
      afterCursor
    });
  },

  startDebugStream(
    options: DebugStreamOptions
  ): Promise<StartDebugStreamResult> {
    return invokeIpc<StartDebugStreamResult>(
      "agent:startDebugStream",
      options
    );
  },

  stopDebugStream(sessionId: string): Promise<void> {
    return invokeIpc<void>("agent:stopDebugStream", sessionId);
  },

  ackEvents(ack: EventAck): void {
    ipcRenderer.send("agent:ackEvents", ack);
  },

  onEvent(cb: (envelope: AgentEventEnvelope) => void): Unsubscribe {
    eventListeners.add(cb);

    return () => {
      eventListeners.delete(cb);
    };
  }
};

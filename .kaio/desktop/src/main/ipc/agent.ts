// src/main/ipc/agent.ts
import { ipcMain } from "electron";
import { handleIpc } from "./safe-handler";
import { VoidInputSchema } from "@shared/schema/ipc.zod";
import {
  ConnectSessionSchema,
  CreateAgentSessionInputSchema,
  DebugStreamOptionsSchema,
  EventAckSchema,
  SendMessageSchema,
  SessionIdSchema,
  StopSessionSchema
} from "@shared/schema/agent.zod";
import { engineSupervisor } from "../engine/supervisor";
import { assertTrustedSender } from "../app/security";
import { getEngineProviderConfig } from "../settings/provider-vault";
import type {
  AgentConnectionInfo,
  AgentSessionInfo,
  AgentSessionSummary,
  SendMessageResult,
  StartDebugStreamResult
} from "@shared/types/agent";

export function registerAgentIpc(): void {
  handleIpc(
    "agent:createSession",
    CreateAgentSessionInputSchema,
    async (input) => {
      const provider = await getEngineProviderConfig(
        input.providerId
      ).catch((error) => {
        console.warn("[IPC] Provider vault lookup failed", error);
        return undefined;
      });

      return engineSupervisor.request<AgentSessionInfo>(
        "agent.createSession",
        {
          ...input,
          provider
        },
        5000
      );
    }
  );

  handleIpc(
    "agent:listSessions",
    VoidInputSchema,
    async () => {
      return engineSupervisor.request<AgentSessionSummary[]>(
        "agent.listSessions",
        {},
        5000
      );
    }
  );

  handleIpc(
    "agent:restoreSession",
    SessionIdSchema,
    async (sessionId) => {
      return engineSupervisor.request<AgentSessionInfo>(
        "agent.restoreSession",
        { sessionId },
        5000
      );
    }
  );

  handleIpc(
    "agent:sendMessage",
    SendMessageSchema,
    async (input) => {
      return engineSupervisor.request<SendMessageResult>(
        "agent.sendMessage",
        input,
        5000
      );
    }
  );

  handleIpc(
    "agent:stopSession",
    StopSessionSchema,
    async (input) => {
      await engineSupervisor.request(
        "agent.stopSession",
        input,
        5000
      );
    }
  );

  handleIpc(
    "agent:connect",
    ConnectSessionSchema,
    async (input) => {
      return engineSupervisor.request<AgentConnectionInfo>(
        "agent.connect",
        input,
        5000
      );
    }
  );

  handleIpc(
    "agent:startDebugStream",
    DebugStreamOptionsSchema,
    async (input) => {
      return engineSupervisor.request<StartDebugStreamResult>(
        "agent.debug.start",
        input,
        5000
      );
    }
  );

  handleIpc(
    "agent:stopDebugStream",
    SessionIdSchema,
    async (sessionId) => {
      await engineSupervisor.request(
        "agent.debug.stop",
        { sessionId },
        5000
      );
    }
  );

  ipcMain.on("agent:ackEvents", (event, payload) => {
    try {
      assertTrustedSender(event);

      const ack = EventAckSchema.parse(payload);

      engineSupervisor.notify("agent.ackEvents", ack);
    } catch (error) {
      console.error("[IPC] agent:ackEvents rejected", error);
    }
  });
}

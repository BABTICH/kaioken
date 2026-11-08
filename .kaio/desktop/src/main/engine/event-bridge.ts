// src/main/engine/event-bridge.ts
import type { BrowserWindow } from "electron";
import { engineSupervisor } from "./supervisor";
import type { AgentEventEnvelope } from "@shared/types/agent";

export function attachEngineEventBridge(window: BrowserWindow): () => void {
  return engineSupervisor.onNotification((notification) => {
    if (notification.method !== "agent.events") {
      return;
    }

    if (window.isDestroyed()) {
      return;
    }

    const envelope = notification.params as AgentEventEnvelope;

    window.webContents.send("agent:event", envelope);
  });
}

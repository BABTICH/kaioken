// src/preload/tools.ts
import { ipcRenderer } from "electron";
import { invokeIpc } from "./invoke";
import type {
  ApprovalRequest,
  ApprovalResolvedEvent,
  AuditEntry,
  RendererApprovalResolution,
  ToolsApi,
  Unsubscribe
} from "@shared/types/tools";

const approvalRequestListeners = new Set<
  (request: ApprovalRequest) => void
>();

const approvalResolvedListeners = new Set<
  (event: ApprovalResolvedEvent) => void
>();

ipcRenderer.on(
  "tools:approval:request",
  (_event, payload: ApprovalRequest) => {
    for (const listener of approvalRequestListeners) {
      listener(payload);
    }
  }
);

ipcRenderer.on(
  "tools:approval:resolved",
  (_event, payload: ApprovalResolvedEvent) => {
    for (const listener of approvalResolvedListeners) {
      listener(payload);
    }
  }
);

export const toolsApi: ToolsApi = {
  onApprovalRequest(
    cb: (request: ApprovalRequest) => void
  ): Unsubscribe {
    approvalRequestListeners.add(cb);

    return () => {
      approvalRequestListeners.delete(cb);
    };
  },

  onApprovalResolved(
    cb: (event: ApprovalResolvedEvent) => void
  ): Unsubscribe {
    approvalResolvedListeners.add(cb);

    return () => {
      approvalResolvedListeners.delete(cb);
    };
  },

  resolveApproval(
    resolution: RendererApprovalResolution
  ): Promise<void> {
    return invokeIpc<void>("tools:approval:resolve", resolution);
  },

  getAuditLog(limit?: number): Promise<AuditEntry[]> {
    return invokeIpc<AuditEntry[]>("tools:audit:get", {
      limit
    });
  }
};

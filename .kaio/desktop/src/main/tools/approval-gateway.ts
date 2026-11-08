// src/main/tools/approval-gateway.ts
import { randomUUID } from "node:crypto";
import { EngineApprovalRequestSchema } from "@shared/schema/tools.zod";
import type {
  ApprovalDecision,
  ApprovalRequest,
  ApprovalResolution,
  AuditEntry,
  RendererApprovalResolution
} from "@shared/types/tools";
import { engineSupervisor } from "../engine/supervisor";
import { getMainWindow } from "../app/window";
import {
  getAuditEntries,
  initAuditLog,
  recordAudit
} from "./audit-log";

const DEFAULT_TIMEOUT_MS = 300_000;

interface PendingApproval {
  request: ApprovalRequest;
  timer: NodeJS.Timeout;
}

const pending = new Map<string, PendingApproval>();

function sendToRenderer(channel: string, payload: unknown): void {
  const win = getMainWindow();

  if (win && !win.isDestroyed()) {
    win.webContents.send(channel, payload);
  }
}

export async function initApprovalGateway(): Promise<void> {
  await initAuditLog();
}

export function handleEngineApprovalRequest(params: unknown): void {
  const parsed = EngineApprovalRequestSchema.safeParse(params);

  if (!parsed.success) {
    console.error(
      "[ApprovalGateway] Invalid engine approval request",
      parsed.error
    );
    return;
  }

  const data = parsed.data;

  const request: ApprovalRequest = {
    requestId: data.requestId,
    sessionId: data.sessionId,
    toolCallId: data.toolCallId,
    messageId: data.messageId,
    kind: data.kind,
    risk: data.risk ?? "destructive",
    title: data.title,
    description: data.description,
    command: data.command,
    filePath: data.filePath,
    diffRef: data.diffRef,
    timeoutMs: data.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    createdAt: data.createdAt ?? Date.now()
  };

  if (pending.has(request.requestId)) {
    return;
  }

  const timer = setTimeout(() => {
    void resolveApprovalInternal({
      requestId: request.requestId,
      decision: "timeout",
      reason: "approval timeout"
    });
  }, request.timeoutMs);

  pending.set(request.requestId, {
    request,
    timer
  });

  sendToRenderer("tools:approval:request", request);
}

export async function resolveRendererApproval(
  resolution: RendererApprovalResolution
): Promise<void> {
  await resolveApprovalInternal(resolution);
}

async function resolveApprovalInternal(
  resolution: ApprovalResolution
): Promise<void> {
  const entry = pending.get(resolution.requestId);

  if (!entry) {
    return;
  }

  clearTimeout(entry.timer);
  pending.delete(resolution.requestId);

  const auditEntry: AuditEntry = {
    id: randomUUID(),
    at: Date.now(),
    sessionId: entry.request.sessionId,
    toolCallId: entry.request.toolCallId,
    kind: entry.request.kind,
    decision: resolution.decision,
    summary: entry.request.title
  };

  await recordAudit(auditEntry);

  sendToRenderer("tools:approval:resolved", {
    requestId: resolution.requestId,
    decision: resolution.decision
  });

  const notified = engineSupervisor.notify("tools.approvalResolved", {
    requestId: resolution.requestId,
    sessionId: entry.request.sessionId,
    toolCallId: entry.request.toolCallId,
    decision: resolution.decision,
    reason: resolution.reason
  });

  if (!notified) {
    console.warn(
      "[ApprovalGateway] Engine unavailable for approval resolution",
      resolution.requestId
    );
  }
}

export function getAuditLog(limit?: number): AuditEntry[] {
  return getAuditEntries(limit ?? 100);
}

export function disposeApprovals(): void {
  for (const [, entry] of pending) {
    clearTimeout(entry.timer);
  }

  pending.clear();
}

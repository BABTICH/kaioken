// src/shared/schema/tools.zod.ts
import { z } from "zod";

export const DestructiveToolKindSchema = z.enum([
  "write_to_file",
  "apply_patch",
  "run_command",
  "git_operation"
]);

export const RendererApprovalDecisionSchema = z.enum([
  "allow_once",
  "deny"
]);

export const ResolveApprovalSchema = z.object({
  requestId: z.string().min(1),
  decision: RendererApprovalDecisionSchema,
  reason: z.string().max(512).optional()
});

export const GetAuditLogSchema = z.object({
  limit: z.number().int().min(1).max(500).optional()
});

export const EngineApprovalRequestSchema = z.object({
  requestId: z.string().min(1),
  sessionId: z.string().min(1),
  toolCallId: z.string().min(1),
  messageId: z.string().optional(),
  kind: DestructiveToolKindSchema,
  risk: z.enum(["mutate", "destructive"]).optional(),
  title: z.string().min(1),
  description: z.string().min(1),
  command: z.string().max(2048).optional(),
  filePath: z.string().max(2048).optional(),
  diffRef: z.string().max(512).optional(),
  timeoutMs: z.number().int().min(1000).max(600_000).optional(),
  createdAt: z.number().int().nonnegative().optional()
});

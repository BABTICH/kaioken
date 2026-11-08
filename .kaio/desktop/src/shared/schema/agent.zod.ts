// src/shared/schema/agent.zod.ts
import { z } from "zod";

const MultiplierSchema = z.number().int().min(1).max(10);

export const CreateAgentSessionInputSchema = z.object({
  workspaceRoot: z.string().min(1),
  providerId: z.string().min(1),
  modelId: z.string().min(1),
  multiplier: MultiplierSchema.optional()
});

export const SessionIdSchema = z.string().min(1);

export const SendMessageSchema = z.object({
  sessionId: z.string().min(1),
  text: z.string().min(1)
});

export const StopSessionSchema = z.object({
  sessionId: z.string().min(1),
  reason: z.string().min(1).optional()
});

export const ConnectSessionSchema = z.object({
  sessionId: z.string().min(1),
  afterCursor: z.string().min(1).optional()
});

export const DebugStreamOptionsSchema = z.object({
  sessionId: z.string().min(1),
  eventsPerSecond: z.number().int().min(1).max(1000).optional(),
  tokensPerEvent: z.number().int().min(1).max(256).optional(),
  durationMs: z.number().int().min(250).max(600_000).optional()
});

export const EventAckSchema = z.object({
  sessionId: z.string().min(1),
  lastSeq: z.number().int().nonnegative(),
  receivedBytes: z.number().int().nonnegative(),
  busyLevel: z.union([z.literal(0), z.literal(1), z.literal(2)])
});

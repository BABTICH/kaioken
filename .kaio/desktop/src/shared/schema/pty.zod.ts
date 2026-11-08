// src/shared/schema/pty.zod.ts
import { z } from "zod";

export const PtyCreateSchema = z.object({
  cols: z.number().int().min(2).max(500),
  rows: z.number().int().min(2).max(200),
  cwd: z.string().min(1).max(1024).optional(),
  shell: z
    .string()
    .min(1)
    .max(128)
    .regex(/^[^\\/]+$/, "Shell must not contain path separators")
    .optional(),
  profile: z
    .enum(["default", "build", "test", "git"])
    .optional()
});

export const PtySessionIdSchema = z.string().min(1);

export const PtyWriteSchema = z.object({
  sessionId: z.string().min(1),
  data: z.string()
});

export const PtyResizeSchema = z.object({
  sessionId: z.string().min(1),
  cols: z.number().int().min(2).max(500),
  rows: z.number().int().min(2).max(200)
});

export const PtyAckSchema = z.object({
  sessionId: z.string().min(1),
  seq: z.number().int().nonnegative(),
  bytes: z.number().int().nonnegative()
});

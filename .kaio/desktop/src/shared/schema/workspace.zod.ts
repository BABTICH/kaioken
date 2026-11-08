// src/shared/schema/workspace.zod.ts
import { z } from "zod";

export const WorkspacePathSchema = z.string().min(1);

export const WorkspaceTreeSchema = z.object({
  path: z.string().min(1),
  depth: z.number().int().min(1).max(5).optional()
});

export const WorkspaceReadFileSchema = z.object({
  path: z.string().min(1)
});

export const WorkspaceStatSchema = z.object({
  path: z.string().min(1)
});

export const WorkspaceSearchSchema = z.object({
  query: z.string().min(1).max(256),
  limit: z.number().int().min(1).max(100).optional(),
  includeIgnored: z.boolean().optional()
});

export const WorkspaceWatchSchema = z.object({
  path: z.string().min(1)
});

export const WorkspaceOpenPathSchema = z.object({
  path: z.string().min(1)
});

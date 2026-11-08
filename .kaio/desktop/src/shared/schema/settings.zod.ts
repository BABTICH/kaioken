// src/shared/schema/settings.zod.ts
import { z } from "zod";

export const ProviderIdSchema = z
  .string()
  .min(1)
  .max(64);

export const SaveProviderInputSchema = z.object({
  id: z
    .string()
    .min(1)
    .max(64)
    .regex(/^[a-zA-Z0-9_-]+$/, "Provider ID must be alphanumeric, dash, or underscore"),

  name: z
    .string()
    .min(1)
    .max(128)
    .optional(),

  baseUrl: z
    .union([z.literal(""), z.string().url().max(2048)])
    .optional(),

  defaultModelId: z
    .string()
    .min(1)
    .max(256)
    .optional(),

  apiKey: z
    .union([z.literal(""), z.string().min(1).max(4096), z.null()])
    .optional()
});

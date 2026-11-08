// src/shared/schema/updater.zod.ts
import { z } from "zod";

export const UpdateChannelSchema = z.enum([
  "stable",
  "beta",
  "dev"
]);

export const SetUpdateChannelSchema = z.object({
  channel: UpdateChannelSchema
});

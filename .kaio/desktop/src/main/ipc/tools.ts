// src/main/ipc/tools.ts
import { handleIpc } from "./safe-handler";
import {
  GetAuditLogSchema,
  ResolveApprovalSchema
} from "@shared/schema/tools.zod";
import {
  getAuditLog,
  resolveRendererApproval
} from "../tools/approval-gateway";
import type { AuditEntry } from "@shared/types/tools";

export function registerToolsIpc(): void {
  handleIpc(
    "tools:approval:resolve",
    ResolveApprovalSchema,
    async (input): Promise<void> => {
      await resolveRendererApproval(input);
    }
  );

  handleIpc(
    "tools:audit:get",
    GetAuditLogSchema,
    async (input): Promise<AuditEntry[]> => {
      return getAuditLog(input.limit);
    }
  );
}

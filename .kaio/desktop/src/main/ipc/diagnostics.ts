// src/main/ipc/diagnostics.ts
import { handleIpc } from "./safe-handler";
import { VoidInputSchema } from "@shared/schema/ipc.zod";
import { SaveCertificationReportSchema } from "@shared/schema/diagnostics.zod";
import * as diagnostics from "../app/diagnostics";
import type {
  DiagnosticsState,
  MainMemoryMetrics,
  StartupMetrics
} from "@shared/types/diagnostics";

export function registerDiagnosticsIpc(): void {
  handleIpc(
    "diagnostics:getState",
    VoidInputSchema,
    (): DiagnosticsState => {
      return diagnostics.getDiagnosticsState();
    }
  );

  handleIpc(
    "diagnostics:getMemory",
    VoidInputSchema,
    (): MainMemoryMetrics => {
      return diagnostics.getMemoryMetrics();
    }
  );

  handleIpc(
    "diagnostics:getStartup",
    VoidInputSchema,
    (): StartupMetrics => {
      return diagnostics.getStartupMetrics();
    }
  );

  handleIpc(
    "diagnostics:markRendererReady",
    VoidInputSchema,
    async (): Promise<void> => {
      diagnostics.markRendererReady();
    }
  );

  handleIpc(
    "diagnostics:saveReport",
    SaveCertificationReportSchema,
    async (input): Promise<void> => {
      await diagnostics.saveCertificationReport(input);
    }
  );
}

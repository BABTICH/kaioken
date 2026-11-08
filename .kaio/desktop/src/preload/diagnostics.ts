// src/preload/diagnostics.ts
import { ipcRenderer } from "electron";
import { invokeIpc } from "./invoke";
import type {
  CertificationReport,
  DiagnosticsApi,
  DiagnosticsState,
  MainMemoryMetrics,
  StartupMetrics
} from "@shared/types/diagnostics";

const stateListeners = new Set<(state: DiagnosticsState) => void>();

ipcRenderer.on(
  "diagnostics:state",
  (_event, payload: DiagnosticsState) => {
    for (const listener of stateListeners) {
      listener(payload);
    }
  }
);

export const diagnosticsApi: DiagnosticsApi = {
  getState(): Promise<DiagnosticsState> {
    return invokeIpc<DiagnosticsState>("diagnostics:getState");
  },

  getMemory(): Promise<MainMemoryMetrics> {
    return invokeIpc<MainMemoryMetrics>("diagnostics:getMemory");
  },

  getStartup(): Promise<StartupMetrics> {
    return invokeIpc<StartupMetrics>("diagnostics:getStartup");
  },

  markRendererReady(): Promise<void> {
    return invokeIpc<void>("diagnostics:markRendererReady");
  },

  saveReport(report: CertificationReport): Promise<void> {
    return invokeIpc<void>("diagnostics:saveReport", report);
  },

  onState(cb: (state: DiagnosticsState) => void) {
    stateListeners.add(cb);

    return () => {
      stateListeners.delete(cb);
    };
  }
};

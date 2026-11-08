// src/preload/crash.ts
import { ipcRenderer } from "electron";
import type { CrashApi } from "@shared/types/ipc";

export const crashApi: CrashApi = {
  record: (report) =>
    ipcRenderer.invoke("crash:record", {
      id: `crash-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      timestamp: Date.now(),
      userAgent: typeof navigator !== "undefined" ? navigator.userAgent : undefined,
      ...report
    }),
  getReports: () => ipcRenderer.invoke("crash:getReports")
};

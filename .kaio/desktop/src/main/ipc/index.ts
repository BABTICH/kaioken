// src/main/ipc/index.ts
import { ipcMain } from "electron";
import { registerWindowIpc } from "./window";
import { registerEngineIpc } from "./engine";
import { registerAgentIpc } from "./agent";
import { registerSettingsIpc } from "./settings";
import { registerWorkspaceIpc } from "./workspace";
import { registerPtyIpc } from "./pty";
import { registerToolsIpc } from "./tools";
import { registerUpdaterIpc } from "./updater";
import { registerDiagnosticsIpc } from "./diagnostics";
import {
  getCrashReports,
  recordCrash,
  type CrashReport
} from "../utils/crash-reporter";

export function registerIpcHandlers(): void {
  console.log("[IPC] Registering handlers...");

  registerWindowIpc();
  registerEngineIpc();
  registerAgentIpc();
  registerSettingsIpc();
  registerWorkspaceIpc();
  registerPtyIpc();
  registerToolsIpc();
  registerUpdaterIpc();
  registerDiagnosticsIpc();

  ipcMain.handle("crash:record", async (_event, report: CrashReport) => {
    await recordCrash(report);
  });

  ipcMain.handle("crash:getReports", async () => {
    return getCrashReports();
  });
}

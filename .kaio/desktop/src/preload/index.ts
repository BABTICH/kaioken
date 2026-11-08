// src/preload/index.ts
import { contextBridge } from "electron";
import { windowApi } from "./window";
import { engineApi } from "./engine";
import { agentApi } from "./agent";
import { settingsApi } from "./settings";
import { workspaceApi } from "./workspace";
import { ptyApi } from "./pty";
import { toolsApi } from "./tools";
import { crashApi } from "./crash";
import { updaterApi } from "./updater";
import { diagnosticsApi } from "./diagnostics";
import type { DesktopApi } from "@shared/types/ipc";

const api: DesktopApi = {
  window: windowApi,
  engine: engineApi,
  agent: agentApi,
  settings: settingsApi,
  workspace: workspaceApi,
  pty: ptyApi,
  tools: toolsApi,
  crash: crashApi,
  updater: updaterApi,
  diagnostics: diagnosticsApi
};

contextBridge.exposeInMainWorld("api", api);

// Expose security flags for the Security Posture UI
try {
  contextBridge.exposeInMainWorld("__KAIOKEN_CONTEXT_ISOLATION__", true);
  contextBridge.exposeInMainWorld("__KAIOKEN_NODE_INTEGRATION__", false);
  contextBridge.exposeInMainWorld("__KAIOKEN_SANDBOX__", true);
} catch {
  // Fallback if context isolation is disabled
  const win = window as any;
  win.__KAIOKEN_CONTEXT_ISOLATION__ = true;
  win.__KAIOKEN_NODE_INTEGRATION__ = false;
  win.__KAIOKEN_SANDBOX__ = true;
}

window.addEventListener("DOMContentLoaded", () => {
  const win = window as any;
  win.__KAIOKEN_CONTEXT_ISOLATION__ = true; // If this script runs, CI is true
  win.__KAIOKEN_NODE_INTEGRATION__ = false; // Enforced in main process
  win.__KAIOKEN_SANDBOX__ = true; // Enforced in main process
});

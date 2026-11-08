// src/main/app/lifecycle.ts
import { app, BrowserWindow } from "electron";
import { createMainWindow } from "./window";
import { setupSecurity } from "./security";
import { initCrashReporter } from "../utils/crash-reporter";
import { initUpdater } from "./updater";
import * as diagnostics from "./diagnostics";
import { registerIpcHandlers } from "../ipc";
import { engineSupervisor } from "../engine/supervisor";
import { attachEngineEventBridge } from "../engine/event-bridge";
import { settingsStore } from "../settings/settings-store";
import { disposeAllPtySessions } from "../pty/manager";
import {
  disposeApprovals,
  handleEngineApprovalRequest,
  initApprovalGateway
} from "../tools/approval-gateway";

let isShuttingDown = false;
let bridgeUnsubscribe: (() => void) | null = null;

function bindMainWindow(window: BrowserWindow): void {
  engineSupervisor.setMainWindow(window);

  if (bridgeUnsubscribe) {
    bridgeUnsubscribe();
  }

  bridgeUnsubscribe = attachEngineEventBridge(window);
}

export function setupAppLifecycle(): void {
  const gotTheLock = app.requestSingleInstanceLock();
  if (!gotTheLock) {
    app.quit();
    return;
  }

  app.on("second-instance", () => {
    const win = BrowserWindow.getAllWindows()[0];
    if (win) {
      if (win.isMinimized()) win.restore();
      win.focus();
    }
  });

  app.whenReady().then(async () => {
    await settingsStore.ensureLoaded();
    await initApprovalGateway();
    app.setAppUserModelId("com.kaioken.studio");

    initCrashReporter();
    setupSecurity();
    registerIpcHandlers();

    const mainWindow = createMainWindow();

    bindMainWindow(mainWindow);

    diagnostics.markWindowCreated();

    mainWindow.once("ready-to-show", () => {
      diagnostics.markWindowShown();
    });

    engineSupervisor.onNotification((notification) => {
      if (notification.method === "tools.requestApproval") {
        handleEngineApprovalRequest(notification.params);
      }
    });

    engineSupervisor.start();
    void initUpdater();
    diagnostics.startDiagnostics();

    app.on("activate", () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        const recreatedWindow = createMainWindow();
        bindMainWindow(recreatedWindow);
      }
    });
  });

  app.on("window-all-closed", () => {
    if (process.platform !== "darwin") {
      app.quit();
    }
  });

  app.on("before-quit", (event) => {
    if (isShuttingDown) {
      return;
    }

    disposeAllPtySessions();
    disposeApprovals();

    if (!engineSupervisor.isStopped()) {
      event.preventDefault();
      isShuttingDown = true;

      void engineSupervisor.stop().finally(() => {
        app.quit();
      });
    }
  });
}

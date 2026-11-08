import { app, BrowserWindow, utilityProcess } from "electron";
import type { UtilityProcess } from "electron";
import { join } from "node:path";
import { redactSecrets } from "../utils/redactor";
import { EngineRpcClient } from "./rpc";
import type {
  EngineState,
  EngineStateEvent,
  EngineSupervisorSnapshot,
  RpcNotification
} from "@shared/types/engine";
import type { Unsubscribe } from "@shared/types/ipc";

const HEARTBEAT_INTERVAL_MS = 5000;
const HEARTBEAT_TIMEOUT_MS = 3000;
const SHUTDOWN_TIMEOUT_MS = 3000;
const MAX_RESTART_DELAY_MS = 15_000;

class EngineSupervisor {
  private child?: UtilityProcess;

  private rpc?: EngineRpcClient;

  private state: EngineState = "stopped";

  private reason?: string;

  private heartbeatTimer?: NodeJS.Timeout;

  private restartTimer?: NodeJS.Timeout;

  private restartAttempts = 0;

  private intentionallyStopping = false;

  private started = false;

  private mainWindow: BrowserWindow | null = null;

  private readonly externalNotificationListeners = new Set<
    (notification: RpcNotification) => void
  >();

  setMainWindow(window: BrowserWindow | null): void {
    this.mainWindow = window;
  }

  getState(): EngineState {
    return this.state;
  }

  isStopped(): boolean {
    return this.state === "stopped" && !this.child;
  }

  getSnapshot(): EngineSupervisorSnapshot {
    return {
      state: this.state,
      pid: this.child?.pid,
      restartAttempts: this.restartAttempts,
      reason: this.reason
    };
  }

  start(): void {
    if (this.started) {
      return;
    }

    this.started = true;
    this.intentionallyStopping = false;

    this.spawn();
    this.startHeartbeat();
  }

  async stop(): Promise<void> {
    this.intentionallyStopping = true;
    this.clearTimers();

    if (!this.child) {
      this.setState("stopped", "no child process");
      return;
    }

    const child = this.child;

    try {
      await this.rpc?.request("engine.shutdown", {}, SHUTDOWN_TIMEOUT_MS);
    } catch (error) {
      console.warn("[EngineSupervisor] Graceful shutdown failed", error);
    }

    this.killChild(child);
    this.setState("stopped", "shutdown");
  }

  request<T>(
    method: string,
    params?: unknown,
    timeoutMs?: number
  ): Promise<T> {
    if (this.state !== "ready" || !this.rpc) {
      return Promise.reject(new Error("ENGINE_DOWN"));
    }

    return this.rpc.request<T>(method, params, timeoutMs);
  }

  notify(method: string, params?: unknown): boolean {
    if (this.state !== "ready" || !this.rpc) {
      return false;
    }

    this.rpc.notify(method, params);
    return true;
  }

  onNotification(
    listener: (notification: RpcNotification) => void
  ): Unsubscribe {
    this.externalNotificationListeners.add(listener);

    return () => {
      this.externalNotificationListeners.delete(listener);
    };
  }

  private spawn(): void {
    if (this.child) {
      return;
    }

    if (this.restartTimer) {
      clearTimeout(this.restartTimer);
      this.restartTimer = undefined;
    }

    this.setState("starting", "spawn");

    const entryPath = join(__dirname, "engine-entry.js");

    const sessionDir = join(
      app.getPath("userData"),
      "kaiopi-sessions"
    );

    const child = utilityProcess.fork(entryPath, [], {
      serviceName: "kaiopi-engine",
      stdio: "pipe",
      env: {
        ...(process.env as Record<string, string>),
        KAIOKEN_SESSION_DIR: sessionDir
      }
    });

    this.child = child;

    const rpc = new EngineRpcClient((message) => {
      try {
        child.postMessage(message);
      } catch (error) {
        console.error("[EngineSupervisor] Failed to post message", error);
      }
    });

    this.rpc = rpc;

    rpc.onNotification((notification) => {
      this.handleNotification(notification);
    });

    child.on("message", (message) => {
      rpc.handleMessage(message);
    });

    child.on("spawn", () => {
      console.log("[EngineSupervisor] Utility process spawned", {
        pid: child.pid
      });
    });

    child.on("exit", (code) => {
      this.handleExit(code);
    });

    child.on("error", (error) => {
      console.error("[EngineSupervisor] Utility process error", error);
    });

    child.stdout?.on("data", (data) => {
      const raw = String(data).trim();
      if (raw) {
        console.log(`[KaiopiEngine] ${redactSecrets(raw)}`);
      }
    });

    child.stderr?.on("data", (data) => {
      const raw = String(data).trim();
      if (raw) {
        console.error(`[KaiopiEngine:err] ${redactSecrets(raw)}`);
      }
    });
  }

  private handleNotification(notification: RpcNotification): void {
    if (notification.method === "engine.ready") {
      this.restartAttempts = 0;
      this.setState("ready", "engine.ready");
    }

    for (const listener of this.externalNotificationListeners) {
      listener(notification);
    }
  }

  private handleExit(code: number | null): void {
    console.warn("[EngineSupervisor] Engine exited", {
      code
    });

    this.rpc?.dispose();
    this.rpc = undefined;
    this.child = undefined;

    if (this.intentionallyStopping) {
      return;
    }

    this.scheduleRestart(`exit:${code ?? "unknown"}`);
  }

  private scheduleRestart(reason: string): void {
    this.reason = reason;

    const delay = Math.min(
      1000 * 2 ** this.restartAttempts,
      MAX_RESTART_DELAY_MS
    );

    this.restartAttempts += 1;

    this.setState("restarting", reason);

    this.restartTimer = setTimeout(() => {
      this.spawn();
    }, delay);
  }

  private startHeartbeat(): void {
    this.heartbeatTimer = setInterval(() => {
      void this.performHeartbeat();
    }, HEARTBEAT_INTERVAL_MS);
  }

  private async performHeartbeat(): Promise<void> {
    if (this.state !== "ready" || !this.rpc) {
      return;
    }

    try {
      await this.rpc.request("engine.ping", {}, HEARTBEAT_TIMEOUT_MS);
    } catch (error) {
      console.error("[EngineSupervisor] Heartbeat failed", error);
      this.killChild(this.child);
    }
  }

  private killChild(child?: UtilityProcess): void {
    if (!child) {
      return;
    }

    try {
      child.kill();
    } catch (error) {
      console.error("[EngineSupervisor] Failed to kill child", error);
    }
  }

  private clearTimers(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = undefined;
    }

    if (this.restartTimer) {
      clearTimeout(this.restartTimer);
      this.restartTimer = undefined;
    }
  }

  private setState(state: EngineState, reason?: string): void {
    this.state = state;

    if (reason) {
      this.reason = reason;
    }

    const event: EngineStateEvent = {
      state,
      reason,
      pid: this.child?.pid
    };

    if (this.mainWindow && !this.mainWindow.isDestroyed()) {
      this.mainWindow.webContents.send("engine:state", event);
    }

    console.log("[EngineSupervisor] state:", state, reason ?? "");
  }
}

export const engineSupervisor = new EngineSupervisor();

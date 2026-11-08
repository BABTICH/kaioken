// src/shared/types/ipc.ts
import type { EngineSupervisorSnapshot } from "./engine";
import type { AgentApi } from "./agent";
import type { SettingsApi } from "./settings";
import type { WorkspaceApi } from "./workspace";
import type { PtyApi } from "./pty";
import type { ToolsApi } from "./tools";
import type { UpdaterApi } from "./updater";
import type { DiagnosticsApi } from "./diagnostics";

export type Json =
  | string
  | number
  | boolean
  | null
  | Json[]
  | { [key: string]: Json };

export type Unsubscribe = () => void;

export interface IpcError {
  code:
    | "INVALID_ARGUMENT"
    | "UNAUTHORIZED"
    | "NOT_FOUND"
    | "TIMEOUT"
    | "ENGINE_DOWN"
    | "WORKSPACE_NOT_OPEN"
    | "PATH_DENIED"
    | "TOOL_DENIED"
    | "PTY_ERROR"
    | "INTERNAL";
  message: string;
  retryable?: boolean;
  details?: Json;
}

export type IpcResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: IpcError };

export interface WindowApi {
  minimize(): Promise<void>;
  maximize(): Promise<void>;
  close(): Promise<void>;
  isMaximized(): Promise<boolean>;
}

export interface EngineApi {
  getState(): Promise<EngineSupervisorSnapshot>;
}

export interface CrashApi {
  record(report: {
    message: string;
    stack?: string;
    componentStack?: string;
  }): Promise<void>;
  getReports(): Promise<
    Array<{
      id: string;
      timestamp: number;
      message: string;
      stack?: string;
    }>
  >;
}

export interface DesktopApi {
  window: WindowApi;
  engine: EngineApi;
  agent: AgentApi;
  settings: SettingsApi;
  workspace: WorkspaceApi;
  pty: PtyApi;
  tools: ToolsApi;
  crash: CrashApi;
  updater: UpdaterApi;
  diagnostics: DiagnosticsApi;
}

declare global {
  interface Window {
    api: DesktopApi;
  }
}


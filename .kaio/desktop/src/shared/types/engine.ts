export interface RpcError {
  code: number;
  message: string;
  data?: unknown;
}

export interface RpcRequest {
  jsonrpc: "2.0";
  id: string;
  method: string;
  params?: unknown;
  [key: string]: unknown;
}

export interface RpcResponse {
  jsonrpc: "2.0";
  id: string;
  result?: unknown;
  error?: RpcError;
  [key: string]: unknown;
}

export interface RpcNotification {
  jsonrpc: "2.0";
  method: string;
  params?: unknown;
  [key: string]: unknown;
}

export type EngineState =
  | "stopped"
  | "starting"
  | "ready"
  | "error"
  | "restarting";

export interface EngineSupervisorSnapshot {
  state: EngineState;
  pid?: number;
  restartAttempts: number;
  reason?: string;
}

export interface EngineStateEvent {
  state: EngineState;
  reason?: string;
  pid?: number;
}

export interface EngineStatus {
  state: "ready";
  pid: number;
  uptimeMs: number;
  startedAt: number;
  version: string;
}

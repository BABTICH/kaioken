import { ipcRenderer } from "electron";
import type { IpcResult } from "@shared/types/ipc";

export class IpcClientError extends Error {
  readonly code: string;
  readonly retryable?: boolean;
  readonly details?: unknown;

  constructor(error: {
    code: string;
    message: string;
    retryable?: boolean;
    details?: unknown;
  }) {
    super(error.message);
    this.name = "IpcClientError";
    this.code = error.code;
    this.retryable = error.retryable;
    this.details = error.details;
  }
}

export async function invokeIpc<T>(
  channel: string,
  ...args: unknown[]
): Promise<T> {
  const result = (await ipcRenderer.invoke(channel, ...args)) as IpcResult<T>;

  if (!result.ok) {
    throw new IpcClientError(result.error);
  }

  return result.value;
}

// src/preload/pty.ts
import { ipcRenderer } from "electron";
import { invokeIpc } from "./invoke";
import type {
  PtyApi,
  PtyCreateOptions,
  PtyCreateResult,
  PtyExit,
  PtyOutput,
  Unsubscribe
} from "@shared/types/pty";

const outputListeners = new Set<(output: PtyOutput) => void>();
const exitListeners = new Set<(info: PtyExit) => void>();

ipcRenderer.on("pty:output", (_event, payload: PtyOutput) => {
  for (const listener of outputListeners) {
    listener(payload);
  }
});

ipcRenderer.on("pty:exit", (_event, payload: PtyExit) => {
  for (const listener of exitListeners) {
    listener(payload);
  }
});

export const ptyApi: PtyApi = {
  create(opts: PtyCreateOptions): Promise<PtyCreateResult> {
    return invokeIpc<PtyCreateResult>("pty:create", opts);
  },

  write(sessionId: string, data: string): void {
    ipcRenderer.send("pty:write", {
      sessionId,
      data
    });
  },

  resize(
    sessionId: string,
    cols: number,
    rows: number
  ): Promise<void> {
    return invokeIpc<void>("pty:resize", {
      sessionId,
      cols,
      rows
    });
  },

  close(sessionId: string): Promise<void> {
    return invokeIpc<void>("pty:close", sessionId);
  },

  onOutput(cb: (output: PtyOutput) => void): Unsubscribe {
    outputListeners.add(cb);

    return () => {
      outputListeners.delete(cb);
    };
  },

  onExit(cb: (info: PtyExit) => void): Unsubscribe {
    exitListeners.add(cb);

    return () => {
      exitListeners.delete(cb);
    };
  },

  ack(sessionId: string, seq: number, bytes: number): void {
    ipcRenderer.send("pty:ack", {
      sessionId,
      seq,
      bytes
    });
  }
};

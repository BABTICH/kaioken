// src/main/ipc/pty.ts
import { ipcMain } from "electron";
import { handleIpc } from "./safe-handler";
import {
  PtyAckSchema,
  PtyCreateSchema,
  PtyResizeSchema,
  PtySessionIdSchema,
  PtyWriteSchema
} from "@shared/schema/pty.zod";
import {
  ackPtySession,
  closePtySession,
  createPtySession,
  resizePtySession,
  setPtyWindowGetter,
  writePtySession
} from "../pty/manager";
import { getMainWindow } from "../app/window";
import { assertTrustedSender } from "../app/security";
import type { PtyCreateResult } from "@shared/types/pty";

export function registerPtyIpc(): void {
  setPtyWindowGetter(() => getMainWindow());

  handleIpc(
    "pty:create",
    PtyCreateSchema,
    async (input): Promise<PtyCreateResult> => {
      return createPtySession(input);
    }
  );

  handleIpc(
    "pty:resize",
    PtyResizeSchema,
    async (input): Promise<void> => {
      resizePtySession(input.sessionId, input.cols, input.rows);
    }
  );

  handleIpc(
    "pty:close",
    PtySessionIdSchema,
    async (sessionId): Promise<void> => {
      closePtySession(sessionId);
    }
  );

  ipcMain.on("pty:write", (event, payload) => {
    try {
      assertTrustedSender(event);

      const parsed = PtyWriteSchema.parse(payload);

      writePtySession(parsed.sessionId, parsed.data);
    } catch (error) {
      console.error("[IPC] pty:write rejected", error);
    }
  });

  ipcMain.on("pty:ack", (event, payload) => {
    try {
      assertTrustedSender(event);

      const parsed = PtyAckSchema.parse(payload);

      ackPtySession(parsed.sessionId, parsed.bytes);
    } catch (error) {
      console.error("[IPC] pty:ack rejected", error);
    }
  });
}

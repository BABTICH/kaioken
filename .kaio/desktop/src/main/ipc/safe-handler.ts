import { ipcMain } from "electron";
import type { IpcMainInvokeEvent } from "electron";
import { ZodError, type ZodType } from "zod";
import { assertTrustedSender } from "../app/security";
import type { IpcResult } from "@shared/types/ipc";

export function handleIpc<TInput, TOutput>(
  channel: string,
  schema: ZodType<TInput>,
  handler: (input: TInput, event: IpcMainInvokeEvent) => Promise<TOutput> | TOutput
): void {
  ipcMain.handle(
    channel,
    async (event, ...args: unknown[]): Promise<IpcResult<TOutput>> => {
      try {
        assertTrustedSender(event);

        const rawInput =
          args.length === 0 ? undefined : args.length === 1 ? args[0] : args;

        const input = schema.parse(rawInput);
        const value = await handler(input, event);

        return {
          ok: true,
          value
        };
      } catch (error) {
        if (error instanceof ZodError) {
          return {
            ok: false,
            error: {
              code: "INVALID_ARGUMENT",
              message: error.issues
                .map((issue) => issue.message)
                .join(", ")
            }
          };
        }

        const message = error instanceof Error ? error.message : String(error);

        const code = message.includes("Unauthorized")
          ? "UNAUTHORIZED"
          : "INTERNAL";

        return {
          ok: false,
          error: {
            code,
            message
          }
        };
      }
    }
  );
}

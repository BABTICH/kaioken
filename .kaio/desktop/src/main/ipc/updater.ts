// src/main/ipc/updater.ts
import { handleIpc } from "./safe-handler";
import { VoidInputSchema } from "@shared/schema/ipc.zod";
import { SetUpdateChannelSchema } from "@shared/schema/updater.zod";
import * as updater from "../app/updater";
import type { UpdateState } from "@shared/types/updater";

export function registerUpdaterIpc(): void {
  handleIpc(
    "updater:getState",
    VoidInputSchema,
    (): UpdateState => {
      return updater.getUpdaterState();
    }
  );

  handleIpc(
    "updater:check",
    VoidInputSchema,
    async (): Promise<void> => {
      await updater.checkForUpdates();
    }
  );

  handleIpc(
    "updater:download",
    VoidInputSchema,
    async (): Promise<void> => {
      await updater.downloadUpdate();
    }
  );

  handleIpc(
    "updater:install",
    VoidInputSchema,
    async (): Promise<void> => {
      await updater.installUpdate();
    }
  );

  handleIpc(
    "updater:setChannel",
    SetUpdateChannelSchema,
    async (input): Promise<void> => {
      await updater.setUpdateChannel(input.channel);
    }
  );
}

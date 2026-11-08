// src/main/ipc/settings.ts
import { handleIpc } from "./safe-handler";
import { VoidInputSchema } from "@shared/schema/ipc.zod";
import {
  ProviderIdSchema,
  SaveProviderInputSchema
} from "@shared/schema/settings.zod";
import {
  listPublicProviders,
  removeProvider,
  saveProvider
} from "../settings/provider-vault";
import type { ProviderPublicConfig } from "@shared/types/settings";

export function registerSettingsIpc(): void {
  handleIpc(
    "settings:listProviders",
    VoidInputSchema,
    async (): Promise<ProviderPublicConfig[]> => {
      return listPublicProviders();
    }
  );

  handleIpc(
    "settings:saveProvider",
    SaveProviderInputSchema,
    async (input): Promise<ProviderPublicConfig> => {
      return saveProvider(input);
    }
  );

  handleIpc(
    "settings:removeProvider",
    ProviderIdSchema,
    async (id): Promise<void> => {
      await removeProvider(id);
    }
  );
}

// src/preload/settings.ts
import { invokeIpc } from "./invoke";
import type {
  ProviderPublicConfig,
  SaveProviderInput,
  SettingsApi
} from "@shared/types/settings";

export const settingsApi: SettingsApi = {
  listProviders(): Promise<ProviderPublicConfig[]> {
    return invokeIpc<ProviderPublicConfig[]>(
      "settings:listProviders"
    );
  },

  saveProvider(
    input: SaveProviderInput
  ): Promise<ProviderPublicConfig> {
    return invokeIpc<ProviderPublicConfig>(
      "settings:saveProvider",
      input
    );
  },

  removeProvider(id: string): Promise<void> {
    return invokeIpc<void>("settings:removeProvider", id);
  }
};

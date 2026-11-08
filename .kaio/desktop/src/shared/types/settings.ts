// src/shared/types/settings.ts

export interface ProviderPublicConfig {
  id: string;
  name: string;
  baseUrl?: string;
  defaultModelId?: string;
  hasApiKey: boolean;
  builtin: boolean;
  updatedAt: number;
}

export interface SaveProviderInput {
  id: string;
  name?: string;
  baseUrl?: string;
  defaultModelId?: string;

  /**
   * - undefined: do not change existing secret
   * - string: replace secret
   * - null: clear secret
   * - empty string: treated as undefined by main-process normalization
   */
  apiKey?: string | null;
}

export interface SettingsApi {
  listProviders(): Promise<ProviderPublicConfig[]>;
  saveProvider(input: SaveProviderInput): Promise<ProviderPublicConfig>;
  removeProvider(id: string): Promise<void>;
}

// src/main/settings/provider-vault.ts
import { app, safeStorage } from "electron";
import { settingsStore, type PersistedProvider } from "./settings-store";
import type {
  ProviderPublicConfig,
  SaveProviderInput
} from "@shared/types/settings";
import type { ProviderEngineConfig } from "@shared/types/agent";

const SAFE_PREFIX = "safe:";
const DEV_PREFIX = "dev:";

function encryptSecret(value: string): string {
  if (safeStorage.isEncryptionAvailable()) {
    return (
      SAFE_PREFIX +
      safeStorage.encryptString(value).toString("base64")
    );
  }

  if (!app.isPackaged) {
    console.warn(
      "[ProviderVault] safeStorage unavailable. Using development-only obfuscation."
    );

    return (
      DEV_PREFIX +
      Buffer.from(value, "utf8").toString("base64")
    );
  }

  throw new Error("SECRET_STORAGE_UNAVAILABLE");
}

function decryptSecret(stored?: string): string | undefined {
  if (!stored) {
    return undefined;
  }

  try {
    if (stored.startsWith(SAFE_PREFIX)) {
      return safeStorage.decryptString(
        Buffer.from(stored.slice(SAFE_PREFIX.length), "base64")
      );
    }

    if (stored.startsWith(DEV_PREFIX) && !app.isPackaged) {
      return Buffer.from(
        stored.slice(DEV_PREFIX.length),
        "base64"
      ).toString("utf8");
    }
  } catch (error) {
    console.error("[ProviderVault] Failed to decrypt secret", error);
  }

  return undefined;
}

function toPublic(provider: PersistedProvider): ProviderPublicConfig {
  return {
    id: provider.id,
    name: provider.name,
    baseUrl: provider.baseUrl,
    defaultModelId: provider.defaultModelId,
    hasApiKey: Boolean(provider.apiKeyEnc),
    builtin: Boolean(provider.builtin),
    updatedAt: provider.updatedAt
  };
}

export async function listPublicProviders(): Promise<
  ProviderPublicConfig[]
> {
  const providers = await settingsStore.getProviders();

  return providers.map(toPublic);
}

export async function saveProvider(
  input: SaveProviderInput
): Promise<ProviderPublicConfig> {
  const existing = await settingsStore.getProvider(input.id);

  const now = Date.now();

  const name =
    input.name === "" ? undefined : input.name;

  const baseUrl =
    input.baseUrl === "" ? undefined : input.baseUrl;

  const defaultModelId =
    input.defaultModelId === "" ? undefined : input.defaultModelId;

  const next: PersistedProvider = {
    id: input.id,
    name: name ?? existing?.name ?? input.id,
    baseUrl: baseUrl ?? existing?.baseUrl,
    defaultModelId: defaultModelId ?? existing?.defaultModelId,
    builtin: existing?.builtin ?? false,
    updatedAt: now,
    apiKeyEnc: existing?.apiKeyEnc
  };

  if (input.apiKey === null) {
    next.apiKeyEnc = undefined;
  } else if (
    typeof input.apiKey === "string" &&
    input.apiKey.length > 0
  ) {
    next.apiKeyEnc = encryptSecret(input.apiKey);
  }

  await settingsStore.upsertProvider(next);

  return toPublic(next);
}

export async function removeProvider(id: string): Promise<void> {
  await settingsStore.removeProvider(id);
}

export async function getEngineProviderConfig(
  providerId: string
): Promise<ProviderEngineConfig | undefined> {
  const provider = await settingsStore.getProvider(providerId);

  if (!provider) {
    return undefined;
  }

  const apiKey = decryptSecret(provider.apiKeyEnc);

  return {
    providerId: provider.id,
    baseUrl: provider.baseUrl,
    apiKey
  };
}

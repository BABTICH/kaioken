// src/renderer/stores/settings.ts
import { atom } from "nanostores";
import type {
  ProviderPublicConfig,
  SaveProviderInput
} from "@shared/types/settings";

export interface SettingsState {
  providers: ProviderPublicConfig[];
  loading: boolean;
  saving: boolean;
  error: string | null;
}

const initialState: SettingsState = {
  providers: [],
  loading: false,
  saving: false,
  error: null
};

export const settingsStore = atom<SettingsState>(initialState);

function set(patch: Partial<SettingsState>): void {
  settingsStore.set({
    ...settingsStore.get(),
    ...patch
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export async function loadProviders(): Promise<void> {
  set({
    loading: true,
    error: null
  });

  try {
    const providers = await window.api.settings.listProviders();

    set({
      providers,
      loading: false
    });
  } catch (error) {
    set({
      loading: false,
      error: errorMessage(error)
    });
  }
}

export async function saveProviderInput(
  input: SaveProviderInput
): Promise<void> {
  set({
    saving: true,
    error: null
  });

  try {
    await window.api.settings.saveProvider(input);
    await loadProviders();

    set({
      saving: false
    });
  } catch (error) {
    set({
      saving: false,
      error: errorMessage(error)
    });
  }
}

export async function removeProviderById(
  id: string
): Promise<void> {
  set({
    saving: true,
    error: null
  });

  try {
    await window.api.settings.removeProvider(id);
    await loadProviders();

    set({
      saving: false
    });
  } catch (error) {
    set({
      saving: false,
      error: errorMessage(error)
    });
  }
}

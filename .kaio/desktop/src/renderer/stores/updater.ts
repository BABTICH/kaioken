// src/renderer/stores/updater.ts
import { atom } from "nanostores";
import type {
  UpdateChannel,
  UpdateState
} from "@shared/types/updater";

const initialState: UpdateState = {
  status: "idle",
  channel: "stable",
  currentVersion: "",
  latestVersion: undefined,
  downloadProgress: 0,
  error: null,
  releaseNotes: "",
  updatedAt: 0
};

export const updaterStore = atom<UpdateState>(initialState);

let initialized = false;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function initUpdaterBridge(): void {
  if (initialized) {
    return;
  }

  initialized = true;

  window.api.updater.onState((state) => {
    updaterStore.set(state);
  });

  void window.api.updater
    .getState()
    .then((state) => {
      updaterStore.set(state);
    })
    .catch(() => {
      // Updater bridge unavailable.
    });
}

export async function checkForUpdates(): Promise<void> {
  try {
    await window.api.updater.check();
  } catch (error) {
    updaterStore.set({
      ...updaterStore.get(),
      status: "error",
      error: errorMessage(error)
    });
  }
}

export async function downloadUpdate(): Promise<void> {
  try {
    await window.api.updater.download();
  } catch (error) {
    updaterStore.set({
      ...updaterStore.get(),
      status: "error",
      error: errorMessage(error)
    });
  }
}

export async function installUpdate(): Promise<void> {
  try {
    await window.api.updater.install();
  } catch (error) {
    updaterStore.set({
      ...updaterStore.get(),
      status: "error",
      error: errorMessage(error)
    });
  }
}

export async function setUpdateChannel(
  channel: UpdateChannel
): Promise<void> {
  try {
    await window.api.updater.setChannel(channel);
  } catch (error) {
    updaterStore.set({
      ...updaterStore.get(),
      status: "error",
      error: errorMessage(error)
    });
  }
}

// src/main/app/updater.ts
import { app } from "electron";
import { promises as fs } from "node:fs";
import path from "node:path";
import { getMainWindow } from "./window";
import type {
  UpdateChannel,
  UpdateState
} from "@shared/types/updater";

const DEFAULT_CHANNEL: UpdateChannel = "stable";

interface UpdaterSettings {
  channel: UpdateChannel;
  installedVersion?: string;
}

let settingsPath: string | null = null;
let installedVersion: string | undefined;

let mockFeed: Record<UpdateChannel, string> | null = null;
let downloadTimer: NodeJS.Timeout | null = null;

let state: UpdateState = {
  status: "idle",
  channel: DEFAULT_CHANNEL,
  currentVersion: "0.0.0",
  latestVersion: undefined,
  downloadProgress: 0,
  error: null,
  releaseNotes: "",
  updatedAt: Date.now()
};

function sendState(): void {
  const win = getMainWindow();

  if (win && !win.isDestroyed()) {
    win.webContents.send("updater:state", state);
  }
}

function setState(patch: Partial<UpdateState>): void {
  state = {
    ...state,
    ...patch,
    updatedAt: Date.now()
  };

  sendState();
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function bumpPatch(version: string): string {
  const core = version.split("-")[0];
  const parts = core.split(".").map((part) => Number.parseInt(part, 10));

  if (parts.length !== 3 || parts.some((part) => Number.isNaN(part))) {
    return "0.1.0";
  }

  parts[2] += 1;

  return parts.join(".");
}

function getMockFeed(): Record<UpdateChannel, string> {
  if (!mockFeed) {
    const stable = bumpPatch(app.getVersion());

    mockFeed = {
      stable,
      beta: `${stable}-beta.1`,
      dev: `${stable}-dev.1`
    };
  }

  return mockFeed;
}

function buildReleaseNotes(
  version: string,
  channel: UpdateChannel
): string {
  return `# Kaioken Desktop ${version}

## Channel

${channel}

## Highlights

- Mock update pipeline validation.
- Session recovery and crash reporting hardening.
- Kaioken design system alignment.

## Fixes

- Improved IPC backpressure.
- Redacted provider secrets from engine logs.
- Strengthened workspace sandbox guards.

## Notes

This is a generated skeleton release note used by Phase 6.3.
Real release notes will be fetched from the signed update feed.`;
}

async function loadSettings(): Promise<UpdaterSettings> {
  settingsPath = path.join(
    app.getPath("userData"),
    "updater-settings.json"
  );

  try {
    const raw = await fs.readFile(settingsPath, "utf8");
    const parsed = JSON.parse(raw) as Partial<UpdaterSettings>;

    const channel =
      parsed.channel === "stable" ||
      parsed.channel === "beta" ||
      parsed.channel === "dev"
        ? parsed.channel
        : DEFAULT_CHANNEL;

    installedVersion =
      typeof parsed.installedVersion === "string"
        ? parsed.installedVersion
        : undefined;

    return {
      channel,
      installedVersion
    };
  } catch {
    return {
      channel: DEFAULT_CHANNEL
    };
  }
}

async function saveSettings(
  channel: UpdateChannel,
  installed?: string
): Promise<void> {
  if (!settingsPath) {
    return;
  }

  const payload: UpdaterSettings = {
    channel
  };

  if (installed) {
    payload.installedVersion = installed;
  }

  try {
    await fs.writeFile(
      settingsPath,
      JSON.stringify(payload, null, 2),
      "utf8"
    );
  } catch (error) {
    console.error("[Updater] Failed to persist settings", error);
  }
}

export function getUpdaterState(): UpdateState {
  return state;
}

export async function initUpdater(): Promise<void> {
  const settings = await loadSettings();

  const currentVersion =
    settings.installedVersion ?? app.getVersion();

  setState({
    channel: settings.channel,
    currentVersion,
    latestVersion: undefined,
    status: "idle",
    error: null,
    downloadProgress: 0,
    releaseNotes: buildReleaseNotes(currentVersion, settings.channel)
  });
}

export async function checkForUpdates(): Promise<void> {
  if (
    state.status === "checking" ||
    state.status === "downloading" ||
    state.status === "installing"
  ) {
    return;
  }

  setState({
    status: "checking",
    error: null
  });

  await delay(700);

  const feed = getMockFeed();
  const latest = feed[state.channel];

  const available = latest !== state.currentVersion;

  setState({
    latestVersion: latest,
    status: available ? "available" : "not-available",
    releaseNotes: buildReleaseNotes(latest, state.channel)
  });
}

export async function downloadUpdate(): Promise<void> {
  if (state.status !== "available") {
    return;
  }

  if (downloadTimer) {
    clearInterval(downloadTimer);
    downloadTimer = null;
  }

  setState({
    status: "downloading",
    downloadProgress: 0,
    error: null
  });

  let progress = 0;

  downloadTimer = setInterval(() => {
    progress += 14 + Math.random() * 9;

    if (progress >= 100) {
      if (downloadTimer) {
        clearInterval(downloadTimer);
        downloadTimer = null;
      }

      setState({
        downloadProgress: 100,
        status: "downloaded"
      });
    } else {
      setState({
        downloadProgress: Math.floor(progress)
      });
    }
  }, 180);
}

export async function installUpdate(): Promise<void> {
  if (state.status !== "downloaded") {
    return;
  }

  setState({
    status: "installing",
    error: null
  });

  await delay(900);

  const installed = state.latestVersion ?? state.currentVersion;

  installedVersion = installed;

  await saveSettings(state.channel, installedVersion);

  setState({
    status: "idle",
    currentVersion: installed,
    latestVersion: installed,
    downloadProgress: 0,
    releaseNotes: buildReleaseNotes(installed, state.channel)
  });

  console.log("[Updater] Mock update installed:", installed);
}

export async function setUpdateChannel(
  channel: UpdateChannel
): Promise<void> {
  await saveSettings(channel, installedVersion);

  setState({
    channel,
    status: "idle",
    error: null,
    releaseNotes: buildReleaseNotes(state.currentVersion, channel)
  });
}

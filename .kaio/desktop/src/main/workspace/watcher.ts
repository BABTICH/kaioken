// src/main/workspace/watcher.ts
import fs from "node:fs";
import { BrowserWindow } from "electron";
import { getTrustedRoot } from "./guard";
import type { WorkspaceWatchEvent } from "@shared/types/workspace";

const watchers = new Map<string, fs.FSWatcher>();

export function watchWorkspacePath(
  relativePath: string,
  mainWindow: BrowserWindow | null
): void {
  const root = getTrustedRoot();

  if (!root || !mainWindow || mainWindow.isDestroyed()) {
    return;
  }

  if (watchers.has(relativePath)) {
    return;
  }

  const absolutePath = `${root}/${relativePath}`.replace(
    /\\/g,
    "/"
  );

  try {
    const watcher = fs.watch(
      absolutePath,
      { recursive: false },
      (eventType, filename) => {
        if (!filename) return;

        const kind: WorkspaceWatchEvent["kind"] =
          eventType === "rename" ? "changed" : "changed";

        const event: WorkspaceWatchEvent = {
          path: relativePath,
          kind
        };

        if (!mainWindow.isDestroyed()) {
          mainWindow.webContents.send("workspace:watch", event);
        }
      }
    );

    watchers.set(relativePath, watcher);
  } catch (error) {
    console.warn(
      `[Workspace] Failed to watch path: ${relativePath}`,
      error
    );
  }
}

export function unwatchWorkspacePath(relativePath: string): void {
  const watcher = watchers.get(relativePath);

  if (watcher) {
    watcher.close();
    watchers.delete(relativePath);
  }
}

export function disposeAllWatchers(): void {
  for (const [path, watcher] of watchers) {
    watcher.close();
  }

  watchers.clear();
}

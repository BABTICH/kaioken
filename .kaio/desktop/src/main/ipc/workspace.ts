// src/main/ipc/workspace.ts
import { dialog, ipcMain } from "electron";
import { handleIpc } from "./safe-handler";
import { VoidInputSchema } from "@shared/schema/ipc.zod";
import {
  WorkspaceOpenPathSchema,
  WorkspaceReadFileSchema,
  WorkspaceSearchSchema,
  WorkspaceStatSchema,
  WorkspaceTreeSchema,
  WorkspaceWatchSchema
} from "@shared/schema/workspace.zod";
import {
  clearTrustedRoot,
  getTrustedRoot,
  setTrustedRoot
} from "../workspace/guard";
import {
  readWorkspaceFile,
  readWorkspaceTree,
  statWorkspacePath
} from "../workspace/fs";
import {
  applyGitStatusToTree,
  getGitStatusMap
} from "../workspace/git";
import { searchWorkspace } from "../workspace/search";
import {
  disposeAllWatchers,
  unwatchWorkspacePath,
  watchWorkspacePath
} from "../workspace/watcher";
import { getMainWindow } from "../app/window";
import { assertTrustedSender } from "../app/security";
import path from "node:path";
import type { WorkspaceMeta } from "@shared/types/workspace";

export function registerWorkspaceIpc(): void {
  handleIpc(
    "workspace:openDialog",
    VoidInputSchema,
    async (): Promise<WorkspaceMeta> => {
      const mainWindow = getMainWindow();

      if (!mainWindow) {
        throw new Error("NO_MAIN_WINDOW");
      }

      const result = await dialog.showOpenDialog(mainWindow, {
        properties: ["openDirectory"],
        title: "Select Workspace Root"
      });

      if (result.canceled || result.filePaths.length === 0) {
        throw new Error("DIALOG_CANCELLED");
      }

      const root = result.filePaths[0];

      setTrustedRoot(root);

      return {
        root,
        name: path.basename(root),
        trusted: true
      };
    }
  );

  handleIpc(
    "workspace:openPath",
    WorkspaceOpenPathSchema,
    async (input): Promise<WorkspaceMeta> => {
      setTrustedRoot(input.path);
      const root = path.resolve(input.path);
      return {
        root,
        name: path.basename(root),
        trusted: true
      };
    }
  );

  handleIpc(
    "workspace:current",
    VoidInputSchema,
    async (): Promise<WorkspaceMeta | null> => {
      const root = getTrustedRoot();

      if (!root) {
        return null;
      }

      return {
        root,
        name: path.basename(root),
        trusted: true
      };
    }
  );

  handleIpc(
    "workspace:tree",
    WorkspaceTreeSchema,
    async (input) => {
      const nodes = await readWorkspaceTree(
        input.path,
        input.depth ?? 3
      );

      const gitStatus = await getGitStatusMap();

      return applyGitStatusToTree(nodes, gitStatus);
    }
  );

  handleIpc(
    "workspace:readFile",
    WorkspaceReadFileSchema,
    async (input) => {
      return readWorkspaceFile(input.path);
    }
  );

  handleIpc(
    "workspace:stat",
    WorkspaceStatSchema,
    async (input) => {
      return statWorkspacePath(input.path);
    }
  );

  handleIpc(
    "workspace:search",
    WorkspaceSearchSchema,
    async (input) => {
      return searchWorkspace(input.query, {
        limit: input.limit,
        includeIgnored: input.includeIgnored
      });
    }
  );

  ipcMain.on("workspace:watch", (event, payload) => {
    try {
      assertTrustedSender(event);
      const parsed = WorkspaceWatchSchema.parse(payload);
      const mainWindow = getMainWindow();
      watchWorkspacePath(parsed.path, mainWindow);
    } catch (error) {
      console.error("[IPC] workspace:watch rejected", error);
    }
  });

  ipcMain.on("workspace:unwatch", (event, payload) => {
    try {
      assertTrustedSender(event);
      const parsed = WorkspaceWatchSchema.parse(payload);
      unwatchWorkspacePath(parsed.path);
    } catch (error) {
      console.error("[IPC] workspace:unwatch rejected", error);
    }
  });
}

export function cleanupWorkspace(): void {
  disposeAllWatchers();
  clearTrustedRoot();
}

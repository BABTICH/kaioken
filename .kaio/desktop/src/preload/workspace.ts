// src/preload/workspace.ts
import { ipcRenderer } from "electron";
import { invokeIpc } from "./invoke";
import type {
  WorkspaceApi,
  WorkspaceFileContent,
  WorkspaceMeta,
  WorkspaceSearchOptions,
  WorkspaceSearchResult,
  WorkspaceStat,
  TreeNode as WTreeNode,
  WorkspaceWatchEvent
} from "@shared/types/workspace";

const watchListeners = new Set<
  (event: WorkspaceWatchEvent) => void
>();

ipcRenderer.on(
  "workspace:watch",
  (_event, payload: WorkspaceWatchEvent) => {
    for (const listener of watchListeners) {
      listener(payload);
    }
  }
);

export const workspaceApi: WorkspaceApi = {
  openDialog(): Promise<WorkspaceMeta> {
    return invokeIpc<WorkspaceMeta>("workspace:openDialog");
  },

  openPath(path: string): Promise<WorkspaceMeta> {
    return invokeIpc<WorkspaceMeta>("workspace:openPath", { path });
  },

  current(): Promise<WorkspaceMeta | null> {
    return invokeIpc<WorkspaceMeta | null>("workspace:current");
  },

  tree(
    path: string,
    depth?: number
  ): Promise<WTreeNode[]> {
    return invokeIpc<WTreeNode[]>("workspace:tree", {
      path,
      depth
    });
  },

  readFile(path: string): Promise<WorkspaceFileContent> {
    return invokeIpc<WorkspaceFileContent>(
      "workspace:readFile",
      { path }
    );
  },

  stat(path: string): Promise<WorkspaceStat> {
    return invokeIpc<WorkspaceStat>("workspace:stat", {
      path
    });
  },

  search(
    query: string,
    opts?: WorkspaceSearchOptions
  ): Promise<WorkspaceSearchResult[]> {
    return invokeIpc<WorkspaceSearchResult[]>(
      "workspace:search",
      {
        query,
        ...opts
      }
    );
  },

  watch(path: string): void {
    ipcRenderer.send("workspace:watch", { path });
  },

  unwatch(path: string): void {
    ipcRenderer.send("workspace:unwatch", { path });
  },

  onWatchEvent(
    cb: (event: WorkspaceWatchEvent) => void
  ): () => void {
    watchListeners.add(cb);

    return () => {
      watchListeners.delete(cb);
    };
  }
};

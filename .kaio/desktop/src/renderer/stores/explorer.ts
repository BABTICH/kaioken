// src/renderer/stores/explorer.ts
import { atom } from "nanostores";
import { addRecentProject } from "./recent-projects";
import type {
  WorkspaceMeta,
  WorkspaceTreeNode
} from "@shared/types/workspace";

export interface ContextMenuState {
  x: number;
  y: number;
  node: WorkspaceTreeNode;
}

export interface ExplorerState {
  workspace: WorkspaceMeta | null;
  tree: WorkspaceTreeNode[];
  expandedNodes: Set<string>;
  selectedNodeId: string | null;
  loading: boolean;
  error: string | null;
  openFiles: string[];
  activeFilePath: string | null;
  openFileContents: Record<string, string>;
  contextMenu: ContextMenuState | null;
}

const initialState: ExplorerState = {
  workspace: null,
  tree: [],
  expandedNodes: new Set(),
  selectedNodeId: null,
  loading: false,
  error: null,
  openFiles: [],
  activeFilePath: null,
  openFileContents: {},
  contextMenu: null
};

export const explorerStore = atom<ExplorerState>(initialState);
export const workspaceStore = explorerStore;

function set(patch: Partial<ExplorerState>): void {
  explorerStore.set({
    ...explorerStore.get(),
    ...patch
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export async function openWorkspace(): Promise<void> {
  set({ loading: true, error: null });

  try {
    const meta = await window.api.workspace.openDialog();

    set({
      workspace: meta,
      loading: false
    });

    addRecentProject(meta.root, meta.name);
    await loadTree(".");
    initWorkspaceWatcher();
  } catch (error) {
    const msg = errorMessage(error);

    if (msg === "DIALOG_CANCELLED") {
      set({ loading: false });
      return;
    }

    set({
      loading: false,
      error: msg
    });
  }
}

export async function openWorkspacePath(targetPath: string): Promise<void> {
  set({ loading: true, error: null });

  try {
    const meta = await window.api.workspace.openPath(targetPath);

    set({
      workspace: meta,
      loading: false
    });

    addRecentProject(meta.root, meta.name);
    await loadTree(".");
    initWorkspaceWatcher();
  } catch (error) {
    const msg = errorMessage(error);
    set({
      loading: false,
      error: msg
    });
  }
}

export async function restoreWorkspace(): Promise<void> {
  try {
    const meta = await window.api.workspace.current();

    if (meta) {
      set({ workspace: meta });
      addRecentProject(meta.root, meta.name);
      await loadTree(".");
      initWorkspaceWatcher();
    }
  } catch (error) {
    console.error("[explorerStore] Failed to restore", error);
  }
}

export async function loadTree(path: string = "."): Promise<void> {
  set({ loading: true, error: null });

  try {
    const nodes = await window.api.workspace.tree(path, 4);

    set({
      tree: nodes,
      loading: false
    });
  } catch (error) {
    set({
      loading: false,
      error: errorMessage(error)
    });
  }
}

export function toggleNode(nodeId: string): void {
  const state = explorerStore.get();
  const next = new Set(state.expandedNodes);

  if (next.has(nodeId)) {
    next.delete(nodeId);
  } else {
    next.add(nodeId);
  }

  set({ expandedNodes: next });
}

export function selectNode(nodeId: string | null): void {
  set({ selectedNodeId: nodeId });
}

export async function openFile(relPath: string): Promise<void> {
  const state = explorerStore.get();

  const already = state.openFiles.includes(relPath);

  const openFiles = already
    ? state.openFiles
    : [...state.openFiles, relPath];

  set({
    openFiles,
    activeFilePath: relPath,
    selectedNodeId: relPath
  });

  if (!explorerStore.get().openFileContents[relPath]) {
    try {
      const file = await window.api.workspace.readFile(relPath);

      set({
        openFileContents: {
          ...explorerStore.get().openFileContents,
          [relPath]: file.content
        }
      });
    } catch {
      // Binary or oversized file; skip content cache.
    }
  }
}

export function closeFile(relPath: string): void {
  const state = explorerStore.get();

  const openFiles = state.openFiles.filter((p) => p !== relPath);

  let activeFilePath = state.activeFilePath;

  if (activeFilePath === relPath) {
    activeFilePath = openFiles[openFiles.length - 1] ?? null;
  }

  const openFileContents = { ...state.openFileContents };
  delete openFileContents[relPath];

  set({
    openFiles,
    activeFilePath,
    openFileContents
  });
}

export function setActiveFile(relPath: string): void {
  set({
    activeFilePath: relPath,
    selectedNodeId: relPath
  });
}

export function openContextMenu(
  x: number,
  y: number,
  node: WorkspaceTreeNode
): void {
  set({
    contextMenu: { x, y, node }
  });
}

export function closeContextMenu(): void {
  set({ contextMenu: null });
}

let watchUnsub: (() => void) | null = null;
let refreshTimer: number | undefined;

export function initWorkspaceWatcher(): void {
  if (watchUnsub) {
    return;
  }

  watchUnsub = window.api.workspace.onWatchEvent(() => {
    if (refreshTimer) {
      window.clearTimeout(refreshTimer);
    }

    refreshTimer = window.setTimeout(() => {
      void loadTree(".");
    }, 300);
  });

  window.api.workspace.watch(".");
}

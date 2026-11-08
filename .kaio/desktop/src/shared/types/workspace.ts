// src/shared/types/workspace.ts

export interface WorkspaceMeta {
  root: string;
  name: string;
  trusted: boolean;
}

export interface WorkspaceTreeNode {
  id: string;
  path: string;
  name: string;
  type: "file" | "directory";
  children?: WorkspaceTreeNode[];
  ignored?: boolean;
  gitStatus?: "modified" | "untracked" | "added" | "deleted" | "renamed";
}

export type TreeNode = WorkspaceTreeNode;

export interface WorkspaceFileContent {
  path: string;
  content: string;
  encoding: "utf8";
  size: number;
  mtime: number;
}

export interface WorkspaceStat {
  path: string;
  type: "file" | "directory";
  size: number;
  mtime: number;
}

export interface WorkspaceSearchOptions {
  limit?: number;
  includeIgnored?: boolean;
}

export interface WorkspaceSearchResult {
  path: string;
  score: number;
  excerpt?: string;
}

export interface WorkspaceWatchEvent {
  path: string;
  kind: "created" | "changed" | "deleted";
}

export interface WorkspaceApi {
  openDialog(): Promise<WorkspaceMeta>;
  openPath(path: string): Promise<WorkspaceMeta>;
  current(): Promise<WorkspaceMeta | null>;
  tree(path: string, depth?: number): Promise<WorkspaceTreeNode[]>;
  readFile(path: string): Promise<WorkspaceFileContent>;
  stat(path: string): Promise<WorkspaceStat>;
  search(
    query: string,
    opts?: WorkspaceSearchOptions
  ): Promise<WorkspaceSearchResult[]>;
  watch(path: string): void;
  unwatch(path: string): void;
  onWatchEvent(cb: (event: WorkspaceWatchEvent) => void): () => void;
}

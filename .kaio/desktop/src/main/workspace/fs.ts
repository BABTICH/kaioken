// src/main/workspace/fs.ts
import fs from "node:fs/promises";
import path from "node:path";
import { assertInsideWorkspace } from "./guard";
import type {
  WorkspaceFileContent,
  WorkspaceStat,
  WorkspaceTreeNode
} from "@shared/types/workspace";

const MAX_READ_SIZE_BYTES = 5 * 1024 * 1024; // 5MB
const IGNORED_NAMES = new Set([
  "node_modules",
  ".git",
  ".DS_Store",
  "Thumbs.db",
  ".next",
  "dist",
  "out",
  ".turbo",
  ".cache"
]);

export async function readWorkspaceFile(
  relativePath: string
): Promise<WorkspaceFileContent> {
  const absolutePath = assertInsideWorkspace(relativePath);

  const stat = await fs.stat(absolutePath);

  if (!stat.isFile()) {
    throw new Error("PATH_IS_NOT_FILE");
  }

  if (stat.size > MAX_READ_SIZE_BYTES) {
    throw new Error("FILE_TOO_LARGE");
  }

  const content = await fs.readFile(absolutePath, "utf8");

  return {
    path: relativePath,
    content,
    encoding: "utf8",
    size: stat.size,
    mtime: stat.mtimeMs
  };
}

export async function statWorkspacePath(
  relativePath: string
): Promise<WorkspaceStat> {
  const absolutePath = assertInsideWorkspace(relativePath);
  const stat = await fs.stat(absolutePath);

  return {
    path: relativePath,
    type: stat.isDirectory() ? "directory" : "file",
    size: stat.size,
    mtime: stat.mtimeMs
  };
}

export async function readWorkspaceTree(
  relativePath: string,
  maxDepth: number = 3,
  currentDepth: number = 0
): Promise<WorkspaceTreeNode[]> {
  const absolutePath = assertInsideWorkspace(relativePath);

  if (currentDepth >= maxDepth) {
    return [];
  }

  const entries = await fs.readdir(absolutePath, {
    withFileTypes: true
  });

  const nodes: WorkspaceTreeNode[] = [];

  for (const entry of entries) {
    if (IGNORED_NAMES.has(entry.name)) {
      continue;
    }

    if (entry.name.startsWith(".")) {
      continue;
    }

    const entryRelativePath = path.join(relativePath, entry.name);
    const isDirectory = entry.isDirectory();

    const node: WorkspaceTreeNode = {
      id: entryRelativePath,
      path: entryRelativePath,
      name: entry.name,
      type: isDirectory ? "directory" : "file"
    };

    if (isDirectory && currentDepth < maxDepth - 1) {
      try {
        node.children = await readWorkspaceTree(
          entryRelativePath,
          maxDepth,
          currentDepth + 1
        );
      } catch {
        node.children = [];
      }
    }

    nodes.push(node);
  }

  // Sort: directories first, then alphabetical
  nodes.sort((a, b) => {
    if (a.type !== b.type) {
      return a.type === "directory" ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });

  return nodes;
}

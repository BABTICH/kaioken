// src/main/workspace/guard.ts
import path from "node:path";
import fs from "node:fs";

let trustedRoot: string | null = null;

export function setTrustedRoot(root: string): void {
  const resolved = path.resolve(root);

  if (!fs.existsSync(resolved)) {
    throw new Error(`Workspace root does not exist: ${resolved}`);
  }

  const stat = fs.statSync(resolved);

  if (!stat.isDirectory()) {
    throw new Error(`Workspace root is not a directory: ${resolved}`);
  }

  trustedRoot = resolved;
}

export function getTrustedRoot(): string | null {
  return trustedRoot;
}

export function clearTrustedRoot(): void {
  trustedRoot = null;
}

/**
 * Resolves a target path against the trusted root and ensures
 * it cannot escape the workspace boundary via traversal or symlinks.
 */
export function assertInsideWorkspace(target: string): string {
  if (!trustedRoot) {
    throw new Error("WORKSPACE_NOT_OPEN");
  }

  const resolvedTarget = path.resolve(trustedRoot, target);

  // Fast path check before realpath
  if (
    !resolvedTarget.startsWith(trustedRoot + path.sep) &&
    resolvedTarget !== trustedRoot
  ) {
    throw new Error("PATH_DENIED");
  }

  // Slow path: resolve symlinks to prevent symlink escape attacks
  try {
    const realRoot = fs.realpathSync(trustedRoot);
    const realTarget = fs.realpathSync(resolvedTarget);

    if (
      !realTarget.startsWith(realRoot + path.sep) &&
      realTarget !== realRoot
    ) {
      throw new Error("PATH_DENIED");
    }

    return realTarget;
  } catch (error) {
    if (
      error instanceof Error &&
      (error.message === "PATH_DENIED" ||
        error.message === "WORKSPACE_NOT_OPEN")
    ) {
      throw error;
    }

    // If realpath fails (e.g., file doesn't exist yet), fall back to fast path
    return resolvedTarget;
  }
}

export function getRelativePath(absolutePath: string): string {
  if (!trustedRoot) {
    throw new Error("WORKSPACE_NOT_OPEN");
  }

  return path.relative(trustedRoot, absolutePath);
}

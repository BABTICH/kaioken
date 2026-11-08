// src/main/workspace/git.ts
import simpleGit from "simple-git";
import { getTrustedRoot } from "./guard";
import type { WorkspaceTreeNode } from "@shared/types/workspace";

export async function getGitStatusMap(): Promise<
  Map<string, WorkspaceTreeNode["gitStatus"]>
> {
  const root = getTrustedRoot();

  if (!root) {
    return new Map();
  }

  const statusMap = new Map<
    string,
    WorkspaceTreeNode["gitStatus"]
  >();

  try {
    const git = simpleGit(root);
    const isRepo = await git.checkIsRepo();

    if (!isRepo) {
      return statusMap;
    }

    const status = await git.status();

    for (const file of status.modified) {
      statusMap.set(file, "modified");
      statusMap.set(file.replace(/\\/g, "/"), "modified");
    }

    for (const file of status.not_added) {
      statusMap.set(file, "untracked");
      statusMap.set(file.replace(/\\/g, "/"), "untracked");
    }

    for (const file of status.created) {
      statusMap.set(file, "added");
      statusMap.set(file.replace(/\\/g, "/"), "added");
    }

    for (const file of status.deleted) {
      statusMap.set(file, "deleted");
      statusMap.set(file.replace(/\\/g, "/"), "deleted");
    }

    for (const file of status.renamed) {
      statusMap.set(file.to, "renamed");
      statusMap.set(file.to.replace(/\\/g, "/"), "renamed");
    }
  } catch (error) {
    console.warn("[Workspace] Failed to read git status", error);
  }

  return statusMap;
}

export function applyGitStatusToTree(
  nodes: WorkspaceTreeNode[],
  statusMap: Map<string, WorkspaceTreeNode["gitStatus"]>
): WorkspaceTreeNode[] {
  return nodes.map((node) => {
    const enriched = { ...node };

    const normalizedPath = node.path.replace(/\\/g, "/");
    const status = statusMap.get(node.path) ?? statusMap.get(normalizedPath);

    if (status) {
      enriched.gitStatus = status;
    }

    if (enriched.children) {
      enriched.children = applyGitStatusToTree(
        enriched.children,
        statusMap
      );
    }

    return enriched;
  });
}

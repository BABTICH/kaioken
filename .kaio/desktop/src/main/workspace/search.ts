// src/main/workspace/search.ts
import fs from "node:fs/promises";
import path from "node:path";
import { getTrustedRoot } from "./guard";
import type { WorkspaceSearchResult } from "@shared/types/workspace";

const IGNORED_NAMES = new Set([
  "node_modules",
  ".git",
  ".DS_Store",
  "Thumbs.db",
  ".next",
  "dist",
  "out",
  ".turbo",
  ".cache",
  "build",
  "coverage"
]);

const MAX_FILES = 20_000;

async function collectFiles(
  dir: string,
  base: string,
  results: string[],
  includeIgnored: boolean
): Promise<void> {
  if (results.length >= MAX_FILES) {
    return;
  }

  let entries;

  try {
    entries = await fs.readdir(dir, { withFileTypes: true });
  } catch {
    return;
  }

  for (const entry of entries) {
    if (results.length >= MAX_FILES) {
      return;
    }

    const isIgnoredName = IGNORED_NAMES.has(entry.name);
    const isHidden = entry.name.startsWith(".");

    const skip =
      !includeIgnored && (isIgnoredName || isHidden);

    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      if (!skip) {
        await collectFiles(fullPath, base, results, includeIgnored);
      }
    } else if (entry.isFile()) {
      const rel = path.relative(base, fullPath);
      results.push(rel);
    }
  }
}

function fuzzyScore(query: string, target: string): number {
  const q = query.toLowerCase();
  const t = target.toLowerCase();

  if (q.length === 0) {
    return 1;
  }

  const idx = t.indexOf(q);

  if (idx !== -1) {
    return 1000 - idx + q.length * 10;
  }

  let qi = 0;
  let score = 0;
  let streak = 0;

  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      qi++;
      streak++;
      score += 1 + streak * 2;
    } else {
      streak = 0;
    }
  }

  if (qi < q.length) {
    return -1;
  }

  return score;
}

export async function searchWorkspace(
  query: string,
  opts?: { limit?: number; includeIgnored?: boolean }
): Promise<WorkspaceSearchResult[]> {
  const root = getTrustedRoot();

  if (!root) {
    return [];
  }

  const files: string[] = [];

  await collectFiles(root, root, files, opts?.includeIgnored ?? false);

  const limit = opts?.limit ?? 20;

  if (!query.trim()) {
    return files.slice(0, limit).map((p) => ({
      path: p,
      score: 0
    }));
  }

  return files
    .map((p) => ({
      path: p,
      score: fuzzyScore(query, p)
    }))
    .filter((r) => r.score >= 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit);
}

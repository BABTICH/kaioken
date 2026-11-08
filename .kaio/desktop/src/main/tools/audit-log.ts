// src/main/tools/audit-log.ts
import { app } from "electron";
import { promises as fs } from "node:fs";
import path from "node:path";
import type { AuditEntry } from "@shared/types/tools";

let entries: AuditEntry[] = [];
let filePath: string | null = null;
let loaded = false;

export async function initAuditLog(): Promise<void> {
  if (loaded) {
    return;
  }

  filePath = path.join(app.getPath("userData"), "kaioken-audit.json");

  try {
    const raw = await fs.readFile(filePath, "utf8");
    const parsed = JSON.parse(raw);

    if (Array.isArray(parsed)) {
      entries = parsed as AuditEntry[];
    }
  } catch {
    entries = [];
  }

  loaded = true;
}

export async function recordAudit(entry: AuditEntry): Promise<void> {
  entries.unshift(entry);

  if (entries.length > 500) {
    entries = entries.slice(0, 500);
  }

  if (!filePath) {
    return;
  }

  try {
    await fs.writeFile(
      filePath,
      JSON.stringify(entries, null, 2),
      "utf8"
    );
  } catch (error) {
    console.error("[AuditLog] Failed to persist audit entry", error);
  }
}

export function getAuditEntries(limit = 100): AuditEntry[] {
  return entries.slice(0, limit);
}

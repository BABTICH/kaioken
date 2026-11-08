// src/main/utils/crash-reporter.ts
import { app } from "electron";
import { promises as fs } from "node:fs";
import path from "node:path";

export interface CrashReport {
  id: string;
  timestamp: number;
  message: string;
  stack?: string;
  componentStack?: string;
  userAgent?: string;
}

let crashLogPath: string | null = null;

export function initCrashReporter(): void {
  crashLogPath = path.join(app.getPath("userData"), "crash-reports.json");
  console.log("[CrashReporter] Initialized at:", crashLogPath);
}

export async function recordCrash(report: CrashReport): Promise<void> {
  if (!crashLogPath) return;

  try {
    let reports: CrashReport[] = [];

    try {
      const raw = await fs.readFile(crashLogPath, "utf8");
      reports = JSON.parse(raw);
    } catch {
      reports = [];
    }

    reports.unshift(report);

    // Keep only the last 20 crash reports
    if (reports.length > 20) {
      reports = reports.slice(0, 20);
    }

    await fs.writeFile(crashLogPath, JSON.stringify(reports, null, 2), "utf8");
    console.error("[CrashReporter] Crash recorded:", report.message);
  } catch (err) {
    console.error("[CrashReporter] Failed to write crash report:", err);
  }
}

export async function getCrashReports(): Promise<CrashReport[]> {
  if (!crashLogPath) return [];

  try {
    const raw = await fs.readFile(crashLogPath, "utf8");
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

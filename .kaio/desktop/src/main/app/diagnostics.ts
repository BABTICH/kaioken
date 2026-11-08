// src/main/app/diagnostics.ts
import { app } from "electron";
import { promises as fs } from "node:fs";
import path from "node:path";
import { getMainWindow } from "./window";
import type {
  DiagnosticsState,
  MainMemoryMetrics,
  StartupMetrics
} from "@shared/types/diagnostics";

const startAt = Date.now();

let windowCreatedAt: number | undefined;
let windowShownAt: number | undefined;
let rendererReadyAt: number | undefined;

const warnings: string[] = [];
const warningSet = new Set<string>();

let guardrailStarted = false;

function addWarning(message: string): void {
  if (warningSet.has(message)) {
    return;
  }

  warningSet.add(message);
  warnings.push(message);

  if (warnings.length > 50) {
    warnings.shift();
  }
}

function startupBudgetMs(): number {
  return app.isPackaged ? 2500 : 8000;
}

function toMB(value: number): number {
  return Number((value / 1024 / 1024).toFixed(1));
}

export function markWindowCreated(): void {
  if (!windowCreatedAt) {
    windowCreatedAt = Date.now();
  }
}

export function markWindowShown(): void {
  if (!windowShownAt) {
    windowShownAt = Date.now();
    sendState();
  }
}

export function markRendererReady(): void {
  if (rendererReadyAt) {
    return;
  }

  rendererReadyAt = Date.now();

  evaluateStartupBudget();
  sendState();
}

export function getStartupMetrics(): StartupMetrics {
  const budgetMs = startupBudgetMs();

  const windowShownMs =
    windowShownAt !== undefined
      ? windowShownAt - startAt
      : windowCreatedAt !== undefined
        ? windowCreatedAt - startAt
        : undefined;

  const rendererReadyMs =
    rendererReadyAt !== undefined
      ? rendererReadyAt - startAt
      : undefined;

  return {
    appStart: startAt,
    windowCreated: windowCreatedAt,
    windowShown: windowShownAt,
    rendererReady: rendererReadyAt,
    windowShownMs,
    rendererReadyMs,
    budgetMs
  };
}

export function getMemoryMetrics(): MainMemoryMetrics {
  const memory = process.memoryUsage();

  return {
    rssMB: toMB(memory.rss),
    heapTotalMB: toMB(memory.heapTotal),
    heapUsedMB: toMB(memory.heapUsed),
    externalMB: toMB(memory.external)
  };
}

function evaluateStartupBudget(): void {
  const metrics = getStartupMetrics();

  if (
    metrics.rendererReadyMs !== undefined &&
    metrics.rendererReadyMs > metrics.budgetMs
  ) {
    addWarning(
      `Startup budget exceeded: ${metrics.rendererReadyMs}ms > ${metrics.budgetMs}ms`
    );
  }
}

function evaluateMemoryBudget(): void {
  const memory = getMemoryMetrics();

  if (memory.heapUsedMB > 500) {
    addWarning(
      `Main heap usage high: ${memory.heapUsedMB.toFixed(1)} MB`
    );
  }

  if (memory.rssMB > 900) {
    addWarning(
      `Main RSS usage high: ${memory.rssMB.toFixed(1)} MB`
    );
  }
}

export function startMemoryGuardrail(): void {
  if (guardrailStarted) {
    return;
  }

  guardrailStarted = true;

  const timer = setInterval(() => {
    evaluateMemoryBudget();
    sendState();
  }, 10_000);

  timer.unref?.();
}

export function getDiagnosticsState(): DiagnosticsState {
  return {
    startup: getStartupMetrics(),
    memory: getMemoryMetrics(),
    warnings: [...warnings],
    updatedAt: Date.now()
  };
}

export function sendState(): void {
  const win = getMainWindow();

  if (win && !win.isDestroyed()) {
    win.webContents.send("diagnostics:state", getDiagnosticsState());
  }
}

export function startDiagnostics(): void {
  startMemoryGuardrail();
  evaluateMemoryBudget();
  sendState();
}

export async function saveCertificationReport(
  report: unknown
): Promise<void> {
  const filePath = path.join(
    app.getPath("userData"),
    "certification-report.json"
  );

  await fs.writeFile(
    filePath,
    JSON.stringify(report, null, 2),
    "utf8"
  );
}

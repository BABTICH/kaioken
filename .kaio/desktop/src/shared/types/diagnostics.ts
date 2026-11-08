// src/shared/types/diagnostics.ts

export type Unsubscribe = () => void;

export interface StartupMetrics {
  appStart: number;
  windowCreated?: number;
  windowShown?: number;
  rendererReady?: number;
  windowShownMs?: number;
  rendererReadyMs?: number;
  budgetMs: number;
}

export interface MainMemoryMetrics {
  rssMB: number;
  heapTotalMB: number;
  heapUsedMB: number;
  externalMB: number;
}

export interface DiagnosticsState {
  startup: StartupMetrics;
  memory: MainMemoryMetrics;
  warnings: string[];
  updatedAt: number;
}

export type CheckStatus =
  | "idle"
  | "running"
  | "pass"
  | "warn"
  | "fail";

export type CertificationOverall =
  | "idle"
  | "running"
  | "pass"
  | "warn"
  | "fail";

export interface CertificationCheck {
  id: string;
  label: string;
  status: CheckStatus;
  detail: string;
}

export interface CertificationMetrics {
  startup?: StartupMetrics;
  mainMemory?: MainMemoryMetrics;
  rendererMemoryMB?: number | null;
  frameDrops?: number;
  longTasks?: number;
}

export interface CertificationReport {
  runAt: number;
  overall: CertificationOverall;
  checks: CertificationCheck[];
  metrics: CertificationMetrics;
  warnings: string[];
}

export interface DiagnosticsApi {
  getState(): Promise<DiagnosticsState>;

  getMemory(): Promise<MainMemoryMetrics>;

  getStartup(): Promise<StartupMetrics>;

  markRendererReady(): Promise<void>;

  saveReport(report: CertificationReport): Promise<void>;

  onState(cb: (state: DiagnosticsState) => void): Unsubscribe;
}

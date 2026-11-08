// src/renderer/stores/diagnostics.ts
import { atom } from "nanostores";
import type {
  CertificationCheck,
  CertificationOverall,
  CertificationReport,
  DiagnosticsState
} from "@shared/types/diagnostics";

export interface DiagnosticsStoreState {
  main: DiagnosticsState | null;
  rendererMemoryMB: number | null;
  checks: CertificationCheck[];
  running: boolean;
  overall: CertificationOverall;
  lastReport: CertificationReport | null;
}

function createInitialChecks(): CertificationCheck[] {
  return [
    {
      id: "security",
      label: "Security sandbox",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "engine",
      label: "Kaiopi engine",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "workspace",
      label: "Workspace API",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "updater",
      label: "Updater service",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "crash",
      label: "Crash reporter",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "startup",
      label: "Startup budget",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "memory",
      label: "Memory guardrails",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "frames",
      label: "Frame budget",
      status: "idle",
      detail: "Not run"
    },
    {
      id: "longtasks",
      label: "Long tasks",
      status: "idle",
      detail: "Not run"
    }
  ];
}

const initialState: DiagnosticsStoreState = {
  main: null,
  rendererMemoryMB: null,
  checks: createInitialChecks(),
  running: false,
  overall: "idle",
  lastReport: null
};

export const diagnosticsStore =
  atom<DiagnosticsStoreState>(initialState);

let initialized = false;
let memoryInterval: number | undefined;

function set(patch: Partial<DiagnosticsStoreState>): void {
  diagnosticsStore.set({
    ...diagnosticsStore.get(),
    ...patch
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function readRendererMemoryMB(): number | null {
  const perf = performance as unknown as {
    memory?: {
      usedJSHeapSize?: number;
    };
  };

  if (
    perf.memory &&
    typeof perf.memory.usedJSHeapSize === "number"
  ) {
    return Number(
      (perf.memory.usedJSHeapSize / 1024 / 1024).toFixed(1)
    );
  }

  return null;
}

export function initDiagnosticsBridge(): void {
  if (initialized) {
    return;
  }

  initialized = true;

  window.api.diagnostics.onState((state) => {
    set({ main: state });
  });

  void window.api.diagnostics
    .getState()
    .then((state) => {
      set({ main: state });
    })
    .catch(() => {
      // Diagnostics unavailable.
    });

  void window.api.diagnostics
    .markRendererReady()
    .catch(() => {
      // Ignore renderer-ready race.
    });

  set({
    rendererMemoryMB: readRendererMemoryMB()
  });

  if (memoryInterval) {
    window.clearInterval(memoryInterval);
  }

  memoryInterval = window.setInterval(() => {
    set({
      rendererMemoryMB: readRendererMemoryMB()
    });
  }, 5000);
}

function monitorFrameDrops(durationMs: number): Promise<number> {
  return new Promise((resolve) => {
    let drops = 0;
    let last = performance.now();
    let active = true;

    const loop = (now: number) => {
      if (!active) {
        return;
      }

      const delta = now - last;
      last = now;

      if (delta > 66) {
        drops += 1;
      }

      window.requestAnimationFrame(loop);
    };

    window.requestAnimationFrame(loop);

    window.setTimeout(() => {
      active = false;
      resolve(drops);
    }, durationMs);
  });
}

function monitorLongTasks(durationMs: number): Promise<number> {
  return new Promise((resolve) => {
    let count = 0;

    const PO = (
      window as unknown as {
        PerformanceObserver?: new (
          callback: (list: { getEntries: () => unknown[] }) => void
        ) => {
          observe: (options: unknown) => void;
          disconnect: () => void;
        };
      }
    ).PerformanceObserver;

    if (!PO) {
      window.setTimeout(() => resolve(0), durationMs);
      return;
    }

    let observer: {
      observe: (options: unknown) => void;
      disconnect: () => void;
    };

    try {
      observer = new PO((list) => {
        count += list.getEntries().length;
      });

      observer.observe({
        entryTypes: ["longtask"]
      });
    } catch {
      window.setTimeout(() => resolve(0), durationMs);
      return;
    }

    window.setTimeout(() => {
      try {
        observer.disconnect();
      } catch {
        // Ignore.
      }

      resolve(count);
    }, durationMs);
  });
}

export async function runCertification(): Promise<void> {
  const current = diagnosticsStore.get();

  if (current.running) {
    return;
  }

  let checks: CertificationCheck[] = createInitialChecks().map((check) => ({
    ...check,
    status: "running" as const,
    detail: "Running"
  }));

  const updateCheck = (
    id: string,
    patch: Partial<CertificationCheck>
  ) => {
    checks = checks.map((check) =>
      check.id === id
        ? {
            ...check,
            ...patch
          }
        : check
    );

    set({
      checks: [...checks]
    });
  };

  set({
    running: true,
    overall: "running",
    checks: [...checks],
    rendererMemoryMB: readRendererMemoryMB()
  });

  const framePromise = monitorFrameDrops(3000);
  const longTaskPromise = monitorLongTasks(3000);

  const metrics: CertificationReport["metrics"] = {};

  // Security sandbox
  const flags = window as unknown as {
    __KAIOKEN_CONTEXT_ISOLATION__?: boolean;
    __KAIOKEN_NODE_INTEGRATION__?: boolean;
    __KAIOKEN_SANDBOX__?: boolean;
  };

  const contextIsolation = flags.__KAIOKEN_CONTEXT_ISOLATION__ === true;
  const nodeIntegration = flags.__KAIOKEN_NODE_INTEGRATION__ === true;
  const sandbox = flags.__KAIOKEN_SANDBOX__ === true;

  const securityPass =
    contextIsolation && !nodeIntegration && sandbox;

  updateCheck("security", {
    status: securityPass ? "pass" : "fail",
    detail: `contextIsolation=${contextIsolation}, nodeIntegration=${nodeIntegration}, sandbox=${sandbox}`
  });

  // Engine
  try {
    const engine = await window.api.engine.getState();
    const engineState = String(engine.state);

    if (engineState === "ready") {
      updateCheck("engine", {
        status: "pass",
        detail: engineState
      });
    } else if (engineState === "starting") {
      updateCheck("engine", {
        status: "warn",
        detail: engineState
      });
    } else {
      updateCheck("engine", {
        status: "fail",
        detail: engineState
      });
    }
  } catch (error) {
    updateCheck("engine", {
      status: "fail",
      detail: errorMessage(error)
    });
  }

  // Workspace
  try {
    const workspace = await window.api.workspace.current();

    if (workspace) {
      updateCheck("workspace", {
        status: "pass",
        detail: workspace.root
      });
    } else {
      updateCheck("workspace", {
        status: "warn",
        detail: "No workspace open"
      });
    }
  } catch (error) {
    updateCheck("workspace", {
      status: "fail",
      detail: errorMessage(error)
    });
  }

  // Updater
  try {
    const updater = await window.api.updater.getState();

    updateCheck("updater", {
      status: "pass",
      detail: `channel ${updater.channel} / status ${updater.status}`
    });
  } catch (error) {
    updateCheck("updater", {
      status: "fail",
      detail: errorMessage(error)
    });
  }

  // Crash reporter
  try {
    const reports = await window.api.crash.getReports();

    const recentCrash =
      reports.length > 0 &&
      Date.now() - reports[0].timestamp < 5 * 60 * 1000;

    updateCheck("crash", {
      status: recentCrash ? "warn" : "pass",
      detail: recentCrash
        ? "Recent crash detected"
        : `${reports.length} crash reports`
    });
  } catch (error) {
    updateCheck("crash", {
      status: "fail",
      detail: errorMessage(error)
    });
  }

  // Startup budget
  try {
    const startup = await window.api.diagnostics.getStartup();

    metrics.startup = startup;

    const ready = startup.rendererReadyMs;
    const budget = startup.budgetMs;

    if (ready === undefined) {
      updateCheck("startup", {
        status: "warn",
        detail: "Renderer ready not recorded yet"
      });
    } else if (ready <= budget) {
      updateCheck("startup", {
        status: "pass",
        detail: `${ready}ms / budget ${budget}ms`
      });
    } else if (ready <= budget * 1.5) {
      updateCheck("startup", {
        status: "warn",
        detail: `${ready}ms / budget ${budget}ms`
      });
    } else {
      updateCheck("startup", {
        status: "fail",
        detail: `${ready}ms / budget ${budget}ms`
      });
    }
  } catch (error) {
    updateCheck("startup", {
      status: "fail",
      detail: errorMessage(error)
    });
  }

  // Memory guardrails
  try {
    const mainMemory = await window.api.diagnostics.getMemory();

    metrics.mainMemory = mainMemory;

    const rendererMemory = readRendererMemoryMB();
    metrics.rendererMemoryMB = rendererMemory;

    let status: CertificationCheck["status"] = "pass";

    if (
      mainMemory.heapUsedMB > 700 ||
      mainMemory.rssMB > 1200
    ) {
      status = "fail";
    } else if (
      mainMemory.heapUsedMB > 500 ||
      mainMemory.rssMB > 900
    ) {
      status = "warn";
    }

    if (rendererMemory === null) {
      if (status === "pass") {
        status = "warn";
      }
    } else if (rendererMemory > 500) {
      status = "fail";
    } else if (rendererMemory > 350 && status !== "fail") {
      status = "warn";
    }

    updateCheck("memory", {
      status,
      detail: `main heap ${mainMemory.heapUsedMB.toFixed(1)} MB / renderer ${
        rendererMemory === null
          ? "n/a"
          : `${rendererMemory.toFixed(1)} MB`
      }`
    });
  } catch (error) {
    updateCheck("memory", {
      status: "fail",
      detail: errorMessage(error)
    });
  }

  // Frame budget
  const frameDrops = await framePromise;
  metrics.frameDrops = frameDrops;

  updateCheck("frames", {
    status:
      frameDrops <= 10
        ? "pass"
        : frameDrops <= 30
          ? "warn"
          : "fail",
    detail: `${frameDrops} dropped frames in 3s`
  });

  // Long tasks
  const longTasks = await longTaskPromise;
  metrics.longTasks = longTasks;

  updateCheck("longtasks", {
    status:
      longTasks <= 3
        ? "pass"
        : longTasks <= 8
          ? "warn"
          : "fail",
    detail: `${longTasks} long tasks in 3s`
  });

  const hasFail = checks.some((check) => check.status === "fail");
  const hasWarn = checks.some((check) => check.status === "warn");

  const overall: CertificationOverall = hasFail
    ? "fail"
    : hasWarn
      ? "warn"
      : "pass";

  const report: CertificationReport = {
    runAt: Date.now(),
    overall,
    checks,
    metrics,
    warnings: diagnosticsStore.get().main?.warnings ?? []
  };

  set({
    running: false,
    overall,
    checks: [...checks],
    lastReport: report
  });

  try {
    await window.api.diagnostics.saveReport(report);
  } catch {
    // Report persistence is best-effort.
  }
}

export async function copyCertificationReport(): Promise<void> {
  const report = diagnosticsStore.get().lastReport;

  if (!report) {
    return;
  }

  try {
    await navigator.clipboard.writeText(
      JSON.stringify(report, null, 2)
    );
  } catch {
    // Clipboard unavailable.
  }
}

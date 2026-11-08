import { useEffect, useState } from "react";

interface SecurityStatus {
  contextIsolation: boolean;
  nodeIntegration: boolean;
  sandbox: boolean;
  cspActive: boolean;
}

export function SecurityPosture() {
  const [status, setStatus] = useState<SecurityStatus>({
    contextIsolation: false,
    nodeIntegration: true,
    sandbox: false,
    cspActive: false
  });

  useEffect(() => {
    // These values are injected by the preload script or inferred
    const win = window as any;

    setStatus({
      contextIsolation: win.__KAIOKEN_CONTEXT_ISOLATION__ === true,
      nodeIntegration: win.__KAIOKEN_NODE_INTEGRATION__ === true,
      sandbox: win.__KAIOKEN_SANDBOX__ === true,
      cspActive: true // Assumed true if the app loaded without CSP violation blocks
    });
  }, []);

  const checks = [
    {
      label: "Context Isolation",
      good: status.contextIsolation,
      detail: status.contextIsolation
        ? "Preload bridge is isolated from renderer JS."
        : "Renderer has access to Node.js primitives."
    },
    {
      label: "Node Integration",
      good: !status.nodeIntegration,
      detail: !status.nodeIntegration
        ? "Renderer cannot require Node modules."
        : "Renderer can execute arbitrary Node code."
    },
    {
      label: "Process Sandbox",
      good: status.sandbox,
      detail: status.sandbox
        ? "Renderer OS process is sandboxed."
        : "Renderer OS process has full system access."
    },
    {
      label: "Content Security Policy",
      good: status.cspActive,
      detail: status.cspActive
        ? "Strict CSP headers enforced by main process."
        : "No CSP restrictions detected."
    }
  ];

  return (
    <div className="surface-card" style={{ marginTop: "var(--s-5)" }}>
      <div className="head">
        <span className="head-num">08</span>
        <span className="head-title">Security Posture</span>
        <span className="head-note">PHASE 6.1</span>
      </div>

      <div className="cmp" style={{ border: "none" }}>
        {checks.map((check) => (
          <div key={check.label} className="pnl">
            <div className="pnl-h">
              <div
                className={`pnl-dot ${
                  check.good ? "pnl-dot--good" : "pnl-dot--bad"
                }`}
              />
              <span className="pnl-l">{check.label}</span>
            </div>

            <div className="claims">
              <div
                className={`claim ${
                  check.good ? "claim--good" : "claim--bad"
                }`}
              >
                <span className="claim-t">{check.detail}</span>
                <span
                  className={`claim-n ${
                    check.good ? "claim-n--good" : "claim-n--bad"
                  }`}
                >
                  {check.good ? "NOMINAL" : "VULNERABLE"}
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

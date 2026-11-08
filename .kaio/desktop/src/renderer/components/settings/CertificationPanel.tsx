// src/renderer/components/settings/CertificationPanel.tsx
import { useStore } from "@nanostores/react";
import {
  copyCertificationReport,
  diagnosticsStore,
  runCertification
} from "@renderer/stores/diagnostics";

export function CertificationPanel() {
  const diagnostics = useStore(diagnosticsStore);

  const running = diagnostics.running;

  const startup = diagnostics.main?.startup;
  const memory = diagnostics.main?.memory;

  return (
    <div className="surface-card" style={{ marginTop: "var(--s-5)" }}>
      <div className="head">
        <span className="head-num">10</span>
        <span className="head-title">Release Certification</span>
        <span className="head-note">PHASE 6.4</span>
      </div>

      <div className="cert-body">
        <div className="cert-metrics">
          <div className="cert-cell">
            <span className="label">Startup</span>

            <span className="cert-value">
              {startup?.rendererReadyMs !== undefined
                ? `${startup.rendererReadyMs}ms`
                : "—"}
            </span>

            <span className="cert-sub">
              budget {startup?.budgetMs ?? "—"}ms
            </span>
          </div>

          <div className="cert-cell">
            <span className="label">Main Heap</span>

            <span className="cert-value">
              {memory ? `${memory.heapUsedMB.toFixed(1)} MB` : "—"}
            </span>

            <span className="cert-sub">
              RSS {memory ? `${memory.rssMB.toFixed(1)} MB` : "—"}
            </span>
          </div>

          <div className="cert-cell">
            <span className="label">Renderer Heap</span>

            <span className="cert-value">
              {diagnostics.rendererMemoryMB === null
                ? "n/a"
                : `${diagnostics.rendererMemoryMB.toFixed(1)} MB`}
            </span>

            <span className="cert-sub">JS heap</span>
          </div>

          <div className="cert-cell">
            <span className="label">Overall</span>

            <span
              className={`cert-status cert-status-${diagnostics.overall}`}
            >
              {diagnostics.overall}
            </span>

            <span className="cert-sub">
              {diagnostics.lastReport
                ? new Date(
                    diagnostics.lastReport.runAt
                  ).toLocaleTimeString()
                : "not run"}
            </span>
          </div>
        </div>

        {diagnostics.main?.warnings.map((warning, index) => (
          <div key={index} className="cert-warning">
            {warning}
          </div>
        ))}

        <div className="cert-actions">
          <button
            type="button"
            className="btn btn--primary btn--sm"
            disabled={running}
            onClick={() => void runCertification()}
          >
            {running ? "Certifying..." : "Run certification"}
          </button>

          <button
            type="button"
            className="btn btn--sm"
            disabled={running || !diagnostics.lastReport}
            onClick={() => void copyCertificationReport()}
          >
            Copy report JSON
          </button>
        </div>

        <div className="tw cert-table-wrap">
          <table className="tbl">
            <thead>
              <tr>
                <th className="cert-table-status">Status</th>
                <th>Check</th>
                <th>Detail</th>
              </tr>
            </thead>

            <tbody>
              {diagnostics.checks.map((check) => (
                <tr key={check.id}>
                  <td>
                    <span
                      className={`cert-status cert-status-${check.status}`}
                    >
                      {check.status}
                    </span>
                  </td>

                  <td className="c-nm">{check.label}</td>

                  <td className="c-mut">{check.detail}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

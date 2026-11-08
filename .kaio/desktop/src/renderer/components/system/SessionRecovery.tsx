// src/renderer/components/system/SessionRecovery.tsx
import { useEffect, useState } from "react";
import { useStore } from "@nanostores/react";
import { workspaceStore, restoreWorkspace } from "@renderer/stores/explorer";
import { sessionStore, restoreLastActiveSession } from "@renderer/stores/session";
import { AlertTriangle } from "lucide-react";

interface SessionRecoveryProps {
  onDismiss?: () => void;
}

export function SessionRecovery({ onDismiss }: SessionRecoveryProps = {}) {
  const workspace = useStore(workspaceStore);
  const session = useStore(sessionStore);

  const [recovering, setRecovering] = useState(false);

  useEffect(() => {
    // Attempt initial probe of workspace to see if there is a workspace to restore
    void window.api.workspace.current().then((meta) => {
      if (meta && !workspace.workspace) {
        workspaceStore.set({
          ...workspaceStore.get(),
          workspace: meta
        });
      }
    });
  }, []);

  const handleRecover = async () => {
    setRecovering(true);

    try {
      await restoreWorkspace();
      await restoreLastActiveSession();

      if (onDismiss) {
        onDismiss();
      }
    } catch (err) {
      console.error("Recovery failed:", err);
    } finally {
      setRecovering(false);
    }
  };

  return (
    <div className="section" style={{ height: "100vh", display: "flex", alignItems: "center" }}>
      <div className="wrap" style={{ maxWidth: "600px", margin: "0 auto" }}>
        <div className="head flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="head-num label flex items-center justify-center">
              <AlertTriangle size={15} />
            </span>
            <span className="head-title h3">Session Recovery Required</span>
          </div>
        </div>

        <p className="lead" style={{ marginBottom: "var(--s-5)" }}>
          Kaioken Desktop Studio detected an unexpected termination. Your previous
          workspace and agent session can be restored from the local persistence layer.
        </p>

        <div className="term" style={{ marginBottom: "var(--s-5)" }}>
          <div className="term-bar">
            <span>Recovery Status</span>
          </div>
          <div className="term-body">
            <div className="tl">
              <span className="t-k">Workspace Root:</span>{" "}
              <span className="t-v">{workspace.workspace?.root ?? "None detected"}</span>
            </div>
            <div className="tl">
              <span className="t-k">Active Session:</span>{" "}
              <span className="t-v">
                {session.activeSession?.sessionId ?? "No active session"}
              </span>
            </div>
          </div>
        </div>

        <div style={{ display: "flex", gap: "var(--s-3)" }}>
          <button
            className="btn btn--primary"
            onClick={() => void handleRecover()}
            disabled={recovering}
          >
            {recovering ? "Restoring..." : "Restore Session"}
          </button>

          <button
            className="btn"
            onClick={() => {
              // Clear recovery state and start fresh
              if (onDismiss) {
                onDismiss();
              } else {
                window.location.reload();
              }
            }}
          >
            Start Fresh
          </button>
        </div>
      </div>
    </div>
  );
}

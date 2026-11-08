import { useEffect, useState } from "react";
import { AppShell } from "@renderer/components/shell/AppShell";
import { SessionRecovery } from "@renderer/components/system/SessionRecovery";

export default function App() {
  const [needsRecovery, setNeedsRecovery] = useState(false);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    const checkCrashState = async () => {
      try {
        // Check if there are recent crash reports
        const reports = await window.api.crash.getReports();

        if (reports && reports.length > 0) {
          const lastCrash = reports[0];
          const timeSinceCrash = Date.now() - lastCrash.timestamp;

          // If the last crash was less than 5 minutes ago, trigger recovery
          if (timeSinceCrash < 5 * 60 * 1000) {
            setNeedsRecovery(true);
          }
        }
      } catch (err) {
        console.error("Failed to check crash state:", err);
      } finally {
        setChecking(false);
      }
    };

    void checkCrashState();
  }, []);

  if (checking) {
    return <div style={{ height: "100vh", background: "var(--bg)" }} />;
  }

  if (needsRecovery) {
    return <SessionRecovery onDismiss={() => setNeedsRecovery(false)} />;
  }

  return <AppShell />;
}

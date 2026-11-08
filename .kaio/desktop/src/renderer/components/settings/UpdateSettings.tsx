// src/renderer/components/settings/UpdateSettings.tsx
import { useStore } from "@nanostores/react";
import {
  checkForUpdates,
  downloadUpdate,
  installUpdate,
  setUpdateChannel,
  updaterStore
} from "@renderer/stores/updater";
import { openReleaseNotes } from "@renderer/stores/release-notes";
import type { UpdateChannel } from "@shared/types/updater";

export function UpdateSettings() {
  const updater = useStore(updaterStore);

  const busy =
    updater.status === "checking" ||
    updater.status === "downloading" ||
    updater.status === "installing";

  return (
    <div className="surface-card" style={{ marginTop: "var(--s-5)" }}>
      <div className="head">
        <span className="head-num">09</span>
        <span className="head-title">Updates & Release Notes</span>
        <span className="head-note">PHASE 6.3</span>
      </div>

      <div className="update-body">
        <div className="update-meta">
          <div className="update-cell">
            <span className="label">Current</span>
            <span className="update-value">
              v{updater.currentVersion || "0.0.0"}
            </span>
          </div>

          <div className="update-cell">
            <span className="label">Channel</span>

            <select
              className="select"
              value={updater.channel}
              onChange={(event) =>
                void setUpdateChannel(
                  event.target.value as UpdateChannel
                )
              }
            >
              <option value="stable">stable</option>
              <option value="beta">beta</option>
              <option value="dev">dev</option>
            </select>
          </div>

          <div className="update-cell">
            <span className="label">Status</span>
            <span className="update-value">{updater.status}</span>
          </div>

          <div className="update-cell">
            <span className="label">Progress</span>
            <span className="update-value">
              {updater.status === "downloading"
                ? `${updater.downloadProgress}%`
                : "—"}
            </span>
          </div>
        </div>

        <div className="update-actions">
          <button
            type="button"
            className="btn btn--sm"
            disabled={busy}
            onClick={() => void checkForUpdates()}
          >
            Check for updates
          </button>

          <button
            type="button"
            className="btn btn--sm"
            disabled={updater.status !== "available" || busy}
            onClick={() => void downloadUpdate()}
          >
            Download
          </button>

          <button
            type="button"
            className="btn btn--primary btn--sm"
            disabled={updater.status !== "downloaded" || busy}
            onClick={() => void installUpdate()}
          >
            Install
          </button>

          <button
            type="button"
            className="btn btn--sm"
            onClick={() => openReleaseNotes()}
          >
            Release notes
          </button>
        </div>

        {updater.error && (
          <div className="update-error">{updater.error}</div>
        )}

        {updater.status === "downloading" && (
          <div className="update-progress">
            <div
              className="update-progress-fill"
              style={{
                width: `${updater.downloadProgress}%`
              }}
            />
          </div>
        )}
      </div>
    </div>
  );
}

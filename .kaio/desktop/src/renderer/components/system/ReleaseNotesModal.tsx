// src/renderer/components/system/ReleaseNotesModal.tsx
import { useStore } from "@nanostores/react";
import {
  closeReleaseNotes,
  releaseNotesStore
} from "@renderer/stores/release-notes";
import {
  checkForUpdates,
  downloadUpdate,
  installUpdate,
  setUpdateChannel,
  updaterStore
} from "@renderer/stores/updater";
import { renderKaiokenMarkdown } from "@renderer/lib/markdown";
import type { UpdateChannel } from "@shared/types/updater";

export function ReleaseNotesModal() {
  const release = useStore(releaseNotesStore);
  const updater = useStore(updaterStore);

  if (!release.open) {
    return null;
  }

  const busy =
    updater.status === "checking" ||
    updater.status === "downloading" ||
    updater.status === "installing";

  const notes =
    updater.releaseNotes ||
    "# No release notes available\n\nRun an update check to fetch the latest channel notes.";

  return (
    <div
      className="release-backdrop"
      onClick={() => closeReleaseNotes()}
    >
      <div
        className="release-panel"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="release-head">
          <div className="release-head-left">
            <span className="label label--accent">Release Notes</span>

            <span className="mono release-version">
              v{updater.currentVersion || "0.0.0"}
            </span>
          </div>

          <button
            type="button"
            className="btn btn--sm"
            onClick={() => closeReleaseNotes()}
          >
            Close
          </button>
        </div>

        <div className="release-toolbar">
          <span className="release-status">
            status {updater.status}
          </span>

          <select
            className="select release-channel"
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

          <div className="release-actions">
            <button
              type="button"
              className="btn btn--sm"
              disabled={busy}
              onClick={() => void checkForUpdates()}
            >
              Check
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
          </div>

          {updater.status === "downloading" && (
            <div className="release-progress">
              <div
                className="release-progress-fill"
                style={{
                  width: `${updater.downloadProgress}%`
                }}
              />
            </div>
          )}
        </div>

        {updater.error && (
          <div className="release-error">{updater.error}</div>
        )}

        <div className="release-content hud-scrollbar">
          <article className="release-article md">
            {renderKaiokenMarkdown(notes)}
          </article>
        </div>
      </div>
    </div>
  );
}

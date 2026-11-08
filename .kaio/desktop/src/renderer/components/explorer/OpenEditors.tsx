// src/renderer/components/explorer/OpenEditors.tsx
import { useStore } from "@nanostores/react";
import {
  explorerStore,
  closeFile,
  setActiveFile
} from "@renderer/stores/explorer";

import { FileCode, X } from "lucide-react";

function fileBaseName(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

export function OpenEditors() {
  const state = useStore(explorerStore);

  if (state.openFiles.length === 0) {
    return null;
  }

  return (
    <div
      style={{
        borderBottom: "var(--hair) solid var(--rule)"
      }}
    >
      <div
        className="label label--accent"
        style={{ padding: "var(--s-2) var(--s-3)" }}
      >
        Open Editors
      </div>

      {state.openFiles.map((p) => {
        const isActive = p === state.activeFilePath;

        return (
          <div
            key={p}
            className={`open-editor-row ${isActive ? "is-active" : ""}`}
            onClick={() => setActiveFile(p)}
          >
            <div className="flex items-center gap-1.5 overflow-hidden flex-1">
              <FileCode size={13} className="shrink-0 text-[var(--kai-blue)] opacity-85" />
              <span className="open-editor-name truncate">{fileBaseName(p)}</span>
            </div>

            <button
              type="button"
              className="open-editor-close flex items-center justify-center"
              aria-label="Close file"
              onClick={(event) => {
                event.stopPropagation();
                closeFile(p);
              }}
            >
              <X size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
}

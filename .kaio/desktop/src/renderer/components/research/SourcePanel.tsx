// src/renderer/components/research/SourcePanel.tsx
import { useEffect, useState } from "react";
import { useStore } from "@nanostores/react";
import {
  addCitationForSelected,
  findSourceById,
  formatSourceCitation,
  getSelectedSource,
  removeCitation,
  researchStore
} from "@renderer/stores/research";

export function SourcePanel() {
  const state = useStore(researchStore);
  const source = getSelectedSource(state);

  const [copied, setCopied] = useState(false);

  useEffect(() => {
    setCopied(false);
  }, [state.selectedSourceId]);

  if (!source) {
    return (
      <aside className="research-panel">
        <div className="research-empty">
          Select a source to inspect it and build citations.
        </div>
      </aside>
    );
  }

  const citation = formatSourceCitation(source);

  const isCited = state.citations.some(
    (entry) => entry.sourceId === source.id
  );

  const copyCitation = async () => {
    try {
      await navigator.clipboard.writeText(citation);
      setCopied(true);

      window.setTimeout(() => {
        setCopied(false);
      }, 1200);
    } catch {
      // Clipboard unavailable.
    }
  };

  return (
    <aside className="research-panel hud-scrollbar">
      <div className="research-panel-section">
        <div className="research-panel-top">
          <span className="label label--accent">Source</span>

          <span className={`research-kind research-kind-${source.kind}`}>
            {source.kind}
          </span>
        </div>

        <h3 className="research-panel-title">{source.title}</h3>

        <p className="research-panel-snippet">{source.snippet}</p>

        <div className="research-meta">
          <span>locator</span>
          <span>{source.url ?? source.path ?? "—"}</span>

          <span>published</span>
          <span>{source.publishedAt ?? "—"}</span>

          <span>trust</span>
          <span>{source.trustScore}%</span>
        </div>

        <div className="research-actions">
          <button
            type="button"
            className="btn btn--primary btn--sm"
            disabled={isCited}
            onClick={() => addCitationForSelected()}
          >
            {isCited ? "Cited" : "Add citation"}
          </button>

          <button
            type="button"
            className="btn btn--sm"
            onClick={() => void copyCitation()}
          >
            {copied ? "Copied" : "Copy citation"}
          </button>
        </div>

        <div className="label">Citation preview</div>

        <pre className="research-citation-preview">{citation}</pre>
      </div>

      <div className="research-panel-section">
        <div className="research-panel-top">
          <span className="label label--accent">Dossier</span>

          <span className="label">
            {state.citations.length} citations
          </span>
        </div>

        {state.citations.length === 0 && (
          <div
            className="mono"
            style={{
              fontSize: "var(--fs-xs)",
              color: "var(--fg-mute)"
            }}
          >
            No citations yet. Add sources to build a research dossier.
          </div>
        )}

        <div className="research-citations">
          {state.citations.map((entry) => {
            const citedSource = findSourceById(state, entry.sourceId);

            if (!citedSource) {
              return null;
            }

            return (
              <div key={entry.id} className="research-citation">
                <span className="research-citation-title">
                  {citedSource.title}
                </span>

                <button
                  type="button"
                  className="btn btn--sm btn--danger"
                  onClick={() => removeCitation(entry.id)}
                >
                  Remove
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </aside>
  );
}

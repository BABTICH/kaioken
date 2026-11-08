// src/renderer/components/research/ResearchSurface.tsx
import { useStore } from "@nanostores/react";
import {
  getActiveQuery,
  researchStore,
  runResearchQuery,
  runResearchQueryWithText,
  setSearchInput
} from "@renderer/stores/research";
import { ResearchResults } from "./ResearchResults";
import { SourcePanel } from "./SourcePanel";
import { Search, Sparkles } from "lucide-react";

export function ResearchSurface() {
  const state = useStore(researchStore);
  const activeQuery = getActiveQuery(state);

  const searching = activeQuery?.status === "searching";

  return (
    <div className="research-surface">
      <div className="research-toolbar">
        <span className="label label--accent">Research</span>

        <input
          className="input research-search"
          placeholder="Search corpus, wiki, docs, and web..."
          value={state.searchInput}
          onChange={(event) => setSearchInput(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              void runResearchQuery();
            }
          }}
        />

        <button
          type="button"
          className="btn btn--primary btn--sm inline-flex items-center gap-1"
          disabled={searching || state.searchInput.trim().length === 0}
          onClick={() => {
            void runResearchQuery();
          }}
        >
          <Search size={13} />
          <span>Search</span>
        </button>

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          disabled={searching}
          onClick={() => {
            void runResearchQueryWithText(
              "Kaioken approval gateway"
            );
          }}
        >
          <Sparkles size={13} />
          <span>Example</span>
        </button>

        <div style={{ flex: 1 }} />

        <span className="label">
          citations {state.citations.length}
        </span>

        {activeQuery && (
          <span className="label">{activeQuery.status}</span>
        )}
      </div>

      <div className="research-body">
        <main className="research-main hud-scrollbar">
          <ResearchResults />
        </main>

        <SourcePanel />
      </div>
    </div>
  );
}

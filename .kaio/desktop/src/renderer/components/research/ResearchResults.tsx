// src/renderer/components/research/ResearchResults.tsx
import { useStore } from "@nanostores/react";
import {
  getActiveQuery,
  researchStore,
  selectResearchSource
} from "@renderer/stores/research";
import type { ResearchSource } from "@shared/types/research";

interface ResultCardProps {
  source: ResearchSource;
  isSelected: boolean;
}

function ResultCard({ source, isSelected }: ResultCardProps) {
  return (
    <article
      className={[
        "research-result",
        isSelected ? "is-active" : ""
      ].join(" ")}
      onClick={() => selectResearchSource(source.id)}
    >
      <div className="research-result-head">
        <span className={`research-kind research-kind-${source.kind}`}>
          {source.kind}
        </span>

        <h3 className="research-result-title">{source.title}</h3>

        <span className="mono research-trust">
          {source.trustScore}%
        </span>
      </div>

      <div className="research-trust-bar">
        <div
          className="research-trust-fill"
          style={{ width: `${source.trustScore}%` }}
        />
      </div>

      <p className="research-result-snippet">{source.snippet}</p>

      <div className="research-result-foot">
        <div className="research-tags">
          {source.tags.slice(0, 3).map((tag) => (
            <span key={tag} className="research-tag">
              {tag}
            </span>
          ))}
        </div>

        <span className="mono research-source-path">
          {source.path ?? source.url ?? source.kind}
        </span>
      </div>
    </article>
  );
}

export function ResearchResults() {
  const state = useStore(researchStore);
  const query = getActiveQuery(state);

  if (!query) {
    return (
      <div className="research-empty">
        <p className="lead">No active research query.</p>

        <p
          className="mono"
          style={{
            fontSize: "var(--fs-xs)",
            color: "var(--fg-mute)"
          }}
        >
          Type a query above and press Enter.
        </p>
      </div>
    );
  }

  if (query.status === "searching") {
    return (
      <div className="research-results-list">
        {Array.from({ length: 5 }).map((_, index) => (
          <div
            key={index}
            className="research-result research-loading animate-shimmer"
          />
        ))}
      </div>
    );
  }

  if (query.results.length === 0) {
    return (
      <div className="research-empty">
        <p className="lead">No results found.</p>
      </div>
    );
  }

  return (
    <div className="research-results-list">
      {query.results.map((source) => (
        <ResultCard
          key={source.id}
          source={source}
          isSelected={source.id === state.selectedSourceId}
        />
      ))}
    </div>
  );
}

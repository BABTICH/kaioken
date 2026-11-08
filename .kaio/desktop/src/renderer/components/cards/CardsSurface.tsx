// src/renderer/components/cards/CardsSurface.tsx
import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import {
  cardsStore,
  generateMockCards,
  loadMockCards,
  openCardEditor,
  setSearchQuery,
  setStatusFilter,
  type CardStatusFilter
} from "@renderer/stores/cards";
import { CardGrid } from "./CardGrid";
import { CardPanel } from "./CardPanel";
import { RotateCw, Sparkles, Plus } from "lucide-react";

export function CardsSurface() {
  const state = useStore(cardsStore);

  const cardCount = state.cards.length;

  useEffect(() => {
    if (cardCount === 0) {
      loadMockCards();
    }
  }, [cardCount]);

  return (
    <div className="cards-surface">
      <div className="cards-toolbar">
        <span className="label label--accent">Cards</span>

        <span className="label">total {state.cards.length}</span>

        <input
          className="input cards-search"
          placeholder="Search cards..."
          value={state.searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
        />

        <select
          className="select cards-filter"
          value={state.statusFilter}
          onChange={(event) =>
            setStatusFilter(event.target.value as CardStatusFilter)
          }
        >
          <option value="all">all</option>
          <option value="draft">draft</option>
          <option value="verified">verified</option>
          <option value="stale">stale</option>
          <option value="archived">archived</option>
        </select>

        <div style={{ flex: 1 }} />

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => loadMockCards()}
        >
          <RotateCw size={12} />
          <span>Reload</span>
        </button>

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => generateMockCards()}
        >
          <Sparkles size={12} />
          <span>Generate</span>
        </button>

        <button
          type="button"
          className="btn btn--primary btn--sm inline-flex items-center gap-1"
          onClick={() => openCardEditor()}
        >
          <Plus size={13} />
          <span>New card</span>
        </button>
      </div>

      <div className="cards-body">
        <main className="cards-main hud-scrollbar">
          <CardGrid />
        </main>

        <CardPanel />
      </div>
    </div>
  );
}

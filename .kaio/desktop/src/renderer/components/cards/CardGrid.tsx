// src/renderer/components/cards/CardGrid.tsx
import { useStore } from "@nanostores/react";
import {
  cardsStore,
  filterCards,
  openCardEditor,
  selectCard
} from "@renderer/stores/cards";
import type { CardStatus } from "@shared/types/cards";

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString();
}

function CardStatusBadge({ status }: { status: CardStatus }) {
  return (
    <span className={`card-status card-status-${status}`}>
      {status}
    </span>
  );
}

export function CardGrid() {
  const state = useStore(cardsStore);
  const cards = filterCards(state);

  if (cards.length === 0) {
    return (
      <div className="cards-empty">
        <p className="lead">No cards match the current filter.</p>

        <p
          className="mono"
          style={{
            fontSize: "var(--fs-xs)",
            color: "var(--fg-mute)"
          }}
        >
          Clear the search, change status filter, reload mock cards, or
          generate new cards.
        </p>
      </div>
    );
  }

  return (
    <div className="grid-rule cards-grid">
      {cards.map((card) => {
        const isActive = card.id === state.selectedCardId;

        return (
          <article
            key={card.id}
            className={`card ${isActive ? "is-active" : ""}`}
            onClick={() => selectCard(card.id)}
            onDoubleClick={() => openCardEditor(card.id)}
          >
            <div className="card-head">
              <CardStatusBadge status={card.status} />

              {typeof card.confidence === "number" && (
                <span className="mono card-confidence">
                  {card.confidence}%
                </span>
              )}
            </div>

            <h3 className="card-title">{card.title}</h3>

            <p className="card-summary">{card.summary}</p>

            <div className="card-tags">
              {card.tags.map((tag) => (
                <span key={tag} className="card-tag">
                  {tag}
                </span>
              ))}
            </div>

            <div className="card-foot mono">
              <span>{card.source ?? "local"}</span>

              <span>{formatDate(card.updatedAt)}</span>
            </div>
          </article>
        );
      })}
    </div>
  );
}

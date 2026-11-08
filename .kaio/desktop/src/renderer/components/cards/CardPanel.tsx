// src/renderer/components/cards/CardPanel.tsx
import { useStore } from "@nanostores/react";
import {
  cardsStore,
  closeCardEditor,
  deleteCard,
  openCardEditor,
  saveCardDraft,
  updateCardDraft
} from "@renderer/stores/cards";
import type { CardStatus } from "@shared/types/cards";

function CardStatusBadge({ status }: { status: CardStatus }) {
  return (
    <span className={`card-status card-status-${status}`}>
      {status}
    </span>
  );
}

function CardDetail() {
  const state = useStore(cardsStore);

  const selected = state.cards.find(
    (card) => card.id === state.selectedCardId
  );

  if (!selected) {
    return (
      <aside className="card-panel">
        <div className="card-panel-empty">
          Select a card to inspect it.
          <br />
          Double-click a card to edit it.
        </div>
      </aside>
    );
  }

  return (
    <aside className="card-panel hud-scrollbar">
      <div className="card-panel-section">
        <div className="card-panel-top">
          <span className="label label--accent">Card</span>

          <CardStatusBadge status={selected.status} />
        </div>

        <h3 className="card-panel-title">{selected.title}</h3>

        {selected.summary && (
          <p className="card-panel-summary">{selected.summary}</p>
        )}
      </div>

      <div className="card-panel-section">
        <div className="label">Body</div>

        <div className="card-detail-body">
          {selected.body || "No body content."}
        </div>
      </div>

      <div className="card-panel-section">
        <div className="label">Tags</div>

        <div className="card-tags">
          {selected.tags.length === 0 && (
            <span className="card-tag">none</span>
          )}

          {selected.tags.map((tag) => (
            <span key={tag} className="card-tag">
              {tag}
            </span>
          ))}
        </div>
      </div>

      <div className="card-panel-section">
        <div className="label">Metadata</div>

        <div className="card-detail-meta">
          <span>id</span>
          <span>{selected.id}</span>

          <span>source</span>
          <span>{selected.source ?? "local"}</span>

          {typeof selected.confidence === "number" && (
            <>
              <span>confidence</span>
              <span>{selected.confidence}%</span>
            </>
          )}

          <span>updated</span>
          <span>{new Date(selected.updatedAt).toLocaleString()}</span>
        </div>
      </div>

      <div className="card-panel-actions">
        <button
          type="button"
          className="btn btn--primary btn--sm"
          onClick={() => openCardEditor(selected.id)}
        >
          Edit
        </button>

        <button
          type="button"
          className="btn btn--danger btn--sm"
          onClick={() => deleteCard(selected.id)}
        >
          Delete
        </button>
      </div>
    </aside>
  );
}

function CardEditorForm() {
  const state = useStore(cardsStore);
  const draft = state.draft;

  if (!draft) {
    return null;
  }

  const isNew = !state.cards.some((card) => card.id === draft.id);

  return (
    <aside className="card-panel hud-scrollbar">
      <div className="card-panel-section">
        <div className="card-panel-top">
          <span className="label label--accent">
            {isNew ? "New card" : "Edit card"}
          </span>

          <button
            type="button"
            className="btn btn--sm"
            onClick={() => closeCardEditor()}
          >
            Close
          </button>
        </div>

        <div className="field">
          <label className="label">Title</label>

          <input
            className="input"
            value={draft.title}
            onChange={(event) =>
              updateCardDraft({
                title: event.target.value
              })
            }
          />
        </div>

        <div className="field">
          <label className="label">Status</label>

          <select
            className="select"
            value={draft.status}
            onChange={(event) =>
              updateCardDraft({
                status: event.target.value as CardStatus
              })
            }
          >
            <option value="draft">draft</option>
            <option value="verified">verified</option>
            <option value="stale">stale</option>
            <option value="archived">archived</option>
          </select>
        </div>

        <div className="field">
          <label className="label">Source</label>

          <input
            className="input"
            value={draft.source ?? ""}
            placeholder="phase-5.4"
            onChange={(event) =>
              updateCardDraft({
                source: event.target.value
              })
            }
          />
        </div>

        <div className="field">
          <label className="label">Tags</label>

          <input
            className="input"
            value={draft.tags.join(", ")}
            placeholder="security, agent, workspace"
            onChange={(event) => {
              const tags = event.target.value
                .split(",")
                .map((tag) => tag.trim())
                .filter((tag) => tag.length > 0);

              updateCardDraft({ tags });
            }}
          />
        </div>

        <div className="field">
          <label className="label">Summary</label>

          <textarea
            className="textarea"
            rows={3}
            value={draft.summary}
            onChange={(event) =>
              updateCardDraft({
                summary: event.target.value
              })
            }
          />
        </div>

        <div className="field">
          <label className="label">Body</label>

          <textarea
            className="textarea"
            rows={10}
            value={draft.body}
            onChange={(event) =>
              updateCardDraft({
                body: event.target.value
              })
            }
          />
        </div>
      </div>

      <div className="card-panel-actions">
        <button
          type="button"
          className="btn btn--primary btn--sm"
          onClick={() => saveCardDraft()}
        >
          Save
        </button>

        <button
          type="button"
          className="btn btn--sm"
          onClick={() => closeCardEditor()}
        >
          Cancel
        </button>

        {!isNew && (
          <button
            type="button"
            className="btn btn--danger btn--sm"
            onClick={() => deleteCard(draft.id)}
          >
            Delete
          </button>
        )}
      </div>
    </aside>
  );
}

export function CardPanel() {
  const state = useStore(cardsStore);

  if (state.editorOpen) {
    return <CardEditorForm />;
  }

  return <CardDetail />;
}

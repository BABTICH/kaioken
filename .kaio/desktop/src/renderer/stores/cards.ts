// src/renderer/stores/cards.ts
import { atom } from "nanostores";
import type {
  CardStatus,
  KnowledgeCard
} from "@shared/types/cards";

export type CardStatusFilter = CardStatus | "all";

export interface CardsState {
  cards: KnowledgeCard[];
  selectedCardId: string | null;
  searchQuery: string;
  statusFilter: CardStatusFilter;
  editorOpen: boolean;
  draft: KnowledgeCard | null;
}

const initialState: CardsState = {
  cards: [],
  selectedCardId: null,
  searchQuery: "",
  statusFilter: "all",
  editorOpen: false,
  draft: null
};

export const cardsStore = atom<CardsState>(initialState);

function set(patch: Partial<CardsState>): void {
  cardsStore.set({
    ...cardsStore.get(),
    ...patch
  });
}

function now(): number {
  return Date.now();
}

function createMockCards(): KnowledgeCard[] {
  const ts = now();

  return [
    {
      id: "card-workspace-guard",
      title: "Workspace Guard",
      summary:
        "All filesystem access is resolved against a trusted workspace root.",
      body: `The workspace guard resolves every requested path against the trusted root.

It blocks:
- Parent traversal with ../
- Absolute paths outside the workspace
- Symlink escapes using realpath containment

This guard is mandatory for workspace reads, file tree expansion, terminal cwd resolution, and future agent file tools.`,
      tags: ["security", "filesystem", "workspace"],
      status: "verified",
      source: "phase-3.1",
      confidence: 98,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-utility-process-isolation",
      title: "Utility Process Isolation",
      summary:
        "The Kaiopi engine runs inside an Electron utility process.",
      body: `Kaiopi is isolated from the Electron main process.

Responsibilities:
- Main: security gateway, window lifecycle, PTY, approvals.
- Utility: agent runtime, session orchestration, indexing.

This gives crash containment and cleaner restart semantics.`,
      tags: ["architecture", "electron", "engine"],
      status: "verified",
      source: "phase-2.1",
      confidence: 96,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-destructive-approval",
      title: "Destructive Approval Gateway",
      summary:
        "State-mutating agent tools require explicit desktop approval.",
      body: `Destructive tools pause execution and emit an approval request.

The main process:
- Stores the pending request.
- Starts a timeout.
- Records an audit entry.
- Notifies the engine only after resolution.

Safe default: deny.`,
      tags: ["security", "agent", "approval"],
      status: "verified",
      source: "phase-4.3",
      confidence: 95,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-stream-backpressure",
      title: "Streaming Backpressure",
      summary:
        "High-frequency agent events use batching, ACKs, and watermarks.",
      body: `The streaming pipeline protects UI responsiveness.

Mechanisms:
- 16ms batch envelopes.
- Sequence numbers and cursors.
- Renderer ACKs.
- Main/engine high and low watermarks.

This keeps token floods from saturating IPC.`,
      tags: ["streaming", "performance", "ipc"],
      status: "verified",
      source: "phase-2.2",
      confidence: 92,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-deterministic-truth",
      title: "Deterministic Truth Layer",
      summary:
        "Kaioken grounding prefers deterministic repository truth over model memory.",
      body: `The truth layer uses repository facts as the primary source of truth.

Examples:
- AST symbol lookup.
- File hashes.
- Test verification.
- Impact analysis.

Model output must be grounded in these artifacts where possible.`,
      tags: ["kaioken", "grounding", "agent"],
      status: "draft",
      source: "kaiopi-spec",
      confidence: 76,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-test-verification-gate",
      title: "Test Verification Gate",
      summary:
        "Agents cannot mark mutating work complete unless verification passes.",
      body: `The verification gate runs the repository test suite after state-mutating changes.

Rules:
- Exit code 0 is required.
- Failed tests block completion.
- Verification output becomes an artifact.

This prevents hallucinated success.`,
      tags: ["verification", "testing", "agent"],
      status: "stale",
      source: "kaiopi-spec",
      confidence: 68,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-git-worktree-isolation",
      title: "Git Worktree Isolation",
      summary:
        "Subagents can execute inside isolated git worktrees.",
      body: `Worktree isolation prevents dirty working trees.

Benefits:
- Parallel agent tasks do not collide.
- Failed experiments can be discarded.
- Main workspace remains clean.

This is a future hardening target for desktop orchestration.`,
      tags: ["git", "isolation", "agent"],
      status: "draft",
      source: "roadmap",
      confidence: 61,
      createdAt: ts,
      updatedAt: ts
    },
    {
      id: "card-cost-multiplier",
      title: "Cost Multiplier",
      summary:
        "Multiplier levels express agent execution intensity and budget impact.",
      body: `Multiplier levels range from x1 to x10.

They affect:
- Model selection.
- Token budget.
- Tool aggressiveness.
- Cost ledger reporting.

The desktop UI should make multiplier changes explicit and auditable.`,
      tags: ["cost", "budget", "agent"],
      status: "archived",
      source: "phase-2.4",
      confidence: 54,
      createdAt: ts,
      updatedAt: ts
    }
  ];
}

function createGeneratedCards(offset: number): KnowledgeCard[] {
  const ts = now();

  const topics = [
    "Symbol Oracle",
    "BM25 Corpus Search",
    "Impact Analysis",
    "ModelPort Budget",
    "Wiki Compiler",
    "Test Verification Gate"
  ];

  return topics.map((topic, index) => {
    const generation = offset + index + 1;

    return {
      id: `generated-card-${ts}-${index}`,
      title: `${topic} ${generation}`,
      summary: `Mock generated knowledge card for ${topic}.`,
      body: `This generated card is a placeholder for future Kaioken knowledge artifacts.

Topic: ${topic}
Generation: ${generation}

Real cards will be produced by the wiki/cards compiler and grounded in repository truth.`,
      tags: ["generated", "kaioken"],
      status: "draft",
      source: "mock-generator",
      confidence: 55 + ((index * 13) % 40),
      createdAt: ts,
      updatedAt: ts
    };
  });
}

export function loadMockCards(): void {
  const cards = createMockCards();

  set({
    cards,
    selectedCardId: cards[0]?.id ?? null,
    searchQuery: "",
    statusFilter: "all",
    editorOpen: false,
    draft: null
  });
}

export function generateMockCards(): void {
  const state = cardsStore.get();

  const generated = createGeneratedCards(state.cards.length);

  set({
    cards: [...generated, ...state.cards]
  });
}

export function selectCard(cardId: string | null): void {
  set({
    selectedCardId: cardId
  });
}

export function setSearchQuery(query: string): void {
  set({
    searchQuery: query
  });
}

export function setStatusFilter(filter: CardStatusFilter): void {
  set({
    statusFilter: filter
  });
}

export function openCardEditor(cardId?: string): void {
  const state = cardsStore.get();

  if (cardId) {
    const card = state.cards.find((item) => item.id === cardId);

    if (card) {
      set({
        selectedCardId: card.id,
        editorOpen: true,
        draft: {
          ...card,
          tags: [...card.tags]
        }
      });

      return;
    }
  }

  const ts = now();

  const draft: KnowledgeCard = {
    id: `card-${ts}`,
    title: "Untitled card",
    summary: "",
    body: "",
    tags: ["draft"],
    status: "draft",
    source: "desktop",
    confidence: 50,
    createdAt: ts,
    updatedAt: ts
  };

  set({
    selectedCardId: draft.id,
    editorOpen: true,
    draft
  });
}

export function closeCardEditor(): void {
  const state = cardsStore.get();

  const draftId = state.draft?.id;

  const selectedCardId =
    state.selectedCardId === draftId &&
    !state.cards.some((card) => card.id === draftId)
      ? null
      : state.selectedCardId;

  set({
    editorOpen: false,
    draft: null,
    selectedCardId
  });
}

export function updateCardDraft(
  patch: Partial<KnowledgeCard>
): void {
  const state = cardsStore.get();

  if (!state.draft) {
    return;
  }

  set({
    draft: {
      ...state.draft,
      ...patch,
      updatedAt: now()
    }
  });
}

export function saveCardDraft(): void {
  const state = cardsStore.get();

  if (!state.draft) {
    return;
  }

  const saved: KnowledgeCard = {
    ...state.draft,
    title: state.draft.title.trim() || "Untitled card",
    updatedAt: now()
  };

  const exists = state.cards.some((card) => card.id === saved.id);

  const cards = exists
    ? state.cards.map((card) =>
        card.id === saved.id ? saved : card
      )
    : [saved, ...state.cards];

  set({
    cards,
    selectedCardId: saved.id,
    editorOpen: false,
    draft: null
  });
}

export function deleteCard(cardId: string): void {
  const state = cardsStore.get();

  const cards = state.cards.filter((card) => card.id !== cardId);

  const selectedCardId =
    state.selectedCardId === cardId ? null : state.selectedCardId;

  const editorOpen =
    state.draft?.id === cardId ? false : state.editorOpen;

  const draft =
    state.draft?.id === cardId ? null : state.draft;

  set({
    cards,
    selectedCardId,
    editorOpen,
    draft
  });
}

export function filterCards(state: CardsState): KnowledgeCard[] {
  const query = state.searchQuery.trim().toLowerCase();

  return state.cards
    .filter((card) => {
      if (
        state.statusFilter !== "all" &&
        card.status !== state.statusFilter
      ) {
        return false;
      }

      if (!query) {
        return true;
      }

      const haystack = [
        card.title,
        card.summary,
        card.body,
        card.source ?? "",
        ...card.tags
      ]
        .join(" ")
        .toLowerCase();

      return haystack.includes(query);
    })
    .sort((a, b) => b.updatedAt - a.updatedAt);
}

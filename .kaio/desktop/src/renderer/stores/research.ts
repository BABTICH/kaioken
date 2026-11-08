// src/renderer/stores/research.ts
import { atom } from "nanostores";
import type {
  Citation,
  ResearchQuery,
  ResearchSource,
  ResearchSourceKind
} from "@shared/types/research";

export interface ResearchState {
  searchInput: string;
  activeQueryId: string | null;
  queries: Record<string, ResearchQuery>;
  selectedSourceId: string | null;
  citations: Citation[];
}

const initialState: ResearchState = {
  searchInput: "",
  activeQueryId: null,
  queries: {},
  selectedSourceId: null,
  citations: []
};

export const researchStore = atom<ResearchState>(initialState);

function set(patch: Partial<ResearchState>): void {
  researchStore.set({
    ...researchStore.get(),
    ...patch
  });
}

function slugify(text: string): string {
  const slug = text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return slug || "research";
}

function generateMockResults(query: string): ResearchSource[] {
  const slug = slugify(query);

  const kinds: ResearchSourceKind[] = [
    "repo",
    "wiki",
    "docs",
    "web",
    "paper"
  ];

  const templates = [
    `${query} — implementation notes`,
    `Grounding ${query} in repository truth`,
    `${query} verification gates`,
    `Kaioken ${query} architecture`,
    `${query} failure modes`,
    `${query} cost and multiplier impact`,
    `Field report: ${query}`,
    `${query} citation dossier`
  ];

  return templates.map((title, index) => {
    const kind = kinds[index % kinds.length];

    const id = `src-${slug}-${index}`;

    const trustScore = 45 + Math.floor(Math.random() * 54);

    const snippet = `Mock research excerpt for "${query}". This ${kind} source discusses ${title.toLowerCase()} with grounded references and verification notes.`;

    const url =
      kind === "web" || kind === "paper"
        ? `https://example.com/${slug}/${index}`
        : undefined;

    const path =
      kind === "repo"
        ? `src/kaioken/${slug}-${index}.ts`
        : kind === "wiki"
          ? `wiki/${slug}-${index}.md`
          : kind === "docs"
            ? `docs/${slug}-${index}.md`
            : undefined;

    return {
      id,
      title,
      kind,
      snippet,
      url,
      path,
      trustScore,
      tags: [kind, "mock", slug],
      publishedAt: new Date(
        Date.now() - index * 86_400_000
      )
        .toISOString()
        .split("T")[0]
    };
  });
}

export function setSearchInput(value: string): void {
  set({
    searchInput: value
  });
}

export function selectResearchSource(
  sourceId: string | null
): void {
  set({
    selectedSourceId: sourceId
  });
}

export function getActiveQuery(
  state: ResearchState
): ResearchQuery | null {
  if (!state.activeQueryId) {
    return null;
  }

  return state.queries[state.activeQueryId] ?? null;
}

export function findSourceById(
  state: ResearchState,
  sourceId: string
): ResearchSource | null {
  for (const query of Object.values(state.queries)) {
    const found = query.results.find(
      (source) => source.id === sourceId
    );

    if (found) {
      return found;
    }
  }

  return null;
}

export function getSelectedSource(
  state: ResearchState
): ResearchSource | null {
  if (!state.selectedSourceId) {
    return null;
  }

  return findSourceById(state, state.selectedSourceId);
}

export function formatSourceCitation(
  source: ResearchSource
): string {
  if (source.url) {
    return `[${source.title}](${source.url})`;
  }

  if (source.path) {
    return `\`${source.path}\` — ${source.title}`;
  }

  return `${source.title} (${source.kind})`;
}

async function executeQuery(text: string): Promise<void> {
  const trimmed = text.trim();

  if (!trimmed) {
    return;
  }

  const queryId = `rq-${Date.now()}`;

  const query: ResearchQuery = {
    id: queryId,
    text: trimmed,
    createdAt: Date.now(),
    status: "searching",
    results: []
  };

  const state = researchStore.get();

  set({
    queries: {
      ...state.queries,
      [queryId]: query
    },
    activeQueryId: queryId,
    selectedSourceId: null
  });

  await new Promise((resolve) => setTimeout(resolve, 650));

  const latest = researchStore.get();
  const existing = latest.queries[queryId];

  if (!existing) {
    return;
  }

  const results = generateMockResults(trimmed);

  set({
    queries: {
      ...latest.queries,
      [queryId]: {
        ...existing,
        status: "complete",
        results
      }
    }
  });
}

export async function runResearchQuery(): Promise<void> {
  const text = researchStore.get().searchInput;

  await executeQuery(text);
}

export async function runResearchQueryWithText(
  text: string
): Promise<void> {
  const trimmed = text.trim();

  if (!trimmed) {
    return;
  }

  set({
    searchInput: trimmed
  });

  await executeQuery(trimmed);
}

export function addCitationForSelected(): void {
  const state = researchStore.get();

  if (!state.selectedSourceId) {
    return;
  }

  const alreadyCited = state.citations.some(
    (citation) => citation.sourceId === state.selectedSourceId
  );

  if (alreadyCited) {
    return;
  }

  const citation: Citation = {
    id: `cite-${Date.now()}`,
    sourceId: state.selectedSourceId,
    addedAt: Date.now()
  };

  set({
    citations: [citation, ...state.citations]
  });
}

export function removeCitation(citationId: string): void {
  const state = researchStore.get();

  set({
    citations: state.citations.filter(
      (citation) => citation.id !== citationId
    )
  });
}

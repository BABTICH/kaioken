// src/shared/types/research.ts

export type ResearchSourceKind =
  | "web"
  | "repo"
  | "wiki"
  | "docs"
  | "paper";

export interface ResearchSource {
  id: string;
  title: string;
  kind: ResearchSourceKind;
  snippet: string;
  url?: string;
  path?: string;
  trustScore: number;
  tags: string[];
  publishedAt?: string;
}

export interface ResearchQuery {
  id: string;
  text: string;
  createdAt: number;
  status: "searching" | "complete" | "error";
  results: ResearchSource[];
}

export interface Citation {
  id: string;
  sourceId: string;
  addedAt: number;
}

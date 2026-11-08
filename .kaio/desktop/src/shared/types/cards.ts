// src/shared/types/cards.ts

export type CardStatus =
  | "draft"
  | "verified"
  | "stale"
  | "archived";

export interface KnowledgeCard {
  id: string;
  title: string;
  summary: string;
  body: string;
  tags: string[];
  status: CardStatus;
  source?: string;
  confidence?: number;
  createdAt: number;
  updatedAt: number;
}

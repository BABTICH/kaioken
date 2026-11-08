// src/shared/types/cost.ts

export interface CostEntry {
  id: string;
  date: string;
  sessionId: string;
  provider: string;
  model: string;
  inputTokens: number;
  outputTokens: number;
  costUsd: number;
  multiplier: number;
}

export interface MultiplierEvent {
  id: string;
  timestamp: number;
  from: number;
  to: number;
  reason: string;
}

export interface BudgetInfo {
  limitUsd: number;
  spentUsd: number;
  period: string;
}

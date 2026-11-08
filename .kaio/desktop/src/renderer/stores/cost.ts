// src/renderer/stores/cost.ts
import { atom } from "nanostores";
import type {
  BudgetInfo,
  CostEntry,
  MultiplierEvent
} from "@shared/types/cost";

export interface CostState {
  entries: CostEntry[];
  multiplierHistory: MultiplierEvent[];
  budget: BudgetInfo;
  selectedEntryId: string | null;
}

const initialState: CostState = {
  entries: [],
  multiplierHistory: [],
  budget: {
    limitUsd: 100,
    spentUsd: 0,
    period: "Current Month"
  },
  selectedEntryId: null
};

export const costStore = atom<CostState>(initialState);

function set(patch: Partial<CostState>): void {
  costStore.set({
    ...costStore.get(),
    ...patch
  });
}

function generateMockData(): {
  entries: CostEntry[];
  multiplierHistory: MultiplierEvent[];
  budget: BudgetInfo;
} {
  const entries: CostEntry[] = [];
  const multiplierHistory: MultiplierEvent[] = [];

  const providers = ["openai", "anthropic", "ollama"];
  const models = ["gpt-4.1", "claude-sonnet-4-5", "llama3"];

  let totalSpent = 0;
  const now = Date.now();
  const dayMs = 24 * 60 * 60 * 1000;

  for (let i = 29; i >= 0; i--) {
    const date = new Date(now - i * dayMs);
    const dateStr = date.toISOString().split("T")[0];

    const dailyEntries = Math.floor(Math.random() * 5) + 2;
    let currentMultiplier = 1;

    for (let j = 0; j < dailyEntries; j++) {
      const providerIdx = Math.floor(Math.random() * 3);
      const inputTokens = Math.floor(Math.random() * 8000) + 500;
      const outputTokens = Math.floor(Math.random() * 2000) + 200;

      // Mock pricing: roughly $3 per 1M input, $15 per 1M output
      const cost = (inputTokens * 0.000003 + outputTokens * 0.000015) * currentMultiplier;

      totalSpent += cost;

      entries.push({
        id: `cost-${i}-${j}`,
        date: dateStr,
        sessionId: `sess-${Math.random().toString(36).slice(2, 8)}`,
        provider: providers[providerIdx],
        model: models[providerIdx],
        inputTokens,
        outputTokens,
        costUsd: Number(cost.toFixed(4)),
        multiplier: currentMultiplier
      });

      // Occasionally change multiplier
      if (Math.random() > 0.85) {
        const nextMultiplier = Math.min(10, Math.max(1, currentMultiplier + (Math.random() > 0.5 ? 1 : -1)));

        if (nextMultiplier !== currentMultiplier) {
          multiplierHistory.push({
            id: `mult-${i}-${j}`,
            timestamp: date.getTime() + j * 1000,
            from: currentMultiplier,
            to: nextMultiplier,
            reason: nextMultiplier > currentMultiplier ? "Complex task detected" : "Task completed"
          });
          currentMultiplier = nextMultiplier;
        }
      }
    }
  }

  // Add some initial multiplier history
  multiplierHistory.unshift({
    id: "mult-init",
    timestamp: now - 30 * dayMs,
    from: 1,
    to: 1,
    reason: "Session initialized"
  });

  return {
    entries: entries.sort((a, b) => b.date.localeCompare(a.date)),
    multiplierHistory: multiplierHistory.sort((a, b) => b.timestamp - a.timestamp),
    budget: {
      limitUsd: 100,
      spentUsd: Number(totalSpent.toFixed(2)),
      period: "Current Month"
    }
  };
}

export function loadMockCostData(): void {
  const data = generateMockData();

  set({
    entries: data.entries,
    multiplierHistory: data.multiplierHistory,
    budget: {
      limitUsd: 100,
      spentUsd: data.budget.spentUsd,
      period: "Current Month"
    },
    selectedEntryId: null
  });
}

export function selectCostEntry(id: string | null): void {
  set({ selectedEntryId: id });
}

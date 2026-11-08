// src/renderer/components/cost/CostSurface.tsx
import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import { costStore, loadMockCostData } from "@renderer/stores/cost";
import { BudgetGauge } from "./BudgetGauge";
import { SpendChart } from "./SpendChart";
import {
  CostLedgerTable,
  MultiplierHistoryTable
} from "./CostTables";
import { RotateCw } from "lucide-react";

export function CostSurface() {
  const state = useStore(costStore);

  useEffect(() => {
    if (state.entries.length === 0) {
      loadMockCostData();
    }
  }, [state.entries.length]);

  return (
    <div className="cost-surface hud-scrollbar">
      <div className="cost-topbar">
        <span className="label label--accent">Cost Ledger</span>
        <span className="label">entries {state.entries.length}</span>

        <div style={{ flex: 1 }} />

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => loadMockCostData()}
        >
          <RotateCw size={12} />
          <span>Reload mock</span>
        </button>
      </div>

      <div className="cost-grid">
        <div className="cost-main">
          <SpendChart />
          <CostLedgerTable />
        </div>

        <aside className="cost-sidebar">
          <BudgetGauge />
          <MultiplierHistoryTable />
        </aside>
      </div>
    </div>
  );
}

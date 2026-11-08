// src/renderer/components/cost/CostTables.tsx
import { useStore } from "@nanostores/react";
import { costStore, selectCostEntry } from "@renderer/stores/cost";

function formatTime(ts: number): string {
  return new Date(ts).toLocaleString();
}

export function CostLedgerTable() {
  const state = useStore(costStore);

  return (
    <div className="cost-table-wrap">
      <div className="cost-table-head">
        <span className="label label--accent">Ledger</span>
        <span className="label">{state.entries.length} entries</span>
      </div>

      <div className="tw hud-scrollbar">
        <table className="tbl">
          <thead>
            <tr>
              <th>Date</th>
              <th>Model</th>
              <th className="c-nm">Tokens (In/Out)</th>
              <th className="c-nm">Cost</th>
              <th>Multiplier</th>
            </tr>
          </thead>
          <tbody>
            {state.entries.slice(0, 50).map((entry) => {
              const isActive = entry.id === state.selectedEntryId;

              return (
                <tr
                  key={entry.id}
                  className={isActive ? "is-active" : ""}
                  onClick={() => selectCostEntry(entry.id)}
                >
                  <td className="c-mut">{entry.date}</td>
                  <td className="c-tp">{entry.model}</td>
                  <td className="c-nm">
                    {entry.inputTokens.toLocaleString()} /{" "}
                    {entry.outputTokens.toLocaleString()}
                  </td>
                  <td className="c-lv">${entry.costUsd.toFixed(4)}</td>
                  <td>
                    <span className="multiplier-badge">
                      x{entry.multiplier}
                    </span>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export function MultiplierHistoryTable() {
  const state = useStore(costStore);

  return (
    <div className="cost-table-wrap">
      <div className="cost-table-head">
        <span className="label label--accent">Multiplier History</span>
        <span className="label">
          {state.multiplierHistory.length} events
        </span>
      </div>

      <div className="tw hud-scrollbar">
        <table className="tbl">
          <thead>
            <tr>
              <th>Time</th>
              <th className="c-nm">Change</th>
              <th>Reason</th>
            </tr>
          </thead>
          <tbody>
            {state.multiplierHistory.map((event) => {
              const isIncrease = event.to > event.from;

              return (
                <tr key={event.id}>
                  <td className="c-mut">{formatTime(event.timestamp)}</td>
                  <td className="c-nm">
                    <span className={isIncrease ? "c-up" : "c-down"}>
                      x{event.from} → x{event.to}
                    </span>
                  </td>
                  <td className="c-tp">{event.reason}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

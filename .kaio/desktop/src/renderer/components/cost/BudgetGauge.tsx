// src/renderer/components/cost/BudgetGauge.tsx
import { useStore } from "@nanostores/react";
import { costStore } from "@renderer/stores/cost";

export function BudgetGauge() {
  const state = useStore(costStore);
  const { limitUsd, spentUsd, period } = state.budget;

  const percent = Math.min(100, (spentUsd / limitUsd) * 100);
  const segments = 20;
  const filledSegments = Math.round((percent / 100) * segments);

  const isWarning = percent >= 75;
  const isCritical = percent >= 90;

  return (
    <div className="budget-gauge">
      <div className="budget-gauge-head">
        <span className="label label--accent">Budget</span>
        <span className="label">{period}</span>
      </div>

      <div className="budget-gauge-values">
        <span className="budget-spent mono">
          ${spentUsd.toFixed(2)}
        </span>
        <span className="budget-limit mono">
          / ${limitUsd.toFixed(2)}
        </span>
      </div>

      <div className="budget-segments">
        {Array.from({ length: segments }).map((_, i) => {
          const isFilled = i < filledSegments;

          let className = "budget-segment";
          if (isFilled) {
            if (isCritical) className += " is-critical";
            else if (isWarning) className += " is-warning";
            else className += " is-normal";
          }

          return <div key={i} className={className} />;
        })}
      </div>

      <div className="budget-gauge-foot mono">
        <span>{percent.toFixed(1)}% consumed</span>
        <span>${(limitUsd - spentUsd).toFixed(2)} remaining</span>
      </div>
    </div>
  );
}

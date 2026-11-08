// src/renderer/components/cost/SpendChart.tsx
import { useMemo } from "react";
import { useStore } from "@nanostores/react";
import { costStore } from "@renderer/stores/cost";

const CHART_WIDTH = 800;
const CHART_HEIGHT = 240;
const PADDING_LEFT = 40;
const PADDING_BOTTOM = 30;
const PADDING_TOP = 20;
const PADDING_RIGHT = 20;

export function SpendChart() {
  const state = useStore(costStore);

  const dailySpend = useMemo(() => {
    const map = new Map<string, number>();

    for (const entry of state.entries) {
      map.set(entry.date, (map.get(entry.date) ?? 0) + entry.costUsd);
    }

    return Array.from(map.entries())
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([date, cost]) => ({ date, cost }));
  }, [state.entries]);

  if (dailySpend.length === 0) {
    return (
      <div className="chart-empty">
        <p className="lead">No spend data available.</p>
      </div>
    );
  }

  const maxCost = Math.max(...dailySpend.map((d) => d.cost), 0.01);
  const drawWidth = CHART_WIDTH - PADDING_LEFT - PADDING_RIGHT;
  const drawHeight = CHART_HEIGHT - PADDING_TOP - PADDING_BOTTOM;
  const barWidth = drawWidth / dailySpend.length;
  const barGap = Math.max(1, barWidth * 0.2);
  const actualBarWidth = barWidth - barGap;

  const yTicks = [0, maxCost * 0.25, maxCost * 0.5, maxCost * 0.75, maxCost];

  return (
    <div className="spend-chart">
      <div className="spend-chart-head">
        <span className="label label--accent">Daily Spend</span>
        <span className="label">Last {dailySpend.length} days</span>
      </div>

      <div className="spend-chart-body hud-scrollbar">
        <svg
          viewBox={`0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`}
          preserveAspectRatio="none"
          className="spend-chart-svg"
        >
          {/* Y-axis grid lines and labels */}
          {yTicks.map((tick, i) => {
            const y = PADDING_TOP + drawHeight - (tick / maxCost) * drawHeight;
            return (
              <g key={`y-${i}`}>
                <line
                  x1={PADDING_LEFT}
                  y1={y}
                  x2={CHART_WIDTH - PADDING_RIGHT}
                  y2={y}
                  stroke="var(--rule)"
                  strokeWidth="1"
                  strokeDasharray="2 2"
                />
                <text
                  x={PADDING_LEFT - 8}
                  y={y + 3}
                  textAnchor="end"
                  className="chart-label"
                >
                  ${tick.toFixed(2)}
                </text>
              </g>
            );
          })}

          {/* Bars */}
          {dailySpend.map((day, i) => {
            const x = PADDING_LEFT + i * barWidth + barGap / 2;
            const barHeight = (day.cost / maxCost) * drawHeight;
            const y = PADDING_TOP + drawHeight - barHeight;

            const isSpike = day.cost > maxCost * 0.8;

            return (
              <g key={day.date}>
                <rect
                  x={x}
                  y={y}
                  width={actualBarWidth}
                  height={barHeight}
                  className={isSpike ? "chart-bar-spike" : "chart-bar"}
                />
                {i % 5 === 0 && (
                  <text
                    x={x + actualBarWidth / 2}
                    y={CHART_HEIGHT - 10}
                    textAnchor="middle"
                    className="chart-label"
                  >
                    {day.date.slice(5)}
                  </text>
                )}
              </g>
            );
          })}
        </svg>
      </div>
    </div>
  );
}

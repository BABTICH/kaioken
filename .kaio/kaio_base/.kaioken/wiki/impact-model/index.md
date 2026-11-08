# Impact Modeling

The impact module (`kaioken/impact`) converts a codebase's symbol graph into predictions about which files a change will affect. It exposes a single public entrypoint via `kaioken/impact/src/index.ts` (which re-exports internals), but the real logic lives in `predict.ts` and `render.ts`).

## Prediction Input

The module's surface is defined by two types in `kaioken/impact/src/predict.ts`:

- `PredictInput` (lines 41-48) — the high-level request. It contains `root` (the repository path), `symbol` (the symbol you changed), `content` (the symbol's current source).
- `ImpactReport` (lines 29-39). It is the output shape: a list of affected `files`, a confidence `score`, and an `explanation` string.

Both types are intentionally minimal — they force callers to provide raw source content, not precomputed metadata, so predictions are reproducible from a single snapshot.

## Prediction Engine

The core logic lives in `predictImpact` (`predict.ts:177-224`). This async function orchestrates the following pipeline:

1. **Build the graph** — it invokes internal graph construction (not exported), parsing the repository at `input.root` into nodes for files, classes, functions, and imports.
2. **Locate the symbol** — matches `input.symbol` against the graph's nodes; if absent, returns an empty report.
3. **Compute transitive dependents** — walks reverse edges from the symbol's node to find all files that import (transitively) the changed symbol.
4. **Score** — counts direct vs indirect dependents, penalizes deep chains (depth > 3) and favors files with higher fan-in.
5. **Explain** — generates a human-readable breakdown listing each dependent file and the path length.

A convenience wrapper, `predictImpactForSymbol` (`predict.ts:226-235`), loads a single file from disk given a `root` and `symbol` path, then calls `predictImpact` with `content` read from that file. It exists to support CLI and script usage where you already have the file on hand.

## Rendering

`renderImpact` (`kaioken/impact/src/render.ts:9-55`), takes an `ImpactReport` and returns an array of Markdown strings. The output is structured as:

- A header line listing the affected file count and confidence score.
- One entry per affected file, prefixed with `- \`path\` (score: X)` and followed by an indented explanation snippet.

The renderer is deliberately side-effect-free — it does not write to disk or mutate state, so callers can pipe its output into diffs, PR comments, or CI logs.

## Model Client Interface

Underlying the scoring is `ModelClient` (`predict.ts:16-18`), an interface with a single `complete` method (line 17). This abstraction lets the impact engine swap between local heuristics and a hosted LLM — the concrete client is chosen elsewhere (e.g., via dependency injection). The default implementation uses graph metrics only; a richer client could leverage `complete` to generate nuanced explanations from `ImpactReport` fields.

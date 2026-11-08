# Impact Modeling

The impact module predicts how editing a codebase symbol ripples through the rest of the repository. It consumes a `PredictInput` describing a single proposed change and returns an `ImpactReport` that quantifies affected files, downstream callers, and staleness risks. The module lives in `kaioken/impact/src/predict.ts` and is re-exported from `kaioken/impact/src/index.ts`.

## Core Data Types

The module's contract centers on two interfaces:

```ts
// kaioken/impact/src/predict.ts:41-48
interface PredictInput {
  root: string;           // Absolute path to the repository root
  symbol: string;         // Fully-qualified symbol name, e.g. "src/utils/formatDate"
}
```

```ts
// kaioken/impact/src/predict.ts:29-39
interface ImpactReport {
  symbol: string;
  files: Array<{ path: string; reason: string }>;  // Files whose content may change
  callers: Array<{ path: string; line: number; code: string }>; // Direct invocations
  summary: string;         // Human-readable, LLM-generated overview
}
```

The `files` list is the direct edit surface: it mirrors what the planning module would touch. `callers` are where downstream execution breaks if the signature changes. The `summary` is a natural-language digest for human review, not a machine contract.

## Prediction and Rendering

The primary entry point is `predictImpact`, which orchestrates the entire pipeline:

```ts
// kaioken/impact/src/predict.ts:177-224
async function predictImpact(input: PredictInput): Promise<ImpactReport>
```

It performs three sequential phases:
1. **Index and diff**: Scans the repo via the `IndexFile` abstraction (not exported here) to map symbols to file paths.
2. **Traverse callers**: Builds a reverse-dependency graph from the symbol's declaration, using AST analysis to find imports and call sites. This yields the `callers` array with exact line numbers for jump-to-definition UX.

3. **Provenance check**: Compares each visited file's cached lineage (from the provenance module) against its current content hash. Mismatches are flagged as stale.

Finally, it composes the `ImpactReport`, invoking a `ModelClient` to generate the `summary` field. The client interface is minimal:

```ts
// kaioken/impact/src/predict.ts:16-18
interface ModelClient {
  complete(opts: { system?: string; prompt: string; purpose?: string; maxOutputTokens?: number }): Promise<string>;
}
```

The default client uses the repository's configured LLM endpoint; `purpose` lets callers hint at prompt specialization (e.g. "summarize impact" vs. "suggest fixes"". The `maxOutputTokens` cap prevents runaway summaries on massive repos.

For convenience, `predictImpactForSymbol` wraps the common case where you have a root path and a symbol name only, omitting optional inputs:

```ts
// kaioken/impact/src/predict.ts:226-235
async function predictImpactForSymbol(root: string, symbol: string): Promise<ImpactReport>
```

This delegates to `predictImpact`. It exists so UI layers don't need to construct a full `PredictInput` when all they have is a symbol picker.

The rendering logic is not inside this module—the serve chapter covers UI—but the `callers` array's `line` fields are deliberately structured for table or diff-view rendering without post-processing. Additionally, the module provides `renderImpact`, which handles rendering impact reports for display.

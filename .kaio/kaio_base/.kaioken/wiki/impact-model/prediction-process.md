# Impact Modeling

## Prediction Process

The impact module turns a proposed change into an `ImpactReport` by combining three signals: a model's guess about what is related, a concrete scan of the codebase for those symbols, and a dependency-resolved hit list.

### The contract

Both entry points produce the same `ImpactReport` shape:

```ts
interface ImpactReport {
  // fields declared at kaioken/impact/src/predict.ts:29-39
}
```

The caller supplies a `PredictInput`, also declared at `kaioken/impact/src/predict.ts:41-48`. The input describes the change; the module does the rest.

### Predicting from a description

`predictImpact(input: PredictInput): Promise<ImpactReport>` (declared at `kaioken/impact/src/predict.ts:177-224`) is the primary path. It uses a `ModelClient` — the interface declared at `kaioken/impact/src/predict.ts:16-18` — to ask for candidate symbols. The client's only method is `complete(opts: { system?: string; prompt: string; purpose?: string; maxOutputTokens?: number }): Promise<string>` (declared at `kaioken/impact/src/predict.ts:17-17`). The module builds a system prompt explaining the repository layout and a prompt describing the change, then parses the model's output into candidate symbol names.

### Predicting from a symbol

`predictImpactForSymbol(root: string, symbol: string): Promise<ImpactReport>` (declared at `kaioken/impact/src/predict.ts:226-235`) is a shortcut: instead of prompting the model for candidates, it takes one symbol directly. `root` is the repository root, used to find source files; `symbol` is the exact name to trace.

### The shared pipeline

Both functions funnel into the same four stages:

1. **Candidate generation** — via the model (`predictImpact`) or the direct argument (`predictImpactForSymbol`). The model output is split into lines, each treated as a potential symbol name.

2. **Source file sweeping** — the module walks the repository from `root`, reading every file that could contain symbol definitions. It collects all declared/exported symbols and builds a map from symbol name to file path. This map is also used to resolve the model's candidates when they are not fully qualified.

3. **Dependency resolution** — for each matched file, the module parses its import/export statements and follows edges transitively. This is how it discovers *affected* modules, not just *mentioned* ones: a file that imports a changed symbol, and a file that imports that file, are both in scope.

4. **Report assembly** — the resolved set is grouped into three `ImpactReport` categories: affected modules, affected documents, and affected skills. The categories are disjoint (a file lands in exactly one). The report is returned as the promise's value.

### Why the pipeline is shaped this way

- **The model is a hint, not an oracle.** `predictImpact` treats the model's output as candidate names and validates them against the real file sweep. If the model hallucinates a symbol, it simply doesn't match a swept file and is dropped. This makes the module robust to poor completions.

- **The sweep is the source of truth for structure.** The walk is permissive (it reads broadly) but the matching is strict: only symbols actually present in the files count. This separation keeps the model's creativity from polluting the dependency graph.

- **Dependency resolution is transitive.** A change to `util/format` can break a doc that imports `components/Chart`, which imports `util/format` indirectly. The module follows that chain, so the report lists the doc even though the symbol never appears there.

- **`predictImpactForSymbol` exists for latency and determinism.** When you already know the symbol (e.g., from a diff parser or a user click), prompting the model adds a round trip and nondeterminism. The shortcut guarantees the same candidate in the same order every time, which is valuable for tests and CI.

### Scope notes

- The module is in `kaioken/impact/src/predict.ts`; the public surface is re-exported from `kaioken/impact/src/index.ts` with no additional declarations.
- The `ModelClient` is injected as a dependency, so callers can provide a mock or a real client without the module knowing which.
- The report's three categories — modules, documents, skills — are the only output granularity. If you need per-line impact, this is the wrong layer; the graph and provenance modules (covered in their own chapters) operate at that depth.

### Flow summary

```
predictImpact(input)                 predictImpactForSymbol(root, symbol)
        |                                      |
        v                                      v
   model.complete() -> candidates       single candidate
        |                                      |
        +-----------+--------------------------+
                    |
                    v
           sweep files from root
           build symbol -> file map
                    |
                    v
           match candidates to files
                    |
                    v
           resolve imports/exports transitively
                    |
                    v
           group into modules/documents/skills
                    |
                    v
              ImpactReport
```

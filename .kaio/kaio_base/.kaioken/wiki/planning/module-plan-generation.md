# Planning Module

The planning module is the pipeline's front-door: it turns a scanned repository into a `ModulePlan` that downstream stages (documentation, impact, provenance) can rely on. It lives in two files — `kaioken/plan/src/propose.ts` for proposing candidate plans, and `kaioken/plan/src/validate.ts` for checking, expanding, and normalizing those candidates.

## Proposal: Model First, Heuristics as Fallback

The entry point is `proposeModulePlan`, an async function that takes a `ScanResult` (the directory/file inventory from the scan module), an optional `IndexResult` (search index, may be `null` if indexing hasn't run), and an optional `ModelClient`. The `options` parameter currently only carries `multiplier`, which scales how many tokens the underlying model may use — a knob for cost/latency tradeoffs.

The shape of the call is deliberate: the caller supplies *evidence*, not a plan. `proposeModulePlan` decides how to interpret that evidence. If a `ModelClient` is provided, it constructs a prompt via `buildPrompt(evidence, depth)` where `depth` is the module's notion of nesting depth (from the Kaioken core types), and sends it to the model. The model's raw JSON response is then passed through `coerceModule`, a defensive parser that treats the model's output as untrusted structure. `coerceModule` returns either a `Module` or `null`; nulls are filtered out, and the result is wrapped in a `ProposeResult`.

If no model is available — or if model completion fails — `proposeModulePlan` falls back to `proposeHeuristicModules`, a deterministic structural clustering routine. It groups files by directory proximity and naming coherence, producing a shallow `Module[]` tree. The `_evidence?: RepositoryEvidence` parameter is accepted but currently unused, a placeholder for future heuristic refinement. This fallback exists so the pipeline never hard-depends on an external model; a repository with no API key still gets *some* plan.

## The Prompt: What the Model Is Told

`buildPrompt` assembles the system instruction shown below, interpolating the `RepositoryEvidence` (file paths, index summaries) and the requested `depth`. The `SYSTEM` constant is the model-facing contract:

```
You decompose a repository into modules for a documentation pipeline. A module is a coherent unit of purpose, not a directory listing. Group by what code is for; split a directory that does two jobs, and merge directories that serve one. Rules: - Use only file paths given in the evidence. Never invent one. - Every file you assign must appear verbatim in the evidence. - Prefer a shallow tree. Nest only when a parent genuinely has sub-parts. - Purpose is one or two sentences saying what the module is for and why it exists. Do not restate the module name. - It is correct to leave peripheral files unassigned. Coverage is not the goal. Reply with JSON only: {"modules":[{"id":"kebab-case-id","name":"Human Name","purpose":"...","files":["path"],"children":[]}]}
```

Three constraints here are load-bearing for the rest of the pipeline:

1. **Purpose over location.** The model is explicitly told that a module is "a coherent unit of purpose, not a directory listing." This prevents it from just mirroring the filesystem tree.
2. **Coverage is not the goal.** Peripheral files may be left unassigned. This keeps the plan focused on documented modules rather than exhaustive mapping — downstream stages only see what actually matters.
3. **Strict JSON with a fixed schema.** `files` entries must be verbatim file paths from the evidence, and `children` recursively nest sub-modules. The reply format is rigid precisely because `coerceModule` parses it untyped.

The `depth` parameter controls how deep the model is asked to nest — shallower for prompt efficiency, deeper when the caller knows the repo is hierarchical.

## Parsing the Model's Reply

`coerceModule` is the pendulum on the other side: it takes an arbitrary `unknown` and returns a `Module` or `null`. It validates that the object has a string `id` (kebab-case enforced), a `name`, a `purpose`, that `files` is an array of strings, and that `children` — if present — is itself an array that gets recursively coerced. Any violation yields `null`, so a malformed subtree is discarded rather than propagated. This is a deliberate cast-iron boundary: the model's free-form output is never trusted to become a `Module` without structural verification.

## Validation: Catching Structural Defects

Once a plan exists (from either path), `validatePlan(plan, scan)` runs it against the actual `ScanResult`. It returns a `PlanValidation` — a result type that aggregates defects. The validator checks the invariants that `proposeModulePlan` promised:

- **No invented files.** Every `files` entry in the plan must appear verbatim in the `scan`. A file that doesn't exist is a defect.
- **No duplicate assignments.** A file assigned to two modules is a defect — the downstream pipeline would have ambiguous ownership.
- **ID uniqueness.** Module ids must be distinct.
- **Purpose non-empty.** A module with a blank purpose is a defect, since the purpose is the unit-of-purpose contract.
- **Directory/path consistency.** File paths must be within the scan's root; a path escaping the scanned root is a defect.

The exact defect types are defined in the plan types (from the Kaioken core `plan` module), and `PlanValidation` carries them so the caller can decide whether to reject the plan outright or proceed with warnings. The contract is: `validatePlan` never mutates, it only reports. Fixing is the next stage's job.

## Expansion and Flattening

`expandDirectories(plan, scan)` is a normalizer: it walks the plan and for any module whose `files` contains a *directory* path (not a file), it replaces that directory entry with the concrete files found under it in `scan`. This exists because the model (or heuristics) may emit directory-level granularity, but downstream stages want file-level precision. The scan is the source of truth for the expansion, so the result still satisfies "every file exists in scan."

`flatten(modules)` is a complementary operation: given a `readonly Module[]` tree (with `children`), it returns a flat list of `Module`s — a depth-first traversal where each module appears once, with no nesting. Downstream stages that iterate linearly (e.g., card generation) prefer this flat form.

Two helpers support plan navigation:

- `findModule(plan, id)` — linear search for a module by id; returns `Module | null`. Used by validation to check id references and by the serve module to locate a specific module's subtree.
- `moduleScope(module)` — returns the set of file paths owned by `module` *and all its descendants*. This is the "scope" notion used when checking whether two modules overlap, or when determining which files a doc build should emit for a module.

## Pipeline Flow

A typical invocation wired into the Kaioken pipeline does:

1. `proposeModulePlan(scan, index, client, { multiplier })` → `ProposeResult` (with `modules`).
2. `validatePlan(result.modules_as_plan, scan)` → catch defects; reject or warn.
3. `expandDirectories(plan, scan)` → ensure file-level precision.
4. `flatten(plan.modules)` → get a flat `Module[]` for the downstream card generation and storage stages.

Both proposal paths (`proposeModulePlan` model route and `proposeHeuristicModules` fallback) return structurally identical `Module[]` trees, so the rest of the pipeline does not care which one produced them. The heuristic fallback is intentionally simple — it does not try to be smart about purpose; it just provides a reasonable structural baseline so documentation generation is never blocked.

The design tension the module resolves: a model produces semantically rich but structurally fragile output (hence `coerceModule`), while the pipeline needs deterministic guarantees (hence `validatePlan`). The fallback guarantees availability; the validation guarantees safety. Everything else — the prompt constraints, the coercion, the expansion — exists to shuttle between those two poles without ever trusting either side.

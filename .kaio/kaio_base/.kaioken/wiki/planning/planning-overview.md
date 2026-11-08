# Planning Module

The planning module (`kaioken/plan`) is the system's "what to build" layer: it defines the target module structure, breaks it down into cards, and validates that both are internally consistent and match the repo's existing conventions. It writes two artifact families under `KAIOKEN_DIR`: a single `module-plan.yaml` and a `cards/` directory of per-card YAML files.

## Data Model

All planning state flows through seven interfaces in `kaioken/plan/src/types.ts`.

`Module` (lines 12-21) describes a destination package in the workspace: its `name`, a `description`, and a set of `roots`? No — it holds the module's `id` and `name`, plus `dependencies` (other modules it imports from) and `tags` for categorization. It is a static descriptor, not a plan.

`ModulePlan` (lines 23-31) is the top-level artifact. It carries a `version`, `module` (a `Module`), and one array: `cards` (the execution units). It also has `provenance`, which ties the plan back to the source documents that motivated it — important for the provenance/staleness workflow described in other chapters.

`Card` (lines 34-53) is the atomic work item. Each card has an `id`, a human `title`, and a `description`. It references the files it touches via `files` (paths relative to the module root). A card carries `entryPoints` (`CardEntryPoint[]`) and `verification` (a `CardVerification` describing how to check it). Notably it has no explicit "done" flag — verification is the contract.

`CardEntryPoint` (lines 55-60) is a named location within a card's file set: it pairs a `path` with a `symbol`, describing where a specific function, class, or export should land.

`CardVerification` (lines 62-71) holds `commands` and `expectations` — shell commands to run and what output they should produce. This is the runtime definition of "does this card work".

`PlanDefect` (lines 74-87) and `PlanValidation` (lines 89-96) are the validation output. `PlanDefect` has `type` (a string category such as "missing-entry-point" or "duplicate-card-id"), a `message`, and a `path`. `PlanValidation` bundles `PlanDefect[]` as `defects` plus a `valid` boolean.

## Artifact I/O

The module's filesystem interface lives in `kaioken/plan/src/artifact.ts`, importing `KAIOKEN_DIR` from the foundation layer.

Two constants define where artifacts live:

```ts
// kaioken/plan/src/artifact.ts:7-8
export const MODULE_PLAN_ARTIFACT = join(KAIOKEN_DIR, "module-plan.yaml");
export const CARDS_DIR = join(KAIOKEN_DIR, "cards");
```

The path helpers `modulePlanPath(root)` and `cardsDir(root)` expand a workspace root into a full path to the plan file or cards directory, respectively. They exist so callers don't hardcode the join pattern.

Writing is straightforward:

- `writeModulePlan(root, plan)` serialises the `ModulePlan` to YAML at `modulePlanPath(root)` and returns the written path. Critical: it does this atomically via a temporary file, so a crash mid-write never leaves a half-written plan.
- `writeCard(root, card)` writes a single `Card` as YAML to `cardsDir(root)/<sanitised-id>.yaml`, returning the path.

Reading handles the inverse:

- `readModulePlan(root)` loads and parses the plan YAML, returning `null` if absent.
- `readCards(root)` reads every YAML file in the cards directory and returns an array of `Card`. It sorts results by `id` for deterministic processing order.

Validation is deferred to the caller — `readModulePlan` itself returns `ModulePlan | null` without asserting shape. That duty falls to `normalisePlan`.

### Normalisation

`normalisePlan(raw: unknown)` is the gatekeeper between untyped YAML and the typed `ModulePlan`. It accepts anything YAML parsing produced and returns a validated `ModulePlan` or `null` if the shape is wrong. It checks for required fields (`version`, `module`, `cards`), coerces string fields where needed, and rejects plans with unknown extra keys — a deliberate strictness so typos like `versoin` fail loudly rather than silently producing a crippled plan.

### File Name Safety

```ts
// kaioken/plan/src/artifact.ts:147-149
export function safeFileName(id: string): string
```

Card IDs can contain arbitrary characters (`:` for namespacing, `/` for hierarchy), which are illegal in filenames. `safeFileName` replaces those with safe substitutes while keeping the ID human-redeable. It's exported because downstream tooling (e.g., the serve module) needs to map between card IDs and file paths.

## Public API

`kaioken/plan/src/index.ts` has no indexed declarations of its own — it's a re-export barrel. The public surface is therefore:

- Type exports: `Module`, `ModulePlan`, `Card`, `CardEntryPoint`, `CardVerification`, `PlanDefect`, `PlanValidation`
- Value exports: `MODULE_PLAN_ARTIFACT`, `CARDS_DIR`, `modulePlanPath`, `cardsDir`, `writeModulePlan`, `readModulePlan`, `normalisePlan`, `writeCard`, `readCards`, `safeFileName`

The codebase also includes functionality for planning module heuristics and card generation, such as proposing module structures, validating plans, expanding directories, and generating cards from module plans. These capabilities support the planning workflow by automating the decomposition of work into verifiable units.

## Architectural Notes

The separation between `readModulePlan` (permissive) and `normalisePlan` (strict) is deliberate. Reading may be called on plans written by older tooling or handcrafted by users; normalising is the single choke point where all plans must conform. If you extend the schema, update `normalisePlan` — not the read path — so legacy plans still load.

Card verification is the contract for the rest of the system: the Kaioken core's `verify` logic (detailed in the Kaioken Core Concepts chapter) consumes `CardVerification.commands`/`expectations` to decide whether a card is done. The planning module only writes these down; it never executes them.

# Planning Module

The planning module orchestrates the pipeline between a repository scan and the final generated cards. It proposes a module structure, validates and expands it against the scan, then generates per-module cards using a model client. The module is deliberately split into four concerns: `propose.ts` (module discovery), `validate.ts` (plan sanity-checking and normalization), `cards.ts` (card generation and verification), and `artifact.ts` (persistence).

## Types and Contracts

The core interfaces live in `types.ts`. `Module` is the minimal unit of organization — it carries an `id`, a `title`, and a list of file paths it owns. `ModulePlan` aggregates modules plus a `depth` hint that downstream consumers use to calibrate prompt complexity (deeper plans get richer prompts). `Card` extends `Module` with generated content fields: a `summary`, `body`, `references`, and an optional `entryPoints` array. `CardEntryPoint` captures a symbol name and depth. `CardVerification` records whether a draft passed validation, plus any defects found and the evidence used. `PlanDefect` and `PlanValidation` describe plan-level problems: `PlanValidation` carries a list of `PlanDefect`s plus the validated `ModulePlan`.

These types are the contract between the planner and the rest of the repo — the `scan` and `index` results passed in are defined in their own modules, and `Module`, `ModulePlan`, and `Card` are consumed by downstream writers in the `serve` and `skillgen` modules.

## Proposal (`propose.ts`)

`proposeModulePlan` is the entry point. It takes a `ScanResult`, an `IndexResult` (nullable), and an optional `ModelClient`. When a client is provided it builds a prompt via `buildPrompt` (which uses `RepositoryEvidence` and `Depth` to craft a context-aware request) and asks the model for a module plan. When no client is given — or the model call fails — it falls back to `proposeHeuristicModules`, a deterministic, scan-driven approach that groups scanned files into modules based on directory structure and naming patterns.

`ProposeResult` wraps the outcome: either a `ModulePlan` or a list of `PlanDefect`s explaining why proposal failed. The `multiplier` option (passed through to `buildPrompt`) lets callers tune the depth heuristic — a higher multiplier yields broader module coverage.

The heuristic path is important because it makes the planner usable offline and testable without a live model. It never throws; it always returns a `ProposeResult`.

## Validation (`validate.ts`)

`validatePlan` is the gatekeeper. It checks a proposed `ModulePlan` against the `ScanResult` and returns `PlanValidation`. Validation catches:
- Modules that reference files not present in the scan.
- Modules that overlap (a file claimed by two modules).
- Modules with zero files after preprocessing.
- Modules whose IDs collide or contain unsafe characters.

Defects are emitted as `PlanDefect`s with a severity and message. `expandDirectories` is a normalization pass that turns directory entries from the scan into explicit file lists — it runs *before* validation so that directory-based module definitions are checked against real files. `flatten` recursively flattens nested module arrays into a single list, which validation uses internally. `findModule` is a simple linear search helper, and `moduleScope` returns the list of file paths a module owns — used by both validation and card generation to know what files a card can cite.

The key design choice: validation is **not** mutation-based. It returns a new `ModulePlan` even when it finds defects, so callers can choose to proceed with warnings or abort. `expandDirectories` is the only mutator, and it's intentionally separate so the raw proposal remains inspectable.

## Card Generation (`cards.ts`)

`generateCard` is the per-module workhorse. It takes a `Module`, the scan's `IndexResult` (for symbol evidence), a `ModelClient`, and options controlling `multiplier`, an `oracle` (`SymbolOracle`), and `knownFiles` (a map of path → content hash). It builds a context-specific prompt with `buildCardPrompt` (which uses `ModuleEvidence` and `Depth`), calls the model, parses the response into a `CardDraft`, then passes it to `verifyCard`.

`verifyCard` is the critical quality gate. It takes the draft, the module, `ModuleEvidence`, and the `oracle`, and returns a `CardVerification`. The verifier checks:
1. Every symbol mentioned in the card's `entryPoints` actually exists in the module's files (via the oracle).
2. Every file referenced in the card's `references` is present in the module's scope.
3. The card's `summary` and `body` are non-empty and within length bounds.

When verification fails, the draft is rejected and `generateCard` returns a `CardResult` marking failure — it does **not** retry. Failures are surfaced to the caller so they can decide on escalation.

`generateCards` orchestrates many cards. It takes a `ModulePlan`, iterates its modules, and calls `generateCard` for each. The `only` option filters to specific module IDs; `onProgress` reports (moduleId, done, total) for UI feedback. Two advanced options work together:
- `incremental` when true, only regenerates cards whose source files changed or are missing. It uses `existingCards` (or loads from `root` via `readCards`) to compare content hashes.
- `knownFiles` from the scan provides the path → hash map so the verifier can distinguish "no declarations" from "no such file" and supply provenance hashes.

`buildCardPrompt` constructs a prompt that includes the module's file evidence, the depth, and instructions to produce structured output. It's separate from `generateCard` so callers can inspect or modify the prompt without reimplementing the orchestration.

## Persistence (`artifact.ts`)

The artifact module defines the on-disk contract. `MODULE_PLAN_ARTIFACT` points to `kaizen/module-plan.yaml` (via the `KAIOKEN_DIR` constant from the foundation layer), and `CARDS_DIR` to `kaizen/cards/`. `modulePlanPath` and `cardsDir` resolve these for a given repository root.

`writeModulePlan` serializes a `ModulePlan` to YAML and returns the path; `readModulePlan` loads it back, returning null if absent. `normalisePlan` is the safety net: it takes raw unknown data (from a file or model output) and coerces it into a `ModulePlan`, returning null if the shape is wrong. This is defensive — the planner never trusts external input blindly.

For cards, `writeCard` writes a single card to a file named via `safeFileName(card.cardId)` under `CARDS_DIR`. `readCards` scans that directory and returns all cards, ignoring unparseable files. `safeFileName` strips characters unsafe for filesystems — important because module IDs come from user input and model output.

The separation between `validate.ts` (pure logic) and `artifact.ts` (I/O) means the planner can be tested in-memory without touching disk, and the persistence layer can evolve (e.g., switching from YAML to JSON) without altering the validation rules.

Coverage of the module's exports is strong: 89% of declarations are covered. Among the uncovered declarations are `DirectoryEvidence` and `ModuleFileEvidence` (evidence types used by the planner), as well as `gatherEvidence` and `gatherModuleEvidence` (functions that collect evidence for the planner's prompts and verification).

## API Summary

The module's public surface is the four functions that compose a pipeline:

1. `proposeModulePlan(scan, index, client?, options?)` → `ProposeResult`
2. `validatePlan(plan, scan)` → `PlanValidation` (plus `expandDirectories` for normalization)
3. `generateCards(plan, index, client, options?)` → `CardResult[]` (each `CardResult` wrapping a `Card` or failure details)
4. `writeModulePlan` / `writeCard` for persistence, `readModulePlan` / `readCards` for loading.

Everything else (`buildPrompt`, `buildCardPrompt`, `verifyCard`, `flatten`, `findModule`, `moduleScope`, `normalisePlan`, `safeFileName`) is internal — but exported so the module can be tested in isolation and so downstream modules (like `skillgen`) can reuse the verification logic without reimplementing it.

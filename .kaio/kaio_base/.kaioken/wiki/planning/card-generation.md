# Card Generation and Verification

## One card per leaf module

The card pipeline lives entirely in `kaioken/plan/src/cards.ts`. A "knowledge card" is a compact JSON object describing one module: a `summary`, a list of `keyPoints`, and `entryPoints` naming concrete declarations and their files. Cards are read by agents before doing work, so the whole pipeline is built around keeping them *true* — never inventing a declaration, never referencing a file that doesn't exist.

The pipeline is deliberately split into two stages that share an interface:

- `generateCard` is the optimistic pass — prompt the model, parse the reply, verify. 
- `verifyCard` is the adversarial pass — it actively tries to find false claims in a draft, and if it finds any, returns a `CardVerification` whose defects drive a correction prompt.

`generateCard(module, index, client, options)` takes a `Module`, an `IndexResult` for background context, and a `ModelClient`. The `options` object carries `multiplier` (number of candidate drafts to try), an `oracle` (a `SymbolOracle` that can confirm or deny symbol existence), and `knownFiles` — a path→content-hash map from the scan, which lets the verifier distinguish "this file has no such declaration" from "this file doesn't exist at all". The function returns a `Promise<CardResult>`.

The system prompt `SYSTEM` (lines 8-22) is what makes the card *specific to this repository*. It tells the model:

> Use only declarations and file paths given in the evidence. Never invent one. Entry points must name a declaration that appears in the evidence, with the file it appears in. Say what the code does and why it is shaped that way.

That last clause is the anti-generic-prose mandate; the prompt refuses filler outright. `CRITIQUE_SYSTEM` (lines 24-28) governs the correction pass with an equally tight scope: *fix exactly what the defect report names, change nothing else*.

## Verification as a defect hunt

`verifyCard(draft, module, evidence, oracle)` returns a `CardVerification`. The `evidence` is a `ModuleEvidence` (the file paths and declarations the card legitimately had access to), and `oracle` is the ground-truth source. The verifier applies several checks:

- Every `entryPoint.name` must resolve via `oracle` to an actual declaration.
- Every `entryPoint.file` must be a real file in the module, not just a string that looks like a path.
- `knownFiles` lets the verifier detect *missing* references (the model forgot to cite a file that exists) versus merely stale ones.

The helper `nameCandidates(written)` handles the pragmatic reality that the model will write a symbol reference in several forms — the literal string, and possibly a qualified or shortened variant. The verifier checks each candidate form so a false positive is caught even if the model wrote the bare name.

`defectCount(verification)` is a trivial sum used by the retry logic: if a draft has more than zero defects, `generateCard` builds a correction prompt and tries again.

## The prompt builders

Two functions construct the two passes' prompts:

- `buildCardPrompt(module, evidence, depth)` — builds the initial generation prompt. `depth` is a `Depth` from the plan (leaf vs. parent); get it wrong and the model is told to produce a card for a module that owns no files.
- `buildCorrectionPrompt(draft, verification, evidence)` — takes the previous draft and the defect list, and embeds both into a prompt that starts from `CRITIQUE_SYSTEM`. The instruction is *fix exactly this*, so the model won't go off adding new content unprompted.

`parseCard(reply)` is a strict JSON parser: it takes the model's raw reply string and returns a `CardDraft`. It does *not* validate against the schema — that's `verifyCard`'s job. The separation means the model can be told "reply with JSON only" in `SYSTEM`, and any malformed reply surfaces as a parse failure rather than a silent drop.

## Batch generation with incremental smarts

`generateCards(plan, index, client, options)` is the entry point that drives the whole module. It takes a `ModulePlan`, flattens it with `flattenLeaves(plan)`, and iterates over every leaf module. `flattenLeaves` is the critical filter: a parent module that only groups children gets *no card* — it owns no files, so a card would be pure invention. Only modules that own files get cards.

The `options` object is where the incremental machinery lives:

- `only?: string[]` — restrict generation to specific module IDs.
- `knownFiles` — the path→hash map from the scan, forwarded to `generateCard` and `verifyCard` so the verifier has provenance hashes.
- `onProgress` — a callback `(moduleId, done, total)` so callers can render progress bars.
- `incremental?: boolean` — when true, only regenerate cards whose source files changed *or* are missing.
- `existingCards` — previously generated cards to consider for reuse.
- `root` — repository root, used to load existing cards from disk if `existingCards` isn't passed.

The freshness check lives in `isCardFresh(card, module, knownFiles)`. Given an existing card and the module it was written for, it asks: has any source file the card depends on changed since the card was written? It compares the card's stored provenance hashes against the current `knownFiles` map. If a file listed in the card is missing from `knownFiles` (deleted) or has a different hash (edited), the card is *not* fresh and must be regenerated. If the card covers a module with no files at all, it's fresh by definition — there's nothing to go stale.

When `incremental` is true, `generateCards` iterates the flattened leaves, and for each leaf:

1. Look up the existing card (from `existingCards` or by loading `root/<module-id>.card.json`).
2. Call `isCardFresh`. If fresh, keep it as-is — no model call, no cost.
3. If stale or missing, run the full `generateCard` pipeline.

The `multiplier` option in both `generateCard` and `generateCards` lets callers retry generation with fresh prompts when verification keeps failing — with `defectCount` driving the retry loop.

## The empty index fallback

`emptyIndex()` returns a fresh, empty `IndexResult`. It exists so that when the underlying index is unavailable (e.g., a scan that produced no index entries), the card pipeline can still run — the verifier just has no cross-reference data, so it relies entirely on the `oracle` and `knownFiles` for correctness checking. This is a deliberate degradation: cards are still *verifiable* (against the symbol oracle and file map) even when the full index is missing.

## Data flow in one paragraph

`flattenLeaves` picks the modules → `buildCardPrompt` constructs the generation input from `ModuleEvidence` + `Depth` → model replies with JSON → `parseCard` yields a `CardDraft` → `verifyCard` runs the oracle and file checks, producing a `CardVerification` → if `defectCount > 0`, `buildCorrectionPrompt` constructs a targeted fix prompt and the loop repeats (up to `multiplier` times) → on success, the `CardResult` is returned and accumulated. In batch mode, `isCardFresh` short-circuits the entire loop for modules whose cards haven't gone stale, making incremental regeneration cheap for large repositories.

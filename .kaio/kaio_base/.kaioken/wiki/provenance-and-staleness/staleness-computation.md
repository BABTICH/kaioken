# Computing Staleness

## The provenance contract

Every generated document carries a `Provenance` record (in `types.ts`) listing the `sources` it was written from. Each `ProvenanceSource` identifies either a whole file (`path` only), a line range (`path` + `startLine`/`endLine`), or a named symbol (`path` + `symbol`). Whole-file sources also carry a `hash` computed at generation time; symbol- and range-based sources **do not** — they must be re-evaluated against the current file content.

The staleness machinery in `staleness.ts` answers one question: *is what I generated last time still a faithful representation of what is on disk now?* It never re-reads file content itself. It trusts the `ScanResult` handed to it, which contains the current per-file hashes (`ReadonlyMap<string, string>` keyed by path) and optionally a symbol-hash map for symbol-aware comparison.

## The public entry points

`computeStaleness(documents, scan, options?)` is the orchestrator. It cross-references every `Provenance` against the `ScanResult` and produces a `StalenessReport` — a per-document status plus aggregate counts. `invalidatedBy(documents, changedPaths, options?)` does the inverse: given a set of paths known to have changed (e.g. from a file watcher), it returns which documents must be regenerated. `changedSourcesFor(record, scan, options?)` computes the minimal diff — only the sources that actually changed — so a model regenerating a document can be told *what* changed, not just *that* something changed.

The `ProvenanceIndex` interface (in `types.ts`) provides a structured way to look up provenance records by document or source, enabling efficient cross-referencing during staleness checks. The `StalenessOptions` interface allows callers to configure how staleness is computed, including whether symbol-hash maps are supplied for symbol-aware comparison and how invalidation matching should behave.

## Status classification

`status(record, current, symbolHashes?)` classifies a single document via three nested steps:

1. `classifySource` — for each `ProvenanceSource`, compare against the current maps:
   - **changed** — the file exists but its hash differs, OR it is a symbol/range source whose symbol hash differs, OR the source references a symbol that no longer exists in the current symbol map.
   - **deleted** — the path is absent from the current file-hash map entirely.
   - **unchanged** — the path hash matches (for whole-file) or the symbol/range hash matches.

2. `freshnessOf` — collapse the per-source classifications into a `Freshness` value using a strict ordering:
   - If **any** source is `changed` → `"stale"` (comment in `types.ts` says "At least one source changed").
   - Else if **any** source is `deleted` → `"orphaned"` ("Every source it was written from is gone" — though note a mix of deleted+unchanged still yields orphaned today; the code does not distinguish partial deletion).
   - Else if the record has **zero** sources → `"unknown"` (nothing to compare).
   - Else → `"current"`.

3. Combine into a `DocumentStatus` with the `freshness` value; the counts of changed and deleted sources are computed from the per-source classification totals.

Note the asymmetry: a document with one changed and one unchanged source is `stale`, not partially anything. The `Freshness` type in `types.ts` has no "partially stale" branch — staleness is existential.

## Binding keys

`bindingKeyFor` is how a `ProvenanceSource` becomes a stable string identity for diffing and invalidation:
- Whole-file sources → `null` (they hash by path alone).
- Symbol sources → `symbolBindingKey(path, symbol)` → `path#symbol`.
- Range sources → `rangeBindingKey(path, startLine, endLine)` → `path:start-end`.

These keys are used internally by `changedSourcesFor` to correlate recorded bindings with the current scan's symbol/range hashes. They also serve as the deduplication key when multiple `Provenance` records reference the same symbol.

## What gets regenerated

`invalidatedBy` is the fast path. It takes `changedPaths` (a set of paths known to have changed) and returns every document whose `Provenance` records touch those paths. For whole-file sources this is a direct path lookup. For symbol/range sources, `bindingKeyFor` computes the binding key, then checks whether the changed path is a prefix of that key. The `InvalidationOptions` let callers tighten or loosen the match (e.g. require exact path match vs. allow directory-prefix matching for monorepo layouts).

## The changed-source diff

`changedSourcesFor` is the slimmest possible regeneration payload. For a given `Provenance`, it walks each source, classifies it against the current scan, and returns only the `ChangedSource` entries whose status is not `"unchanged"`. Each `ChangedSource` carries the `ProvenanceSource` plus a `status` (`"changed"` | `"deleted"`) and, for changed whole-file sources, the new hash. This is the input a model consumes when regenerating a document — it sees exactly which files moved, not the entire source list.

Symbol-aware invalidation requires the caller to supply `options.symbolHashes` — a map from `path#symbol` keys to current hashes. Absent that, symbol sources are conservatively treated as changed if the file itself changed, since there is no way to tell whether the specific symbol moved.

## Lockfiles and config basenames

Two constant sets in `staleness.ts` define what `isDocumentableFile` accepts:

- `LOCKFILES` — package-manager lockfiles (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lock`, `cargo.lock`, `gemfile.lock`, `composer.lock`, `poetry.lock`, `pipfile.lock`, `pubspec.lock`, `podfile.lock`, `flake.lock`, `npm-shrinkwrap.json`, `go.sum`).
- `CONFIG_BASENAMES` — project configuration files (`package.json`, `makefile`, `justfile`, `jenkinsfile`, `dockerfile`, `containerfile`, `.editorconfig`, `.nvmrc`, `.node-version`, `biome.json`, `biome.jsonc`).

These basenames are matched case-insensitively (files like `Dockerfile` or `Makefile` are captured) because they are the files whose *existence or absence* changes what documentation should say — they are not source code but they constrain it. `isDocumentableFile(file: FileRecord)` returns true only for these special basenames **or** for regular source files (recognized by extension — the implementation checks the `FileRecord`'s `kind`/`lang` fields). Lockfiles and config files are deliberately included because badgering a model with lockfile diffs is noise, but a new `package.json` dependency is a signal that documentation about the project's dependencies is now wrong.

## The status report shape

`StalenessReport` (in `types.ts`) aggregates the per-document `DocumentStatus` list, plus counts for documents classified as current, stale, and orphaned. The `unknown` freshness is deliberately not counted in any bucket — it is the "we have no idea" signal that should surface as an unclassified document, not as a wart in a summary statistic.

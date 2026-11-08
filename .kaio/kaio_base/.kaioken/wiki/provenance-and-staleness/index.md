# Provenance and Staleness

## Why provenance exists

Every artifact in the repository — a doc, a card, a report — is generated from source documents. `kaioken/provenance/src/types.ts` defines the contracts that let the rest of the system answer one question: *is this artifact still trustworthy?*

```ts
types.ts:43-50
export interface Provenance {
  id: string;
  sources: ProvenanceSource[];
  createdAt: string;
  updatedAt: string;
}
```

Each `ProvenanceSource` (lines 16–40) pins down *what* the artifact was written from:

- `path` — the file it came from.
- `kind` — how it was read (`"document"`, `"symbol"`, `"range"`).
- `symbol` / `startLine` / `endLine` — only set for `"symbol"`/`"range"` kinds.
- `hash` — the content hash at generation time.
- A stable identity string is computed by `bindingKeyFor` below.

The `hash` plus the stable identity are the core of change detection. If the identity is still present in the current scan but the hash differs, the source changed. If the identity is gone entirely, the source was deleted or renamed.

## Staleness computation

`computeStaleness` in `kaioken/provenance/src/staleness.ts:28-77` is the entry point. It takes the full set of `Provenance` records a project has, plus the latest `ScanResult`, and returns a `StalenessReport` (defined at `types.ts:118-141`) that groups those records into three buckets: `stale`, `current`, and `orphaned`.

The logic is deliberately simple — there is no graph walking here, no dependency resolution. For each document, the staleness checker:

1. Reconciles each recorded source against the scan.
2. If **every** source is still present and their hashes match → `"current"`.
3. If **at least one** source has changed (different hash for the same binding key identity) → `"stale"`.
4. If **every** source is missing from the scan → `"orphaned"` (heuristic: the document was likely deleted or renamed).
5. If a document records **no sources at all**, its status is `"unknown"` — we can't say anything about it. This matters for generated artifacts that only embed their own content.

The `Freshness` type (`types.ts:59-66`) encodes exactly these four states as a discriminated union, and `DocumentStatus` (`types.ts:68-78`) pairs each `Provenance` with its `Freshness` plus a list of `ChangedSource` entries explaining *why* — so the caller can render a human-readable reason without re-deriving it.

## Change detection: the site of the lossy compression

The hard part is change detection. Files come and go, and a symbol moving within a file must not orphan every document that quoted it. That's why `bindingKeyFor` (`staleness.ts:211-220`) exists:

```ts
staleness.ts:211-220
export function bindingKeyFor(source: ProvenanceSource): string | null {
  switch (source.kind) {
    case "document":
      return `doc:${source.path}`;
    case "symbol":
      return symbolBindingKey(source.path, source.symbol);
    case "range":
      return rangeBindingKey(source.path, source.startLine, source.endLine);
  }
}
```

A document-level binding is just the path — renamed files are detected as orphaned. A symbol binding is `path#symbol`, which survives line shifts. A range binding is `path:start-end`, which is exact but brittle; the intuition is that ranges are only used for short-lived artifacts (like a card quoting three lines) and the staleness should degrade gracefully if the range moved.

`symbolBindingKey` and `rangeBindingKey` are tiny helpers that format those strings deterministically so the same source always produces the same key, which is what makes reconciliation possible at all.

## Changed sources, invalidated documents

`changedSourcesFor` (`staleness.ts:177-208`) is the per-record worker. Given one `Provenance` and the scan, it returns the `ChangedSource[]` for that record — one entry per source that either changed its hash or disappeared. It takes `Pick<StalenessOptions, "symbolHashes">`, so the caller can pass a precomputed map of `symbolBindingKey → hash` to avoid re-hashing every symbol for every document; the staleness report reuses this map across all documents so a 10k-document repo doesn't hash the same symbols ten thousand times.

`invalidatedBy` (`staleness.ts:149-167`) is the inverse operation: given a set of changed paths (e.g. from a file watcher), return the `id`s of every document whose provenance references at least one of those paths. This is what the watch/incremental-rebuild path uses — it avoids a full re-scan when only a few files changed. Note it takes the raw path list, not a full `ScanResult`, because the caller already knows which files changed; it does not re-check whether the source still exists.

## Options: resolving the "it exists but I haven't read it" problem

Both `StalenessOptions` and `InvalidationOptions` (`types.ts:81-106`) carry options for how to handle unresolved sources and performance. When unresolved sources are skipped (the default), a source whose binding key points at a path that exists in the scan but whose hash wasn't resolved — e.g. because the scan only computed document hashes, not per-symbol content — is *skipped* rather than counted as changed. When you can guarantee every binding kind is fully populated, which is the case in the full pipeline but not in partial scans, you can opt into counting them.

`StalenessOptions.symbolHashes` is the performance escape hatch; pass it if you already computed all symbol hashes, and the functions will use it instead of re-hashing. The staleness module never calls the filesystem itself — it works entirely off the `ScanResult` passed in, which keeps it testable and lets callers (like the watcher) decide what granularity of scan to pay for.

## What counts as documentable

Finally, `isDocumentableFile` (`staleness.ts:272-307`) is the filter that decides whether a `FileRecord` is worth tracking provenance for at all. It excludes generated output dirs (anything under `dist/`, `out/`, `node_modules/`, etc.), lockfiles, images, and other binary content — the point being that provenance is only meaningful for *source* files that a human (or an AI) can edit and regenerate from. Keeping this as a pure predicate over `FileRecord` means the staleness pipeline and the scan module share the exact same notion of "what is a source," and they cannot drift apart in behavior.

The `ProvenanceIndex` interface (`types.ts:52-56`) provides a way to organize and look up provenance records efficiently, and `readProvenanceIndex` and `gatherProvenance` are available for loading and collecting provenance data from disk and scans respectively. `checkDrift` and `readCardsSafe` provide additional utilities for detecting inconsistencies between provenance records and the actual state of the repository, while also safely handling card data that may be absent or malformed.

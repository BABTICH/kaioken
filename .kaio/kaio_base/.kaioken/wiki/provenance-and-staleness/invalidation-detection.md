# Provenance and Staleness

## Change Detection and Invalidation

### Identifying Changed Sources

The staleness module compares what a document originally depended on against what a fresh scan of the filesystem reveals. The heart of this is `computeStaleness(documents, scan, options?)`, which walks every provenance record and a `ScanResult` to produce a `StalenessReport`. Each `Provenance` entry records the source files it was built from along with their hashes at build time; the scan provides current hashes. When the two diverge, the document is stale.

For any individual document, `status(record, current, symbolHashes?)` classifies every recorded source via `classifySource(source, current, symbolHashes?)`. A source is `"deleted"` if its path no longer appears in the scan, `"changed"` if its hash differs, and `"unchanged"` otherwise. The overall freshness is then derived by `freshnessOf(record, changed, deleted)`, which returns `DocumentStatus["freshness"]` — effectively aggregating deleted sources as more severe than merely changed ones.

Crucially, hashes are not always whole-file. Symbol-aware sources carry a `symbol` field, and for those the staleness check needs to know whether *that symbol's* implementation changed, not just whether the file was touched. That is why `classifySource` accepts an optional `symbolHashes` map: when present, it looks up the hash for the source's `symbolBindingKey` rather than comparing the file hash. This lets the system avoid invalidating a document when an unrelated part of a large source file changes.

### Binding Keys

The mapping from a symbol-aware source to a comparable hash is via a *binding key*:

```ts
// kaioken/provenance/src/staleness.ts:211-230
export function bindingKeyFor(source: ProvenanceSource): string | null
export function symbolBindingKey(path: string, symbol: string): string
export function rangeBindingKey(path: string, startLine: number, endLine: number): string
```

`bindingKeyFor` returns `null` for whole-file sources (they use the raw file hash) and the appropriate composite key otherwise. `symbolBindingKey` produces `path#symbol`; `rangeBindingKey` produces `path:start-end`. These keys are what the scan populates in its symbol hash map, so the lookup in `classifySource` is direct. The distinction matters because a `ProvenanceSource` can be either a whole file, a named symbol within a file, or a line-range within a file — and each requires a different granularity of change detection.

### Computing Invalidations

Given a set of changed paths, `invalidatedBy(documents, changedPaths, options?)` answers: which documents must be regenerated? It does not naively match paths. A document is invalidated if any of its sources is affected by a changed path — which, for symbol-bound sources, means the changed path is the source's file *and* the symbol's binding key appears in the scan's changed-symbol set. The `InvalidationOptions` can carry those symbol hashes so that a change to line 40 of a file does not invalidate a document bound to lines 1–10 of the same file.

This function is the inverse of `computeStaleness` in a sense: staleness asks "given the current scan, is this document outdated?", while invalidation asks "given these specific changes, which documents should I proactively mark dirty?". The latter is used by callers that have a diff or an event stream and want to avoid a full rescan.

### The Diff for Regeneration

When a document *is* stale, the model regenerating it should not be shown every changed file — only the ones that actually matter. `changedSourcesFor(record, scan, options?)` computes exactly that. It returns `ChangedSource[]`, a filtered and shaped list of sources for one `Provenance` record, using the same symbol-awareness as `classifySource`. The optional `symbolHashes` in `options` again lets symbol-bound sources be compared at symbol granularity. The result is the minimal evidence set to hand to a regeneration prompt.

### File Classification

Finally, `isDocumentableFile(file: FileRecord)` decides whether a scanned file is the kind of thing documentation should describe at all. It is a whitelist/blacklist guard: the `LOCKFILES` set (package locks, `go.sum`, etc.) and `CONFIG_BASENAMES` set (`package.json`, `Makefile`, `.editorconfig`, and friends) are explicitly excluded — the former because they are generated and meaningless to document, the latter because they are configuration that changes constantly and would cause needless staleness churn. Everything else (source code, markdown, prose) is documentable. This filter is applied during scans so that provenance records never reference files that would only serve to make every document perpetually stale.

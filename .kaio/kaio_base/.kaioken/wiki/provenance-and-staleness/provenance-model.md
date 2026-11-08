# Provenance and Staleness

## Provenance Model

The provenance subsystem answers three questions about a generated document: *what was it written from*, *when was it written*, and *is it safe to trust now*. Its data model lives in `kaioken/provenance/src/types.ts`.

### `ProvenanceSource`

Every input material a document was generated from — a fetched research page, a scanned file, a chat transcript — becomes a `ProvenanceSource`. It records the material's identity and its state at generation time:

```ts
interface ProvenanceSource {
  id: string;
  uri: string;
  contentType: string;
  capturedAt: number;
}
```

`id` is a stable identifier for the source material (typically a hash of its content or its storage key). `uri` locates it in the repository or externally. `capturedAt` is a wall-clock timestamp — not a version number — because the staleness logic compares it against *when the document was generated*, not against a monotonically increasing sequence. If source materials could arrive out of order (e.g., re-fetched pages), a timestamp is the only safe common clock.

### `Provenance`

A `Provenance` binds a set of sources to one artifact. It is deliberately flat:

```ts
interface Provenance {
  artifactId: string;
  sources: ProvenanceSource[];
  generatedAt: number;
}
```

`artifactId` is the generated document's ID; `sources` is the complete list of inputs that went into it; `generatedAt` is when the generation finished. There is deliberately no "version" field — the model assumes staleness is derived, never stored. Storing a version would require updating it on every re-generation, and the whole point of the staleness computation is to avoid that bookkeeping.

### `ProvenanceIndex`

A `ProvenanceIndex` is the in-memory or persisted lookup structure that maps artifact IDs to their `Provenance` records:

```ts
interface ProvenanceIndex {
  [artifactId: string]: Provenance;
}
```

It exists so that staleness queries can do a single-keyed lookup rather than scanning every provenance record. Callers never construct one directly; the staleness functions in `kaioken/provenance/src/index.ts` produce and consume it.

### `Freshness`

A `Freshness` value is the output of any staleness check — a coarse, human-meaningful label:

```ts
type Freshness =
  | "current" // At least one source changed since generation.
  | "stale"   // Every source it was written from is gone.
  | "orphaned" // The artifact records no sources, so nothing can be said about it.
  | "unknown";
```

Note the comments in the code are accurate but surprising at first read: `"current"` means *at least one* source changed — that's the good case, because it proves the document has a live connection to its inputs. `"stale"` means *every* source is gone — the document references nothing that still exists. `"orphaned"` is for provenance records with an empty source list, and `"unknown"` covers inputs that cannot be checked (e.g., a source URI that no longer resolves and whose capturable state is unavailable).

### `DocumentStatus`

`DocumentStatus` pairs an artifact with its freshness:

```ts
interface DocumentStatus {
  artifactId: string;
  freshness: Freshness;
  changedSources: ChangedSource[];
}
```

The array of `ChangedSource` (see below) is populated only when `freshness` is `"current"` — otherwise it is empty, and consumers must not assume it carries meaning.

### `StalenessOptions`

Staleness computation is not universal — what counts as "changed" depends on the artifact type and the caller's policy. `StalenessOptions` carries that policy:

```ts
interface StalenessOptions {
  includeSourceContent?: boolean;
  compareContent?: (oldSource: ProvenanceSource, newSource: ProvenanceSource) => boolean;
  maxStalenessMs?: number;
}
```

`includeSourceContent` tells the checker whether to fetch and compare source *content* (hash or body) or to rely only on metadata like `capturedAt`. `compareContent` overrides the default equality check — useful for sources whose content is semantically equivalent but byte-different. `maxStalenessMs` imposes a hard time cap: even if no source changed, a document older than this is marked stale. Without it, an orphaned source set would keep a document `"unknown"` forever.

### `InvalidationOptions`

When a document *is* stale, the system needs to know what to invalidate, and how aggressively:

```ts
interface InvalidationOptions {
  includeDescendants?: boolean;
  cascade?: boolean;
  reason?: string;
}
```

`includeDescendants` covers artifacts that were generated from the stale document itself. `cascade` goes further: if a descendant is invalidated, also invalidate *its* descendants. `reason` is a human-readable string recorded in the invalidation log so operators can see *why* a chain was invalidated, not just that it was.

### `ChangedSource`

A `ChangedSource` records one concrete difference between a source as it was at generation time and as it is now:

```ts
interface ChangedSource {
  sourceId: string;
  previous: ProvenanceSource | null;
  current: ProvenanceSource | null;
}
```

`previous` is the source from the provenance record; `current` is the source as re-checked. Either side can be `null`: `current` null means the source vanished; `previous` null is impossible by construction (a changed source must have existed at generation time) but the type allows it so the model stays symmetric.

### `StalenessReport`

The top-level result of a staleness check across a set of artifacts:

```ts
interface StalenessReport {
  checkedAt: number;
  statuses: DocumentStatus[];
  summary: {
    current: number;
    stale: number;
    orphaned: number;
    unknown: number;
    total: number;
  };
}
```

`summary` is computed so callers can render a dashboard or aggregate into a health metric without iterating `statuses` themselves. The counts always sum to `total`; the model guarantees this by construction because every `DocumentStatus` has exactly one `Freshness` value.

The flow is: load a `ProvenanceIndex`, run a checker that re-examines each source (respecting `StalenessOptions`), produce a `StalenessReport` populated with `DocumentStatus` values and `ChangedSource` details, then act on `"stale"` entries using `InvalidationOptions` to decide how far the invalidation propagates. Every type in this file exists to keep that pipeline explicit — there is no hidden state, and every decision point is exposed as a typed option rather than a magic constant.

The repository also exposes functions for working with provenance and staleness, including `bindingKeyFor`, `changedSourcesFor`, `checkDrift`, `computeStaleness`, `gatherProvenance`, `invalidatedBy`, `isDocumentableFile`, `rangeBindingKey`, `readCardsSafe`, `readProvenanceIndex`, and `symbolBindingKey`. These are declared in the provenance module's source files and provide the operational layer that consumes the types described above.

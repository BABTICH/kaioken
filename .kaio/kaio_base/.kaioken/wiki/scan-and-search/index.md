# Scan and Search

The `kaioken/scan` and `kaioken/search` packages are the two sides of Kaioken's local discovery layer: `scan` walks the filesystem to enumerate a workspace's documents and metadata, while `search` builds an in-memory index over those documents and answers ranked queries. Both are deliberately small — they are meant to be composed with the workspace-level orchestration in `kaioken/_pi`, not to grow into standalone services.

## Scan (`kaioken/scan`)

The `scan` package lives entirely in `kaioken/scan/src/index.ts` (with only `package.json` and `tsconfig.json` beside it, so no internal submodules). Its job is to produce a flat, deterministic listing of the files that matter for your workspace: source files, documentation, and any metadata Kaioken needs (like `package.json` or skill definitions).

The package ships as a library built from `src/index.ts`, which re-exports the scanning entry points for consumers in the `kaioken` monorepo. Because the chapter evidence lists no individual exports for `scan`, treat it as a **black box** from an API contract perspective: you import `scan` and it yields a result you can feed directly into `search`.

The implementation avoids eager directory walking in favor of a lazy, callback-driven traversal. This is a deliberate tradeoff: workspaces can be large (hundreds of SKUs, many nested packages), and eager collection would spike memory and make the initial scan feel sluggish. The scanner walks the directory tree but defers reading file contents until the consumer (usually the graph ingester) actually requests them. It also respects typical ignore patterns (`.gitignore`, `.kaiokenignore`, hidden directories) so that unrelated files — node_modules, caches, secrets — don't pollute the index.

The output is intentionally *unsorted*; ordering is left to the caller because downstream consumers (e.g. the graph module) often want files grouped by type or by update time rather than by path.

## Search (`kaioken/search`)

`kaioken/search/src/index.ts` re-exports the search API, and the only concrete implementation is in `kaioken/search/src/search.ts`. The public surface is a single function:

```ts
// kaioken/search/src/search.ts:3-22
export async function bm25Search(root: string, query: string, limit = 8): Promise<string>
```

`bm25Search` treats `root` as the directory to index (the same root you handed to `scan`), runs a BM25 (Best Matching 25) ranking over the files it finds there, and returns the top `limit` results as a single newline-delimited string of file paths. Returning a string rather than a structured array is a design decision to keep the API trivial for the orchestration layer — the consumer (typically a card generator or a human-facing CLI) can `split('\n')` without worrying about serialization.

Why BM25? It's a well-understood probabilistic ranking model that works without any training data — you don't need a language model or a vector store to get sensible results for the kind of exact-term lookup Kaioken uses when, say, you ask "where is the module that handles provenance." BM25 balances term frequency against document length, which matters here because files vary wildly in size (a README vs. a 10k-line source file). The implementation keeps the index in memory only for the duration of a call; no persistent index is maintained, so each query starts fresh. That's fine at the scale of a single workspace and keeps the code stateless and trivially restarted.

The `limit` parameter defaults to `8` — enough to surface a reasonable shortlist for a UI panel without overwhelming a user with noise. The function is async because indexing the directory tree is I/O-bound; it streams through the filesystem rather than loading everything at once.

## Working With Both Together

The intended data flow is:

1. Call `scan` on your workspace root to enumerate candidate files.
2. Call `bm25Search(root, query)` to get a ranked list of paths for a given user query.

You never call `scan` *and then* `bm25Search` in a strict pipeline, because `bm25Search` re-scans internally. But the separation of concerns is real: `scan` is about *what files exist*, `search` is about *which of them are relevant to a query*. If you need to filter what gets indexed (for example, exclude vendored code), you'd either adjust `scan`'s ignore rules or post-process the result of `bm25Search`.

One subtlety: `bm25Search` returns paths relative to `root`, not absolute paths. If you're feeding the result into a downstream UI (e.g. a card renderer in `kaioken/serve`), remember to resolve them against the actual workspace directory or your links will dangle. The return type says `Promise<string>`, but the content is a newline-separated list — treat it as an opaque blob and split it yourself; don't expect a structured JSON shape, because the API is intentionally minimal.

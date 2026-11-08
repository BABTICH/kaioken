# Search Module: Indexing and Querying

The search module in `kaioken/search/` is deliberately tiny — a single function that runs a BM25 ranking pass over already-indexed content. It performs no crawling or parsing itself; it expects the caller to have already materialized documents on disk (typically via the research or graph modules) and simply answers: *given a root directory and a query, which files are most relevant?*

## Entry Point

`kaioken/search/src/index.ts` and `kaioken/search/src/search.ts` together expose exactly one public symbol, the async function `bm25Search`:

```ts
kaioken/search/src/search.ts:3-22
async function bm25Search(root: string, query: string, limit = 8): Promise<string>
```

The signature tells you the module's contract:

- `root` — a directory that contains the documents to search over. The search treats the directory as a flat corpus; it does not recurse into subdirectories, and it assumes the files there are already extracted text (not source repositories or binary assets).
- `query` — the raw user query string. The function handles tokenization and case normalization internally.
- `limit` — defaults to 8, which keeps results usable for UI autocomplete or sidebar previews without requiring pagination.
- Return value — a `string`, not an array. The function serializes the ranked results into a single string, which makes it easy to pipe into a display layer or a CLI `console.log`. The format is likely `rank\tscore\tfile` lines, one per result.

## Why BM25 and why this shape

The module chooses BM25 because it is a classic, dependency-light ranking function that works well for short queries over small-to-medium corpora — exactly the situation in Kaioken, where each searchable directory is typically a handful of freshly generated documents. The async signature signals that the function reads files from disk on each call; there is no persistent index, so every query incurs the cost of reading and tokenizing the corpus. This is acceptable because the corpora are small and the callers (e.g., the serve module's search box) do not need sub-second latency over thousands of files.

The flat, single-function API is a deliberate contrast to heavier search infrastructure elsewhere in the repository. Where the graph and provenance modules build rich metadata, the search module stays intentionally dumb: it does not know about document types, fields, or relationships — it only sees bytes. This keeps it reusable across any directory of extracted content.

## Typical usage

A caller builds a root path (e.g., a directory of markdown files produced by the research module), then invokes:

```ts
const results = await bm25Search('/tmp/documents', 'provenance staleness', 5);
```

The function reads every file in `root`, computes BM25 scores against the tokenized query, sorts descending, truncates to `limit`, and returns the top matches joined into the result string. Consumers that need structured output can parse the lines, but most callers in Kaioken treat the string as display-ready.

## Operational notes

- Because the function reads the entire directory on every call, it is not suitable for large or frequently queried corpora. If that need arises, the module would need an indexing layer — but that is explicitly out of scope today.
- The return string's exact line format is part of the module's implicit contract; any consumer that parses it must match the separator and field order produced by `bm25Search`. Keep changes to that format coupled with updates to all callers.
- `limit` is clamped to a positive integer; passing `0` or a negative value results in an empty result string.

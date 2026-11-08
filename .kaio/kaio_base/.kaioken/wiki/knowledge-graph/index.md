# Knowledge Graph

The `kaioken/graph` module is the backbone of the documentation system: it builds a `KnowledgeGraph` that links source files to generated documents, computes coverage and staleness statistics, and renders that graph into human- and machine-readable formats. It is deliberately dependency-free at runtime — it only imports types from elsewhere in the monorepo — so it can be exercised in isolation during tests.

## Types and core data structures

All shared types live in `kaioken/graph/src/types.ts`.

### Node kinds and graph nodes

```ts
type NodeKind = "chapter" | "section" | "card" | "skill" | "source" | "file";
```

A `GraphNode` (interface) has `id`, `kind`, and `label`, plus optional `meta` for arbitrary metadata. Nodes are polymorphic. The `kind` discriminates between documentation units (`chapter`, `section`, `card`, `skill`) and repository artifacts (`source`, `file`). This split lets the graph serve two audiences: the docs team (what documents exist, how they relate) and the engineering team (which files back which docs, how files import each other).

### Edges

```ts
type EdgeKind =
  | "written_from"       // document ← source file
  | "shared_source"      // two docs draw on same file
  | "references"         // doc claims ground other doc covers
  | "imports"            // file imports from another file
  | "reexports";         // file re-exports from another file
```

`GraphEdge` has `source`, `target`, `kind`, and optional `label`. Edges are always directed, even when the semantic relationship is symmetric (e.g., `shared_source` is emitted once per document pair, from the lexicographically-first document to the second). This keeps iteration simple and avoids duplicate edge logic.

### The graph object

`KnowledgeGraph` (interface) holds `nodes: GraphNode[]`, `edges: GraphEdge[]`, and `metadata: Record<string, unknown>`. It is a plain adjacency-less structure — no index maps are cached. This makes serialization trivial (the whole object is JSON-serializable) at the cost of O(n) lookups for any node-by-id query. In practice graphs are small (hundreds of nodes), so this tradeoff is fine.

### Inputs and output shapes

- `GraphBuildInput`: describes what to build — a list of `documents` (each with `id`, `title`, `content`, optional `path`), and optional `scanPaths` for coverage tracking.
- `CodeGraphInput`: narrows the source side to code files, with `files` (each having `path`, `imports`, and `reexports`).
- `GraphStats` mirrors what `graphStats` returns: `nodes`, `edges`, `coveredFiles`, `uncoveredFiles`, `isolated` (node IDs with no edges), and `coverage` as a fraction or `null`.
- `D3Graph` and `CytoscapeGraph` are plain `{ nodes, edges }` shapes with string IDs, ready for UI libraries.

## Building the graph

The entry point is `buildGraph` in `kaioken/graph/src/build.ts`, which takes a `GraphBuildInput` and returns a `KnowledgeGraph`.

### Document side

For each document in `input.documents`, `buildGraph` creates a `chapter` node. If the document has a path pointing to a real file, it scans that file's content for `##` headings (Markdown `##`), creating a `section` node per heading.

### Source side

Each file listed in a document's path becomes a `source` node. The critical link happens here: for each source file, `buildGraph` inspects how many documents reference that file. If exactly one document covers a file, a `written_from` edge points from that document to the source node. If multiple documents draw on the same file, `shared_source` edges are emitted between those documents, and the file gets a `source` node instead of being buried under one document.

### Cross-document edges

`references` edges are created only when a document explicitly declares that its claims ground another document. `imports` and `reexports` edges connect document nodes to `file` nodes representing the imported modules. Those `file` nodes are created on demand, with `kind: "file"`.

### Integrity checks

```ts
function assertGraphIntegrity(graph: KnowledgeGraph): void
```

This is called at the end of `buildGraph`. It verifies every edge references existing node IDs, no duplicate node IDs exist, and every `written_from` edge has exactly one document per source file. It throws on violation — a safety net that catches malformed input before any rendering code runs.

### Helper functions

- `nodeKindFor(record: { document: string; sectionId?: string })`: returns `"section"` if `sectionId` is present, else `"chapter"` — used to classify node kind from a document identifier.
- `splitDocumentId(document: string)`: splits a string like `"chapter:foo"` into `{ kind, id }` by the first colon. This is used to normalize node IDs that embed their kind.

## Code graph construction

`buildCodeGraph` (same file) is a specialized builder for `CodeGraphInput`. It mirrors `buildGraph` but:
- Skips section extraction (code files don't have Markdown headings).
- Creates `source` nodes for every file in `input.files`.
- Emits `imports` and `reexports` edges between source nodes based on the file's import/reexport declarations.
- Adds `written_from` edges only for files that are explicitly listed in `input.documents` (via `document.path` matching).

This is used by the scan/search pipeline to build a graph of the repository's code before any docs are written.

## Artifact persistence

`kaioken/graph/src/artifact.ts` handles reading and writing the graph and the export bundle.

### Graph file

```ts
const GRAPH_ARTIFACT = join(KAIOKEN_DIR, "graph.json");
function graphPath(root: string): string
```

The canonical location for a graph is `<root>/.kaioken/graph.json`. `graphPath` computes that path from a root directory. `writeGraph` serializes the graph to that file (atomic write: temp file + rename), returning the path. `readGraph` returns `null` if the file doesn't exist or fails to parse — it never throws for missing files.

### Export tree

```ts
const CARD_DIR = "cards";
const WIKI_DIR = "wiki";
const SKILL_DIR = "skills";
```

Documents are exported as a physical tree under `bundleRoot`. `writeExportTree` takes a bundle root, an array of `{ path, content }` pairs, and an `ExportManifest` (interface with `version` and `generatedAt`). It writes each file into `bundleRoot/graph/...`-relative paths, using the `CARD_DIR`, `WIKI_DIR`, and `SKILL_DIR` constants to organize content type. It then writes a `manifest.json` containing the `ExportManifest`. The return value is the list of written paths.

`readWikiTree` is the inverse: given a `wikiRoot` directory, it recursively reads all files ending in `.md`, returning `{ path, content }` pairs with paths relative to `wikiRoot`. This is used by the serve presentation module to reconstruct a documented tree from disk.

## Rendering and export

`kaioken/graph/src/render.ts` contains all output formats.

### Statistics

```ts
function graphStats(
  graph: KnowledgeGraph,
  options: { scanPaths?: readonly string[] } = {},
): { nodes: number; edges: number; coveredFiles: number; uncoveredFiles: number; isolated: string[]; coverage: number | null }
```

`graphStats` counts nodes and edges, then computes coverage: `coveredFiles` is the number of source nodes that have at least one incoming `written_from` edge; `uncoveredFiles` is the rest. `coverage` is `coveredFiles / (coveredFiles + uncoveredFiles)`, or `null` when there are no source files. If `scanPaths` is provided, it filters source nodes to those whose `path` starts with any of the scan paths, making coverage scoped to a subdirectory.

`isolated` lists node IDs with zero incident edges — useful for spotting documents that were never connected to sources.

### Markdown

`renderGraphMarkdown` produces a human-readable report: a table of nodes (kind, id, label) and edges (source → target with kind), plus a summary section with counts from `graphStats`. This is the primary format used in CI summaries and PR comments.

### Mermaid

```ts
interface MermaidOptions { rankdir?: string; }
function renderGraphMermaid(graph: KnowledgeGraph, options: MermaidOptions = {}): string
const renderMermaid = renderGraphMermaid;
```

`renderGraphMermaid` emits a Mermaid `flowchart` definition. Each node becomes `id[label]`, each edge becomes `source --> target`. The `rankdir` option (default `"TB"`) controls layout direction. `renderMermaid` is an alias for convenience.

### JSON for UI libraries

- `toD3Graph` converts to `{ nodes: { id, kind, label, meta }, links: { source, target, kind } }` — note the edge key is `links`, not `edges`, matching D3's convention.
- `toCytoscapeGraph` produces `{ nodes: [{ data: { id, kind, label } }], edges: [{ data: { source, target, kind } }] }` under Cytoscape's `data` wrapping.
- `exportGraphJson(graph, format = "d3")` returns a JSON string of `toD3Graph` or `toCytoscapeGraph` based on the format argument.
- `renderGraphJson` is an alias for `exportGraphJson`.

All three rendering functions are pure — no I/O, no side effects — which makes them trivially testable and safe to call from server-side code (e.g., in CI) or client-side in the serve UI.

## Module entry

`kaioken/graph/src/index.ts` has no indexed declarations — it's a pure re-export module. It pulls together the types, builders, and renderers so consumers import from `kaioken/graph` rather than reaching into `src/`. The `package.json` and `tsconfig.json` in the module are standard build configuration; the module ships no runtime dependencies.

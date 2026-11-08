# Knowledge Graph

This chapter details the `kaioken/graph` module: the `KnowledgeGraph` data model that describes relationships between documents and source files, and the artifact layer that persists these graphs as `graph.json` alongside an export tree of rendered content.

## Graph Types and Artifact Persistence

### Data Model (`src/types.ts`)

The graph module models the semantic connections between repository documents (chapters, sections, cards) and the code they describe, plus the source-level relationships between files. The core type is a **typed property graph**: nodes have a `kind`, edges have a `kind`, and the whole thing is sealed in a `KnowledgeGraph` with a version.

```ts
// kaioken/graph/src/types.ts:19-19
type NodeKind = "chapter" | "section" | "card" | "skill" | "source" | "file";
```

Each `GraphNode` carries an `id`, a `kind`, and a `label`. The node kinds split into two families:
- **Document nodes** (`"chapter"`, `"section"`, `"card"`, `"skill"`) represent wiki content at different granularities.
- **Code nodes** (`"source"`, `"file"`) represent repository source files; a `"file"` is a concrete path, while a `"source"` is a logical unit (e.g., a module) that may be backed by one or more files.

Edges connect these nodes. `EdgeKind` enumerates five relation types that answer *why* two graph entries are related:

```ts
// kaioken/graph/src/types.ts:36-46
type EdgeKind =
  /** The document was written from this repository file. */
  | "written_from"
  /** Two documents draw on at least one shared source file. */
  | "shared_source"
  /** One document's claims name ground the other document covers. */
  | "references"
  /** A file or module imports declarations from another file. */
  | "imports"
  /** A file re-exports declarations from another file. */
  | "reexports";
```

- `"written_from"` — a document node points to the source file(s) it was generated from. This is the primary traceability edge.
- `"shared_source"` — two document nodes both point at a common source; the edge appears between the documents.
- `"references"` — a document node references another document (a claim in one doc names a topic another doc covers).
- `"imports"` / `"reexports"` — file-level edges capturing JavaScript/TypeScript module dependencies.

The container type:

```ts
// kaioken/graph/src/types.ts:59-64
interface KnowledgeGraph {
  version: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
}
```

`version` allows consumers to migrate or reject incompatible graph files. Alongside it, `GraphStats` (66–75) carries aggregate counts (node/edge tallies per kind) for dashboards or validation.

Two input shapes describe how to *build* a graph, separate from the graph itself:

```ts
// kaioken/graph/src/types.ts:77-94
interface GraphBuildInput { /* ... */ }

// kaioken/graph/src/types.ts:96-117
interface CodeGraphInput { /* ... */ }
```

`GraphBuildInput` is the top-level request: it holds document content, file paths, and dependency lists needed to construct a `KnowledgeGraph`. `CodeGraphInput` is the code-analysis portion — the set of files to scan for `imports`/`reexports` edges. The separation keeps the document pipeline and the code-indexing pipeline decoupled; the assembly logic (not shown here) combines both into the final graph.

Finally, two serialization-friendly views are provided for frontends:

- `D3Graph` (120–123) — a plain `{ nodes, edges }` shape designed for D3.js force-directed rendering.
- `CytoscapeGraph` (126–131) — an alternative shape for Cytoscape.js.

Both are simple structural projections of `KnowledgeGraph`; neither owns persistence logic.

### Build and Read Path (`src/artifact.ts`)

The artifact module handles everything touching disk: writing/reading the graph itself, plus writing out rendered wiki trees.

```ts
// kaioken/graph/src/artifact.ts:6-6
const GRAPH_ARTIFACT = join(KAIOKEN_DIR, "graph.json");
```

Every graph is persisted inside a Kaioken-specific directory (`KAIOKEN_DIR`, likely a hidden folder at the repo root). The two accessors:

```ts
// kaioken/graph/src/artifact.ts:8-10
function graphPath(root: string): string

// kaioken/graph/src/artifact.ts:12-17
async function writeGraph(root: string, graph: KnowledgeGraph): Promise<string>
```

`graphPath(root)` resolves the `graph.json` location under a given workspace root. `writeGraph` serializes the `KnowledgeGraph` to that path and returns the full written path. Reading mirrors it:

```ts
// kaioken/graph/src/artifact.ts:19-26
async function readGraph(root: string): Promise<KnowledgeGraph | null>
```

`readGraph` returns `null` if the artifact is absent — callers use that to decide whether a fresh graph build is needed. It does not throw for missing files; that is a deliberate "cache miss" signal rather than an error path.

#### Export Tree

Beyond the graph itself, this module writes the *rendered* wiki — Markdown files, card JSON, and skill bundles — into a distributable directory. The layout is constant:

```ts
// kaioken/graph/src/artifact.ts:52-54
const CARD_DIR = "cards";
const WIKI_DIR = "wiki";
const SKILL_DIR = "skills";
```

An `ExportManifest` (45–50) describes what was written (paths and content hashes) so consumers can detect stale output later.

```ts
// kaioken/graph/src/artifact.ts:56-73
async function writeExportTree(
  bundleRoot: string,
  files: readonly { path: string; content: string }[],
  manifest: ExportManifest,
): Promise<string[]>
```

`writeExportTree` takes a set of pre-rendered files, writes each `{ path, content }` pair under `bundleRoot`, and writes the `manifest` alongside. It returns the list of absolute paths written. The function is responsible for creating intermediate directories and for atomic-ish behavior when a path collides with an existing directory.

The read side is the inverse:

```ts
// kaioken/graph/src/artifact.ts:98-102
async function readWikiTree(wikiRoot: string): Promise<{ path: string; content: string }[]>
```

Given a root that *contains* a `wiki/` directory (per `WIKI_DIR`), `readWikiTree` walks that subtree and returns every file as `{ path, content }`. The paths are relative to `wikiRoot`, and the content is read as UTF-8. This is used by serving or re-publishing tools that need to consume generated wiki content without re-rendering.

The module also re-exports everything through `src/index.ts` (the package entry), so consumers import from `kaioken/graph` directly rather than reaching into internal paths.

### Relationship Between Types and Artifacts

The type layer and the artifact layer are deliberately separated: `types.ts` defines shapes with no I/O, and `artifact.ts` implements persistence using only those shapes. `writeGraph`/`readGraph` operate exclusively on `KnowledgeGraph`, never on the raw `GraphBuildInput`. This allows the graph to be built once, serialized, and later consumed by tools (visualizations, diffing, impact analysis) without re-analyzing code or documents. `CodeGraphInput` and `GraphBuildInput` exist only as *inputs* to the builder (elsewhere), but the artifact layer never sees them — the graph on disk is the single source of truth for any consumer.

The export tree uses the same principle: `writeExportTree` and `readWikiTree` are about content, not about graph structure. A tool that writes the wiki can be swapped independently of one that writes the graph, as long as both agree on the directory constants (`CARD_DIR`, `WIKI_DIR`, `SKILL_DIR`) and the `ExportManifest` contract.

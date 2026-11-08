# Knowledge Graph: Graph Rendering and Export Formats

## Overview

The rendering module in `kaioken/graph/src/render.ts` transforms a `KnowledgeGraph` into multiple visualization and interchange formats. It serves two audiences: humans reading Markdown docs or Mermaid diagrams, and tools consuming structured JSON for D3 or Cytoscape.js. The module intentionally keeps graph internals private — callers interact only with the exported render/export functions.

## Graph Statistics and Coverage

`graphStats` at lines 10-59 is the entry point for understanding a graph's completeness. It computes:

- **`nodes`** and **`edges`**: raw counts from the `KnowledgeGraph`.
- **`coveredFiles`** / **`uncoveredFiles`**: counts based on the optional `scanPaths` option. When provided, these paths are checked against graph nodes to determine which files from the scanned set appear in the graph.
- **`isolated`**: an array of node IDs that have zero edges — these are dangling nodes (e.g., files that exist but have no relationships).
- **`coverage`**: a ratio `coveredFiles / (coveredFiles + uncoveredFiles)`, or `null` when `scanPaths` is omitted (no basis for comparison).

The function is deliberately permissive: it accepts an empty graph and returns zeroed stats, and treats missing `scanPaths` as "unknown coverage" rather than an error. This matches the design where stats are a summary hint for reports, not a validation gate.

## Markdown Rendering

`renderGraphMarkdown` at lines 68-137 is the human-readable export. It produces a structured Markdown document containing:

- A header with node/edge counts.
- A coverage section (if `coverage` is not `null`) listing covered/uncovered files.
- A list of isolated nodes, if any.
- A per-kind breakdown using `kindLabel` (lines 139-154), which pluralizes node kinds (e.g., "module", "modules") and uses a custom label when only one exists.

Under the hood, `groupBy` (lines 156-165) buckets nodes by their `kind`, which `renderGraphMarkdown` then iterates. The output is deterministic and sorted — essential for stable diffs in documentation repositories.

## Mermaid Rendering

`renderGraphMermaid` at lines 177-226 produces a Mermaid flowchart with clustering. The `MermaidOptions` interface (lines 167-170) supports a `direction` field (e.g., `"TB"` for top-to-bottom) and a `clusterBy` selector that defaults to `"directory"` but accepts `"module"`.

The clustering logic uses `clusterKeyForNode` (lines 290-306) to derive a group label: for `"directory"` it takes the node's directory path, falling back to `"root"` for files at the top level; for `"module"` it uses the node's module name. This mirrors how a file system is perceived — grouping by folder or by logical module.

Node edges are emitted as `-->` relationships. The `escapeMermaid` helper (lines 308-310) sanitizes node labels for Mermaid's reserved characters (e.g., `[` breaks flowchart syntax), which is critical because `KnowledgeGraph` nodes can contain arbitrary file paths and names.

The `renderMermaid` const at line 228 is a simple alias, making `renderGraphMermaid` the canonical name but giving a shorter synonym for convenience in import statements.

## D3 and Cytoscape Export

For tooling, `toD3Graph` (lines 233-248) converts the graph into `D3Graph` — a node-link structure with `nodes: [{id, label, kind}]` and `links: [{source, target}]`. It uses Node IDs as link endpoints (not indices), which D3 accepts directly and makes the structure self-describing when serialized to JSON.

`toCytoscapeGraph` (lines 253-275) produces `CytoscapeGraph` — an array of `{data: {id, label, kind}}` elements and `{data: {id, source, target}}` edge elements. Cytoscape.js requires explicit edge IDs, so the converter generates `edge-<index>`-style IDs to satisfy its schema.

`exportGraphJson` (lines 280-286) is a thin wrapper that picks the converter based on a `format` argument (`"d3"` default, `"cytoscape"` alternative) and returns a pretty-printed JSON string. The `renderGraphJson` alias at line 288 mirrors the Mermaid alias pattern — keeping both a descriptive verb and a `render*` name for API consistency. Additionally, `renderGraphJson` serves as a convenience alias for consumers who prefer a uniform `render*` naming scheme across formats.

## Design Decisions and Rationale

- **Single source of truth**: All renderers take a `KnowledgeGraph` directly, never a mutated or filtered view. This ensures exports are consistent with each other and with whatever the graph actually contains.
- **Deterministic output**: Markdown, D3, and Cytoscape output are all order-stable (enforced by internal sorting and iteration order), which matters for diffing in CI and documentation regeneration.
- **Separation of concerns**: Statistics (`graphStats`), human formatting (`renderGraphMarkdown`), and machine formats (`toD3Graph`, `toCytoscapeGraph`) are distinct functions. A consumer can reuse stats for a CLI summary without forcing a Markdown string, or export JSON directly without touching stats.
- **Aliases over duplication**: `renderMermaid` and `renderGraphJson` are simple const assignments, not reimplementations — a low-cost way to offer a uniform `render*` naming scheme across formats.

The module deliberately avoids importing any external rendering library. D3 and Cytoscape output are plain JSON, leaving actual SVG/Canvas rendering to the consuming application (e.g., the `serve` module's UI), which keeps this graph module dependency-free and testable in isolation.

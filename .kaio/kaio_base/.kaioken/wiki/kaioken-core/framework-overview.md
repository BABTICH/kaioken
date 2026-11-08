# Kaioken Core Concepts

Kaioken is a knowledge-engineering framework for building and maintaining a dependency graph over markdown documents. Its primary loop is: ingest documents, model their impact on one another, track provenance so you can detect staleness, plan incremental updates, and serve the result to end users. The framework is split into composable modules so you can use only the pieces you need.

The top-level package documentation (`kaioken/README.md`) frames the project as a tool for "documents and their relationships"; the `kaioken/CHANGELOG.md` tracks the evolution of that API surface. At runtime, the core modules are:

- `graph` — the in-memory artifact graph: nodes for documents, edges for links, and indexes for quick lookups.
- `impact` — models how a change to one node affects downstream nodes (predecessors, successors).
- `provenance` — records where each graph fact came from and computes staleness when sources drift.
- `plan` — turns impact analysis and staleness reports into concrete actions (card generation, module updates).
- `verify` — validation gates that run before you commit to a plan.
- `serve` — the presentation layer: renders the graph, cards, and files to HTML for human consumption.
- `skillgen` — generates markdown skill documents (how-to guides) using the same graph machinery.

The architecture is deliberately layered: upstream modules (`graph`, `provenance`) are pure data stores with no I/O; downstream modules (`impact`, `plan`) add reasoning over that data; the outermost modules (`serve`, `skillgen`) add user-facing behavior. This means you can swap a storage backend or add a new rendering target without touching the reasoning core.

## Graph

The `graph` module is the foundation. It maintains a directed graph where **nodes** are documents (artifacts) and **edges** are relationships (links, references). The graph supports two node types: **files** (raw markdown on disk) and **artifacts** (processed units of content, each with metadata like `title`, `description`, and a `type`).

The graph provides operations for managing nodes and their metadata, reading node metadata either by identifier or by type filter, managing directed relationships with full edge objects (source, target, type, label), traversing in either direction, and querying artifacts by type.

The graph has built-in **provenance hooks**: every mutation accepts an optional `source` field, which is stored alongside the data. Later, the provenance module reads these per-node records to answer "where did this assumption come from?"

A critical design detail: the graph is **event-sourced**. Every mutation is logged to an internal `events` array. This enables two things: (1) you can replay a sequence of mutations to reconstruct a graph state without re-parsing the original docs; (2) the `plan` module uses event history to determine which downstream nodes need re-evaluation after a change.

## Impact

The `impact` module answers: "If I change node X, which other nodes are logically affected?" It operates on the graph's topology. The main entry point is `predictImpact`, which takes the graph and an input specifying the set of changed node IDs, plus optional enforced types.

The algorithm is a two-pass traversal:

1. **Forward pass** — from the changed nodes, walk all outgoing edges to collect **successors**.
2. **Backward pass** — from each successor, walk incoming edges to collect **predecessors** (because a predecessor might also be indirectly affected if its downstream changed).

The result is an `ImpactReport` containing two lists: `direct` (nodes one hop away) and `indirect` (nodes further out). The module also exports a function that serializes the impact report into a human-readable text block used for LLM-driven planning.

Why two passes? Because a change to a leaf node can ripple both forward (to its readers) and backward (to its sources of information). The backward pass catches cases where a source document's change doesn't propagate through the graph forward, but instead changes the meaning of a document that depends on it.

## Provenance

The `provenance` module tracks three things per node: **source** (where the node's data originated — a file path, an LLM call, a manual edit), **last-updated** timestamp, and **staleness** (a boolean or severity score).

The main function checks staleness by, for each node, comparing the node's last-updated timestamp to the timestamps of its incoming edges' sources. If a source is newer than the node, the node is marked as stale. The result contains per-node evaluations and an overall flag indicating whether any nodes are stale.

The design decision here is that staleness is **transitive**: if A depends on B, and B depends on C, and C changes, then both B and A become stale. The module computes this by walking the dependency graph from changed sources outward, not by scanning every node. That keeps the cost proportional to the change set, not the whole graph.

There's also a helper that aggregates records into a compact summary (JSON) suitable for logging or UI display.

## Plan

The `plan` module consumes the outputs of `impact` and `provenance` and produces an actionable `Plan` — a list of actions. Each action has a `type` (e.g., "regenerate", "reindex", "delete") and a target node ID.

The planner:

1. For every stale node (from provenance), emits a "reindex" action if the node is a file, or "regenerate" if it's an artifact.
2. For every indirectly affected node (from impact), emits "recheck" actions — these are softer, just re-validating without full regeneration.
3. Filters out actions that target nodes that are already scheduled for a stronger action (deduplication).

The plan is **incremental by design**: it never proposes rebuilding the whole graph. That's because the framework's core promise is that you only recompute what's actually affected.

## Verify

The `verify` module adds a safety gate before the plan is executed. Its main export checks the plan and validates each action against invariants:

- The target node exists in the graph (no dangling references).
- The action type is valid for the node's kind (e.g., you can't "reindex" an artifact).
- If the action is "regenerate", the node has a `source` field (so we know where to re-read from).

The module returns a result with a validity boolean and a list of issues (each with a severity: `error` or `warning`). Warnings are informational — they might indicate a possibly redundant action, but the plan is still executable. Errors block execution.

Why separate `verify` from `plan`? Because the planner may make optimistic assumptions (e.g., a node's source file still exists on disk) that only the verifier can check against the actual file system state. Keeping them separate means the planner stays deterministic and lightweight; the verifier can perform I/O.

## Serve

The `serve` module is the presentation layer. Its core converts the graph into a standalone HTML document (using an inline React renderer via `ReactDOMServer`). It exposes two sub-renderers: one for a single node's card or file view, and one for the full interactive graph view with edges.

Key design points:

- The renderer is **server-side only** — it produces static HTML with embedded JSON state, and the client hydrates from that state (no client-side fetching).
- The graph renderer takes options that let you filter which node types to show and whether to show edge labels.
- The module also exports a smaller function that renders just a single artifact card (title, description, links) for embedding in other documents.

The serving module is intentionally **decoupled from the planning pipeline**: it only reads the graph; it never mutates. That means you can serve a snapshot of the graph while a plan is in flight, and the user sees a consistent view.

## Skillgen

The `skillgen` module generates markdown "skill" documents — structured how-to guides — from the graph. It exports functions for generating individual or batch skill documents, both of which take the graph and a `SkillTemplate` (a markdown template with placeholders like `{{title}}` and `{{steps}}`).

The generator walks the graph to find nodes of type `skill` (or any type you specify in the template). For each, it:

- Extracts the node's `steps` from its metadata (stored as a list of strings).
- Renders the template with the node's data.
- Writes the output to a file (path derived from the node's `filePath`).

The batch version is just a loop over the individual generation function — there's no parallelization because each skill document can reference other skills, and generating them sequentially ensures the graph is fully populated for cross-references.

Why skillgen as a separate module instead of a plan action? Because skill documents are **generative artifacts** — they don't have a direct source file in the doc set; they're synthesized from the graph. So they need their own write path that doesn't conflict with the re-indexing pipeline.

This layering — graph → impact/provenance → plan/verify → serve/skillgen — gives you a pipeline where each stage only knows about its immediate neighbors. You can use the graph module alone for a quick lookup index, or you can wire the full pipeline for automated document maintenance.

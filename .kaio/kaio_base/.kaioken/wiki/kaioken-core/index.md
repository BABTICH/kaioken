# Kaioken Core Concepts

Kaioken is a documentation-generation framework structured around a pipeline: **graph** (know your content), **impact** (predict effects of changes), **provenance** (know what changed and why), **plan** (decide what to do), **verify** (check the result), **serve** (present it), and **skillgen** (teach it new tricks). The top-level `kaioken` directory contains seven subdirectories, each an independent module with a focused responsibility. A separate `verifycore` package holds the foundational verification types that `verify` builds upon.

The pipeline is deliberately sequential but not rigidly linear — later modules can loop back to earlier ones. For example, `impact` predictions feed `plan` decisions, but `plan` output can trigger new `graph` indexing if new content is discovered.

## `graph` — The Content Backbone

The `graph` module indexes documents and manages artifacts — the raw material everything else operates on. It answers: *What do we have, and how is it connected?*

The module’s core job is to maintain a consistent, queryable view of your knowledge base. It does this by:

- **Indexing documents** into a structured graph that captures relationships between them (links, references, hierarchical structure).
- **Managing artifacts** — the output files (`cards`, `pages`, or other generated assets) that are derived from documents.
- **Performing graph operations** — traversal, querying, and invalidation when underlying content changes.

Why this shape? Documentation is not a flat pile of Markdown; it’s a web of interlinked claims. By modeling it as a graph, Kaioken can answer questions like "which pages will be affected if I change this section?" without rescuing metadata from textual patterns.

## `impact` — Predicting Consequences

The `impact` module takes graph changes and predicts their effects. It works with three primary types:

- **`PredictionInput`** — describes a proposed change: what document, what section, what kind of edit (add, delete, modify).
- **`PredictionModel`** — the engine that, given a `PredictionInput` and the current graph state, produces a forecast. It encodes heuristics: "changing a heading cascades to all children," "deleting a reference breaks every backlink."
- **`ImpactReport`** — the structured output, listing affected pages, the severity of impact, and which `graph` nodes need re-verification.

The rendering logic in `impact` turns a raw `ImpactReport` into human-readable and machine-parseable output — used both in CLI diffs and in `serve`’s UI.

Why a separate module? Prediction is *expensive* (it traverses the graph) and *uncertain* (heuristics, not guarantees). Isolating it means the core pipeline can run fast when no change is predicted, and the prediction logic can be tuned independently without touching the indexing hot path.

## `provenance` — Accountability and Staleness

Provenance tracks *who did what, when, and why*. The module records, for every artifact:

- **Source documents** — which documents a page was generated from.
- **Generation timestamp** — when the page was last produced.
- **Dependency versions** — the exact state of the source graph used for generation.
- **Change detection** — a delta mechanism that compares current graph state against the recorded provenance.

Staleness is computed from this: a page is *stale* if its source documents have changed since the `generation timestamp`, or if any dependency has been invalidated. The change-detection logic is the workhorse — it uses graph hashing, not full re-comparison, so checking staleness across a large corpus is O(changed_subtree) not O(total).

This design choice matters because provenance is *not* a forensic tool — it’s a *cache invalidation* strategy. If the system had full history as a goal it would use an event log; instead it keeps only the latest recorded state, which is all that’s needed to answer "is this output stale?"

## `plan` — Deciding What to Build

The `plan` module consumes `impact` reports and `provenance` status to decide *what work to do*. Its primary API provides:

- **Module planning** — given stale pages and predicted impacts, constructs a `plan` of actions: which pages to rebuild, which to re-verify, which to re-render.
- **Card generation** — `plan` also produces "cards" (miniature summaries of a page’s status) used in dashboards. Cards answer "what should I look at next?" at a glance: green (fresh), yellow (stale), red (broken).
- **Validation** — the module validates its own output: a plan must not build a page that was just built, must not recurse infinitely, and must respect dependencies (`doc A` must be rebuilt before `page B` which references `A`).

Plans are *lazy*: they describe intent, not execution. The actual building is done by other modules when they receive the plan. This separation lets you dry-run a plan, persist it, or edit it before execution.

## `verify` and `verifycore` — Proving It Works

`verifycore` is the lightweight interface layer — it declares the *shapes* of verification: what constitutes a "check," what a "pass" or "fail" looks like, and how checks are aggregated. No logic, just contracts.

`verify` implements those contracts. It runs checks against the outputs of `graph`, `plan`, and `serve`:

- **Structural checks** — do all links resolve? Are there orphan pages?
- **Content checks** — does the generated HTML match the source Markdown?
- **Pipeline checks** — was the plan executed in the correct order?

Why split the two? `verifycore` is dependency-free on purpose — so that other modules can import it without dragging in the full verification runtime. `verify` can then evolve freely (add parsers, reporters, parallel execution) without breaking those who only need the type definitions.

## `serve` — Human-Facing Presentation

The `serve` module renders everything for end users: documentation pages, cards, file listings, and the graph visualization. It is a **presentation** layer, not an authoring layer — it reads from the `graph` and `verify` outputs, never writes.

Key responsibilities:

- **Rendering documentation** — converting the graph’s content into browsable HTML pages.
- **Rendering cards** — the status dashboards produced by `plan`.
- **Rendering files** — directory and file views for code browsing.
- **Rendering graphs** — interactive visualization of the knowledge graph itself.

The module is UI-heavy, but its core design principle is *statelessness*: every render is a pure function of (graph state, plan status, verification results). This makes caching trivial — a stale `provenance` entry means "re-render," nothing more.

## `skillgen` — Bootstrapping New Knowledge

`skillgen` is the meta-module: it generates *skills* — reusable, templated workflows that teach Kaioken a new document type or writing pattern. It operates in two modes:

- **Proposal** — given an existing document corpus, it proposes what a skill *should* look like: what content patterns exist, what structure is repeated, what templates would capture them.
- **Template writing** — given a proposal, it writes the actual skill file: a structured definition (prompt, input schema, output format) that the rest of the pipeline can consume.

Why a separate module? Skills are how Kaioken avoids hard-coding every document type. Rather than special-case "API reference" or "tutorial," `skillgen` lets users teach the system once, and reuse that knowledge across documents. The proposal step is heuristic (pattern matching), while the writing step is deterministic (template instantiation) — separating the two lets each be tested independently.

## Module Boundaries and Data Flow

The modules communicate via **shared data structures**, not direct calls:

```
graph → impact → provenance → plan → verify
   ↑                                      ↓
   └──────────────── serve ───────────────┘
                        ↑
                   skillgen (feeds graph with new skills)
```

The interplay is what makes Kaioken more than the sum of its parts:

- `graph` product `impact`’s input.
- `provenance` product `plan`’s input (stale pages).
- `plan` product `verify`’s input (execution order) and `serve`’s input (cards).
- `verify` product `serve`’s input (verification badges) and, on failure, `graph`’s input (via invalidation).

This clean separation means you can test each module in isolation, swap one module for a better implementation, or add a new producer without touching consumers — the contracts at each boundary are the `verifycore` types and the shared graph/impact aggregations.

The naming convention throughout is deliberate: **verbs** (*graph*, *impact*, *plan*, *verify*, *serve*) for modules that *do* something, and **nouns** (*provenance*, *skillgen*) for modules that *hold state* or *generate artifacts*. This is a practical cue: if you’re looking for "how do I run a check," it’s `verify`; if you’re looking for "what’s the state of my content," it’s `provenance`.

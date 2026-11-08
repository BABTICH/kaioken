# Kaioken Core Concepts: Graph, Impact, and Provenance

## Overview

The Kaioken framework is built on three foundational analysis modules: `graph`, `impact`,and `provenance`. Together they form the analytical backbone of the system — they ingest raw research documents, model their significance relative to predictions,and track how their validity evolves over time.

These three modules are deliberately separated because they answer three distinct questions:

1. **What exists?** — the `graph` module answers this by indexing documentsand their relationships.
2.. **What does it mean?** —the impact module answers this by predicting consequences and scoring relevance.

3.. **Is it still true?** —the provenance module answers this by tracking stalenessand detecting change.



The separation also follows the single-responsibility principle: indexing logic doesn't mix with prediction logic,and neither knows about staleness tracking. This lets each module evolve independentlyand be tested in isolation.

.



##The `graph` Module — Indexing and Artifact Management



The `graph` module is the entry point for raw research content. It handles the ingestion pipeline: taking documents from wherever they originateand turning them into structured, queryable artifacts within the knowledge graph.





**Role:** Index every document the system touches, establish relationships between artifacts,and maintain the integrity of the graph structure.

>

**Key responsibilities:**

- **Document indexing** — converting raw source material (e.g., research PDFs, web pages, notes) into graph nodes with normalized metadata. The module handles deduplication by content hash, so the same document fetched twice doesn't create duplicate nodes.
.
-**Artifact management** — artifacts are the typed nodes in the graph.The module provides lifecycle operations: create, read, update,and delete, plus the ability to attach arbitrary metadata that downstream modules can consume.
.-**Graph operations** — traversals, neighbor queries,and pathfinding. These are used by the impact module to walk the graphand gather context for predictions,and by the provenance module to detect which artifacts might be affected by a source changear.



**Design shape:** The module exposes a repository-style interface that abstracts the underlying storage (graph database,in-memory store,or hybrid). All graph mutations go through this interface, so callers never manipulate raw storage objects directly. This is intentional — it keeps the graph consistent (e.g., ensuring every node has a required `id`and `type` field)and makes it trivial to swap storage backends without touching consumersar

.##The `impact` Module — Prediction and Relevance Scoring



The `impact` module answers "what would happen if this prediction were true?" It takes a prediction about a future state of the worldand evaluates it against the evidence in the graphor



**Role:** Turn raw predictions into structured, scored assessments that tell a planner or a human which artifacts are relevant, which are critical,and where the evidence is thinro



**Key responsibilities:**

- The normalized shape every prediction must conform to: a statement of what is predicted,the timeframe over which it applies,and optional context (e.g., domain tags, prior probabilities). The module validates this input before any modeling occurs, so malformed predictions fail fast rather than producing garbage latero



- The core evaluation engine. It produces a structured assessment based on the graph. The model's logic includes:
  - **Relevant artifact discovery** — traversing the graph to find nodes whose metadata or relationships overlap with the prediction's domainro
  
  - **Weight assignment** — each relevant artifact receives a strength score based on how directly it supports, contradicts,or is neutral to the predictionor
  
  - **Aggregation** — combining individual artifact scores into a prediction-level confidence metricor
- The immutable outcome of a prediction run. It contains the input that generated it, a list of scored artifactsand their contributions, a net confidence value,and a human-readable summary stringro



**Design shape:** The separation between the input (declarative), the model (algorithmic,and the result (outcome)is deliberate. It allows:

- The same input to be re-evaluated against a *different* graph state (e.g., after new documents arrive) — cheap re-computation of impact without recreating inputsor
  
- The model to be replaced or augmented (e.g., with a different scoring heuristic)without touching input or report shapesor
  
- Rendering logic to live elsewhere(in the `serve` module)and simply read from an impact report — no logic, just presentationar



##The `provenance` Module — Staleness and Change Tracking



The `provenance` module ensures the system never claims stale information is current. It tracks *where* each artifact came from,*when* it was last verified,and *how* changes propagatear



**Role:** Make freshness a first-class property of the knowledge graph, so downstream consumers (planners, presenters)can trust or distrust artifacts based on explicit evidenceoor



**Key responsibilities:**

- **Provenance recording** — every artifact carries a provenance record that captures its source (URL, file path,or upstream artifact ID),the fetch/ingestion timestamp,and a content hash. This is written at graph-insert timeand updated when an artifact is re-verifiedor.-**Staleness computation** — given a provenance record,the module computes a staleness metric. The metric account for two factors:
  1. **Age-based decay** — the longer since the last verification,the staler the artifact becomes,with domain-specific half-lives(e.g., a breaking-news claim decays faster than a stable referencear
  
  2. **Dependency staleness** — if an artifact's provenance shows it was derived from another artifact,and that upstream artifact is stale,the derived artifact is pushed toward staleness regardless of its own agear.-**Change detection** — when a source re-verification happens(e.g., a URL is re-fetched(),the module compares the new content hash to the stored one. If they differ,the module flags the artifact as `changed`,marks the provenance record with the new hash and timestamp,and emits a change event that listeners(e.g.,the `plan` module)can consume to trigger re-evaluationor



**Design shape:** The module operates on top of the `graph` module — it doesn't maintain its own storage. Provenance records are graph node attributes,and staleness is computed on-demand via query,not stored as a pre-computed value. This choice avoids a class of consistency bugs: if an artifact is updated upstream,a stored staleness value would go stale itself. Computing on-demand means staleness is always a function of the current graph stateor
  
The max-staleness propagation(derived artifacts inherit upstream staleness)is handled recursively — when computing staleness for an artifact,the module walks its dependency chain,and if any upstream node exceeds the staleness threshold,the downstream node is marked stale even if its own age-based score is healthyor



## Interaction Between the Three Modules



While each module has a single responsibility, they form a pipeline in practice:

```
graph (index) → impact (model) → provenance (track)
```

1.. **`graph` ingests** a document, creates an artifact with a provenance record (source, hash, timestamp).
2.. **`impact` evaluates** a prediction against that artifact — its relevance and weight are scored.

3.. **`provenance` protects** the result — if the artifact is later found changed or stale,the impact report becomes suspect,and the system knows to re-run the predictionor



The interfaces between modules are intentionally narrow: `impact` reads only from the `graph`'s query API; `provenance` reads only provenance attributesand emits change events. Neither touches the other's internals,and neither knows about the `skillgen` or `serve` modules. That isolation is what makes the Kaioken architecture modular — you can swap the `impact` model's scoring algorithm,replace the `graph` storage backend,or change the staleness policy,and the rest of the system behaves as before.

# Serve and Presentation

The `kaioken/serve` package is the static site generator for Kaioken. It turns a `Library` — the in-memory model of a wiki's documents, skills, and cards — into a set of HTML pages for end users. The package splits cleanly into two layers:

1. **Data shaping** (`library.ts`, `graph.ts`): read the wiki from disk, build a traversable graph of documents and files.
2. **Presentation** (`pages/*.ts`): render that data into HTML, with a shared layout, CSS, and navigation chrome.

All page renderers are pure functions that take a `Site` (the navigation context) plus page-specific data and return an HTML string. None of them touch the filesystem; they only consume what was read and computed upstream. This makes the rendering layer trivially testable and keeps file I/O confined to `library.ts`.

## Data model: `library.ts`

`readLibrary(root: string): Promise<Library>` is the entry point. It scans the wiki directory and returns a fully-populated `Library`. The `Library` interface is a snapshot of everything the UI needs:

- `chapters: WikiChapter[]`, `docs: WikiDoc[]`, and the indexing maps `byPath` and `bySource` for resolving documents by their filesystem location or provenance source.
- `skills: Skill[]` and `cards: CardSummary[]` — the generated assets that the UI exposes.
- `counts` and `freshness` — aggregate numbers (`current`, `stale`, `orphaned`, `unknown`) and a single scalar freshness ratio, used directly by the overview page's stats cards and meters.
- `judged: boolean` and `generatedAt: string` — flags the UI uses to show whether verification has run and when the library was built.

The empty state is `EMPTY_LIBRARY`, a frozen constant that gives the UI sane defaults when nothing has been generated yet — so a first run doesn't crash on `undefined`.

Two small helpers complete the module:

- `readCardFile(path: string): Promise<Record<string, unknown> | null>` reads a single card file from disk as a generic JSON object. It returns `null` on failure rather than throwing, which lets the cards page degrade gracefully for missing or malformed cards.
- `titleFromId(id: string): string` converts a technical id (like a file path) into a human-readable title. This is used to generate headings and navigation labels.

The heavier work — parsing frontmatter, extracting headings, validating documents — lives in the modules covered by other chapters (graph, impact, provenance). `readLibrary` orchestrates them: it calls out to those subsystems and assembles the result into the unified `Library` shape the UI consumes.

## Graph representation: `graph.ts`

The graph module provides a minimal, typed adjacency structure for the wiki's contents. The key distinction from the knowledge graph chapter's module is that this is the **presentation-facing** graph — a flattened, denormalized view shaped exactly for the graph visualization on the overview page.

`buildGraph(root: string, library: Library): Graph` is the constructor. It returns a `Graph` with three parts:

- `nodes: GraphNode[]` — each node has an `id`, a `label`, a `kind` of `GraphNodeKind` (`"doc" | "file" | "section"`), and a `path` for linking back to the rendered page.
- `edges: GraphEdge[]` — directed edges typed by `GraphEdgeKind` (`"contains" | "links" | "source"`). "Contains" expresses hierarchical nesting (document → section, chapter → document), "links" expresses cross-references, and "source" ties a file back to the document it was authored for.
- `stats: GraphStats` — counts of nodes and edges, plus a breakdown per kind, which the UI shows as small stat chips.

The helper `docId(path: string): string` produces the canonical node id for a document — it's just `doc:` prefixed to the path. The graph page consumes `buildGraph`'s output to draw the visualization; the node ids must be stable across builds because the graph view deep-links each node to its document page.

## The shared shell: `layout.ts`

Every page is wrapped by the `layout(site: Site, shell: Shell): string` function. `Shell` carries the page's specific content (`main`, optional `sidebar` and `rail` HTML), while `Site` carries the navigation context: the library, active tab, search state, and the like.

The layout produces a fully-formed HTML document with:

- A **sticky top bar** with the brand mark, tab navigation, and a search form. The tabs are highlighted via the `aria-current` attribute, which the CSS hooks into.
- A **left sidebar** listing all generated documents, grouped by chapter as collapsible `<details>` elements. Each group shows a count badge. The "all documents" link at the top is a direct path to the wiki index.
- A **right rail** with a small canvas-based local graph preview (styled via `.rail-graph`) and an outline of the current page's `h2`/`h3` headings.
- A `<footer>` with generation metadata.

The three-column layout (`.page--railed`) collapses responsively — on narrow screens the sidebar and rail drop below the main content rather than becoming hamburger menus. The rule after the `.topbar` block in `STYLE` states the intent explicitly: "On a phone the chrome must not cost half the screen before the first word."

### Styling: `STYLE`

`STYLE` is a ~380-line CSS template string embedded directly in the module. It defines a coherent design system via CSS custom properties in `:root`, with a full dark-mode override under `@media (prefers-color-scheme: dark)`. The token names (`--accent`, `--ok`, `--warn`, `--bad`, `--surface`, etc.) are used across all views, so changing a single token re-themes the whole site.

Notable structural decisions in the CSS:

- `color-mix()` for the translucent top bar background and `backdrop-filter: blur(8px)` for a frosted-glass effect — a deliberate progressive enhancement; browsers that lack it still get a readable bar.
- `scroll-padding-top: 84px` on `html` so anchor jumps don't hide headings under the sticky bar.
- A `:target` style on prose headings that highlights the section being navigated to — used heavily by the wiki document page.
- Print styles that hide the chrome and collapse to single column, so a printed document is just the prose.

### Small UI atoms

`layout.ts` also exports a set of tiny, pure HTML helpers used across all views:

- `badge(freshness: Freshness, title = ""): string` — renders a pill with the appropriate `badge-*` class for a freshness value. The label comes from `FRESHNESS_LABEL`, a lookup map from `Freshness` to text (`"current"`, `"stale"`, `"orphaned"`, `"unverified"`).
- `meter(fraction: number): string` — a horizontal bar showing a fraction, with `.warn`/`.bad` modifier classes when the ratio is poor.
- `stat(n: number | string, label: string): string` — a number-plus-label stat block used in the overview's stats grid.
- `hrefFor(prefix: string, path: string): string` — builds a URL from a prefix (like `/wiki/` or `/files/`) and a path, handling the necessary encoding.
- `fileLink(path: string): string` — convenience wrapper around `hrefFor` producing a `/files/` link.
- `plural(n: number, one: string, many = `${one}s`): string` — correct pluralization for UI text.
- `shortDate(iso: string): string` — formats an ISO date for display.
- `command(text: string): string` — wraps text in a `<code>` element styled for command-line snippets.
- `notFoundPage(site: Site, what: string): string` — the standard 404 view, used by any page that can't find its target.
- `str(raw: unknown): string` — a defensive stringifier that handles `undefined`/`null` gracefully so templates never crash on missing data.

## Document rendering: `wikiView.ts`

`wikiPage(site: Site): string` is the index of all wiki documents. It groups docs by chapter, renders a summary card grid, and shows freshness stats from the library.

`docPage(site: Site, path: string, title: string, headings: readonly Heading[], html: string): string` renders a single document. The `html` argument is pre-rendered markdown-to-HTML — this module doesn't parse anything, it just wraps the HTML in the page shell with the headings extracted earlier (by the graph module) for building the rail outline and the mobile table of contents.

`mobileToc(headings: readonly Heading[]): string` generates a compact, collapsible `<details>`-based table of contents for small screens — the same heading structure that the rail shows on desktop, but hidden behind a disclosure so it doesn't push the prose down.

The document page takes advantage of the CSS `:target` highlighting: when a user follows a heading anchor, that section gets a subtle background tint so the eye finds it immediately.

## Cards and skills: `cardsView.ts`

`cardsPage(site: Site): string` lists all generated cards as a grid of links. Each card gets a title and a short description derived from the card data.

`cardPage(site: Site, file: string, card: Record<string, unknown>): string` renders a single card's full contents. The `card` argument is the raw JSON object from `readCardFile` — the page function doesn't know the card's schema, it just presents whatever fields exist in a generic, readable layout. This is a deliberate contract: cards are free-form data, and the renderer stays schema-agnostic.

`skillsPage(site: Site): string` is the skills index, listing every `Skill` in the library. `skillPage(site: Site, path: string, title: string, headings: readonly Heading[], html: string): string` renders one skill's full content, following the same signature as `docPage` — pre-rendered HTML plus headings, wrapped in the standard chrome.

## Files: `filesView.ts`

`filesPage(site: Site, language: string): string` lists all files in the library, filtered by a `language` parameter. The language filter drives the "chips" navigation (the `.chips` CSS class) that lets users narrow the file listing to a specific programming language.

`filePage(site: Site, path: string): string | null` renders a single file view. It returns `null` when the file isn't found in the library — the caller is expected to fall back to `notFoundPage`. The file view shows the source code with syntax highlighting. The CSS has a `pre.code[data-lang]::before` rule that displays the language label as a small caps tag in the corner of the code block.

## Search results: `searchView.ts`

`searchPage(site: Site, query: string, hits: readonly SearchHit[], semantic: boolean, kind: string, available: Record<string, number>, limit: number): string` renders the full search results page. It shows:

- The query term in the page title.
- A `semantic` flag toggles an explanatory note that results come from embedding-based search rather than keyword matching.
- A `kind` filter (e.g., "docs", "files", "cards") with counts from `available`, rendered as chips.
- A `limit` that the UI reports as "showing first N of M results" when there are more hits than fit on the page.

`hitHref(hit: SearchHit): string` maps a search hit to a deep link — documents link to their wiki page, files to their file view, cards to their card page. It's a single dispatcher so `searchPage` stays clean.

## Overview dashboard: `overviewView.ts`

`overviewPage(site, counts, chunkCount, semantic): string` is the landing page's main content. It takes:

- `counts: Record<string, number>` — per-kind document counts (docs, files, sections) for the stat grid.
- `chunkCount` — the number of text chunks the search index was split into; shown as a stat to signal indexing depth.
- `semantic: boolean` — whether embedding-based search is enabled; the page shows a note explaining the search mode.

The page renders the freshness meter and the per-status badges, then delegates the prose summary text to `freshnessSummary(library: Library): string`. That helper produces a human-readable sentence describing the library's health — "12 current, 2 stale, 1 orphaned" — that the overview embeds near the top.

## Graph visualization: `graphView.ts`

`graphPage(): string` is the lone export. It returns a full HTML page whose only content is a `<canvas>` element. The actual graph drawing happens in client-side JavaScript which this module embeds as a `<script>` tag — the canvas is sized via the `.rail-graph` CSS class on the page shell.

The graph page does **not** take a `Library` argument. It relies on the data being embedded in the page itself at render time: the page includes the graph data (from `buildGraph`) serialized inline, and the client-side script reads it from the DOM. This keeps the server-rendered HTML self-contained — no fetch calls needed, the visualization works on `file://` or any static host.

## A note on the module boundaries

The `pages/*.ts` files (except `layout.ts`) follow a consistent signature shape:

- List pages take `(site: Site)` plus filter arguments.
- Detail pages take `(site: Site, path: string, ...renderedContent)`.

This uniformity means adding a new view is mechanical: write a render function that takes `Site` and returns a string, then wire it into the router in `kaioken/serve/src/index.ts`. The heavy lifting — parsing, validation, graph building — all happens before the page functions are called, so the views stay dumb templates over already-shaped data.

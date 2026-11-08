# Serve and Presentation

## Overview and Library

The serve module in `kaioken/serve/src` is the end-user face of Kaioken: it turns the internal graph, provenance, and skill data into static HTML pages that people can read, search, and navigate. The module is deliberately thin on business logic — it does no analysis, only presentation — but the way it structures its data model matters because every page renders from the same `Library` shape.

### The Library model

Everything the UI can show is captured in the `Library` interface in `kaioken/serve/src/library.ts`. Think of it as a single denormalized snapshot of the workspace state as of the last build. The `Library` interface at lines 100-115 declares fields for:

- `chapters`: top-level organizational units (`WikiChapter`), each with a title and id
- `docs`: the individual rendered documents (`WikiDoc`), carrying prose content, frontmatter, and verification state
- `byPath` and `bySource`: `Map` lookups so pages can find a doc by its filesystem path or by its source artifact
- `skills` and `cards`: the generated skill content and card summaries
- `counts`: a breakdown of document freshness (`current`, `stale`, `orphaned`, `unknown`)
- `freshness`: a single numeric score (0 to 1) representing overall stale-ness
- `judged` and `generatedAt`: signals for UIs that need to warn about unverified state

The individual shapes matter because they're shared across every view. `WikiDoc` (lines 31-56) carries the full prose, but also `DocVerification` (lines 59-70) which records whether the doc has been verified against its source. `WikiChapter` (lines 72-79) is almost skeletal — an id and a title — because chapters are merely grouping structures for the sidebar. `Skill` (lines 81-87) and `CardSummary` (lines 89-98) hold the metadata pages need when listing skills or cards without rendering their full bodies.

The `EMPTY_LIBRARY` constant at line 117 is the zero-state initializer. It exists so that a server that hasn't yet read a workspace can still return a well-formed `Library` with empty maps and zeroed counts — every view can handle it without null checks.

`readLibrary` (lines 138-255) is the only function that constructs a real `Library`. It takes the repository root path, walks the workspace, and populates the maps and counts. The function is async because reading and parsing every document in a large repository is I/O-bound; the serve module treats this as a build-time or startup operation, not a per-request operation.

Two small utilities sit at the bottom of `library.ts`:

- `readCardFile` (lines 569-576) reads a single card JSON file and returns its record, or `null` if the file is absent or malformed.
- `titleFromId` (lines 611-614) derives a human-readable title from an id — used when an id-based slug is all we have and we need a display name.

### Site and layout

The layout in `kaioken/serve/src/pages/layout.ts` provides the HTML chrome that wraps every page. Its entry point is the `layout` function (lines 408-462), which accepts a `Site` and a `Shell`.

The `Site` interface (lines 389-393) describes the navigation context: the site title, the path prefix, the current page's "tabs" (like Overview, Docs, Cards, Skills, Graph), and the raw `Library` used to render sidebars and counts.

The `Shell` interface (lines 395-406) captures per-page content that varies: the main body HTML, optional sidebar and rail content, breadcrumbs, and the page title.

The layout function assembles these into a complete HTML document. Its design intent is that pages only provide their unique content plus a few navigation hints; everything else — the top bar, the tabs, the find form, the sticky sidebar showing all generated docs grouped by chapter, the right rail with its local-graph preview and on-page table of contents, and the footer — comes from `layout` so the whole site is consistent and every page has identical navigation.

The `STYLE` constant (lines 7-386) is the entire CSS for the served site in a single string. It defines the design tokens (via CSS custom properties in `:root`, with a dark-mode override under `prefers-color-scheme`), the responsive breakpoints, and the component classes used everywhere: `.topbar`, `.page--railed` (the three-column grid of sidebar / main / rail), `.card`, `.stats`, `.badge`, `.prose`, `.table-wrap`, `.pager`, and the search result styles. It's a giant string rather than an external stylesheet because the serve module wants each page to be fully self-contained HTML — no separate CSS request needed.

A set of small render helpers live alongside `layout`:

- `FRESHNESS_LABEL` (lines 466-471) maps the status keys to friendly labels ("current", "stale", "orphaned", "unverified").
- `badge` (lines 473-477) renders a colored freshness pill.
- `meter` (lines 479-483) renders an inline progress bar with a fraction between 0 and 1.
- `stat` (lines 485-487) renders a big-number stat block.
- `hrefFor` (lines 490-492) builds a URL from a prefix and a path.
- `fileLink` (lines 494-496) wraps a path in a link to its source file.
- `plural` (lines 498-500) handles singular/plural wording.
- `shortDate` (lines 502-506) formats an ISO timestamp readably.
- `command` (lines 508-510) wraps text in a mono-styled command runnable snippet.
- `str` (lines 531-533) coerces an unknown value to string defensively.

Finally, `notFoundPage` (lines 515-529) is the 404 fallback: it returns a full page (using the same `layout` shell) with a message about what was requested and a back-to-overview link.

### Overview page

The `overviewPage` function in `kaioken/serve/src/pages/overviewView.ts` builds the dashboard shown at the site root. It takes the `Site` plus a `Record<string, number>` of per-chapter document counts, the total chunk count, and a boolean indicating whether the workspace uses semantic indexing — these latter two influence what the overview encourages the user to do next.

The page renders headline stats, freshness meters per chapter, a summary of recent activity, and guidance prompts. Its companion `freshnessSummary` (lines 118-128) computes the human-readable summary string from the library's freshness score and counts (e.g. "2 of 4 chapters current").

The overview exists because users coming to the served site should immediately understand what the workspace looks like: how many docs exist per chapter, how many are stale, and whether verification has happened. It's a status board, not a document browser — that's what the chapter and doc pages handle.

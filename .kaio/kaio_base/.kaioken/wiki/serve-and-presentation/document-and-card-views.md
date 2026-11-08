# Serve and Presentation: Document and Card Views

## Wiki Document Rendering

The wiki view system in `kaioken/serve/src/pages/wikiView.ts` assembles HTML for both the wiki landing page and individual document pages. The entry point is `wikiPage(site: Site): string`, which builds the wiki index from the current `Site` state.

### Document Landing and Navigation

The wiki index is constructed from a `Library` via `docRow(library: Library): (doc: WikiDoc) => string`, which returns a function that renders each `WikiDoc` as a row. The `leadDoc(chapter: WikiChapter): WikiDoc | undefined` helper identifies the chapter's opening document — the first document in a chapter that serves as its introduction. This distinction matters because lead documents get special treatment in navigation and rendering.

### Document Page Assembly

`docPage(site: Site, path: string, title: string, headings: readonly Heading[], html: string): string` is the core renderer for an individual document. It receives the pre-rendered `html` content (produced elsewhere) plus the document's `headings` for navigation. The page is composed from several modular pieces:

**Verification Badge**: `verificationBadge(doc: WikiDoc): string` renders a compact indicator of whether the verifier's claims held up during checking. This is deliberately separate from the full report so users get an at-a-glance signal without the detail.

**Verification Panel**: `verificationPanel(doc: WikiDoc): string` provides the complete verifier's findings, explicitly including failed checks — not just successes. The panel shows what was verified, what passed, and critically, what didn't, so readers can assess trustworthiness of the document's claims.

**Staleness Note**: `staleNote(doc: WikiDoc): string` adds a warning banner when the document is outdated according to the provenance system. This appears prominently before the content so users aren't misled by stale information.

### Navigation Layout

The document page uses a three-part layout with distinct concerns:

**Mobile TOC**: `mobileToc(headings: readonly Heading[]): string` generates a collapsible table of contents for narrow screens. The code comment explains the layout rationale: on narrow screens the rail moves *below* the article, so the map must move *up* into the header area to remain accessible.

**Pager**: `pager(previous: WikiDoc | undefined, next: WikiDoc | undefined): string` creates prev/next navigation at the document bottom, taking either or both as undefined to render only the appropriate links.

**Sidebar**: `sidebarFor(library: Library, current: string): string` renders the left navigation — the full generated document tree grouped by chapter and section. The `current` parameter marks the active document for highlighting.

**Right Rail**: `outlineFor(path: string, headings: readonly Heading[]): string` builds the right-side outline from the document's heading structure. At its top, `railGraphWidget(path: string): string` embeds a small local-graph preview. This widget carries an inline script — `railGraphBoot(path: string): string` — that initializes the graph widget client-side, keeping the HTML lightweight and deferring graph rendering until the page loads.

### Provenance Display

`provenancePanel(doc: WikiDoc): string` renders the provenance information for the document, showing where content came from and how it's been tracked. This panel connects the document view to the provenance system, displaying source artifacts and tracking information that users can audit.

## Card Views

`kaioken/serve/src/pages/cardsView.ts` handles skill and card rendering — the interactive, summary-style content distinct from long-form wiki documents.

### Card Listing and Detail

`cardsPage(site: Site): string` renders the index of all available cards, while `cardPage(site: Site, file: string, card: Record<string, unknown>): string` renders a single card. The `Record<string, unknown>` parameter indicates cards are schema-flexible — the view treats card data as heterogeneous key-value pairs rather than a fixed structure, accommodating different card types.

### Skill Pages

`skillsPage(site: Site): string` lists all skills in the system. `skillPage(site: Site, path: string, title: string, headings: readonly Heading[], html: string): string` renders an individual skill's full documentation page — notably, it shares the same signature shape as `docPage` from the wiki module, including headings-based navigation and pre-rendered HTML, but serves skill-specific content.

## File Views

`kaioken/serve/src/pages/filesView.ts` provides raw file browsing for end users.

`filesPage(site: Site, language: string): string` lists files filtered by programming language. The `language` parameter suggests the view groups files by their language ecosystem rather than showing everything at once.

`filePage(site: Site, path: string): string | null` renders a single file's contents. The nullable return type is critical: it signals "file not found" by returning `null` rather than throwing, allowing callers to handle missing files gracefully by falling back to a 404 or error view. File paths are validated against the site's allowed content before rendering.

# Scan and Search

## Scan Module: Directory Scanning

The `kaioken/scan` module owns the filesystem traversal that feeds every other subsystem in the repository — the knowledge graph, provenance tracking, planning, and serve all consume the directory tree that `scan` produces. Its purpose is not to store content, but to produce a stable, processable view of a directory: what files exist, where they live, and how they relate to one another.

The module is intentionally minimal at the boundary. `kaioken/scan/src/index.ts` re-exports the public API without adding logic — it is the single entry point that other packages import. The `kaioken/scan/package.json` declares the module's dependencies and scripts, while `kaioken/scan/tsconfig.json` configures TypeScript compilation. Neither file contains indexed declarations; they exist to wire the module into the workspace build and test pipeline.

### Why the module exists as a separate package

The scan module is split out from the rest of the codebase for two reasons. First, filesystem I/O is a side effect that must be isolated: every other module can operate on in-memory representations of the tree without knowing where the bytes came from. Second, the scan output is consumed by multiple packages (`graph`, `impact`, `serve` in particular), and a shared contract prevents each consumer from re-implementing traversal and risk diverging on path handling, ordering, or filtering rules.

`kaioken/scan/src/index.ts` has no indexed exports, which is deliberate: the module's public surface is defined elsewhere (in the sibling `_pi` or `graph` infrastructure), and this file exists purely to aggregate. If you add a new function to the scan implementation, you must export it here or no consumer will see it.

### Traversal semantics

The actual traversal implementation lives in the `_pi` directory (covered in the Foundation chapter) and is invoked through the scan module's public functions. The contract those functions honor:

- **Deterministic ordering** — files are returned in a sorted, depth-first order so that any two runs over the same tree produce identical sequences. This matters for provenance checksums and for card generation in the planning module, where ordering affects output stability.
- **Path normalization** — all paths are returned relative to the scan root and use forward slashes regardless of platform. Windows `C:\foo` becomes `foo`, and macOS/Linux paths keep their `./` prefix stripped. This normalization is what allows cross-platform caching and comparison.
- **Filtering** — hidden directories (`.git`, `.kaioken`) and binary blobs are skipped by default. The filter is configurable via the scan options, which the `graph` module uses to exclude non-document files from indexing.
- **Error propagation** — unreadable directories are collected and returned alongside the tree rather than silently dropped. Callers like `serve` surface these as warnings so a user sees why a file is missing from search results, rather than debugging a mysterious absence.

### Consumption patterns

The scan module's consumers depend on three properties:

1. **Stable identity** — each file is identified by its normalized relative path. This path is the primary key in the graph's document store, and the scan module guarantees it is unique and reproducible.
2. **Lineage metadata** — the scan result includes modification timestamps and file sizes. These feed the provenance module's staleness computation: if a file's timestamp changes but its content hash does not, the graph can avoid re-indexing.
3. **Tree structure** — the result is a nested tree, not a flat list. The planning module walks this tree to generate cards per directory, and the serve module uses it to render the file browser UI.

### Working on this module

When you modify the scan module, keep these invariants in mind:

- **Never reorder output** — consumers cache results; reordering breaks their assumptions.
- **Never add files to the tree** — only the traversal is allowed to decide membership. If you need derived files (e.g., generated indexes), expose them separately rather than mutating the scan result.
- **Keep `kaioken/scan/src/index.ts` as the sole export surface** — any new public function must be re-exported here, and any signature change must be reflected in the type declarations that consumers import.

The scan module is the quiet backbone of the repository — it does one job, does it deterministically, and every other module trusts its output without re-checking the filesystem. That trust is the contract you preserve as you iterate on this code.

# Knowledge Graph: Graph Construction from Documents and Code

## Overview

The graph module in `kaioken/graph/src/build.ts` is the single source of truth for turning raw artifacts and source code into a `KnowledgeGraph`. It contains two independent construction paths — `buildGraph` for document-derived knowledge and `buildCodeGraph` for import/re-export relationships — plus a set of shared utilities for node classification and path resolution. The module is deliberately split this way because documents and code have fundamentally different provenance models: documents carry explicit section IDs and claims, while code requires syntactic analysis of import statements.

## Document Graph Construction

### `buildGraph`

The primary entry point is `buildGraph(input: GraphBuildInput): KnowledgeGraph`, which derives a graph from artifacts that already exist on disk. Its shape is driven by the need to reconcile multiple provenance sources: each artifact may reference documents by ID, but those references can be expressed as full paths, bare IDs, or section-qualified claims.

The function first collects all documents from the input artifacts and builds a lookup map from document ID to its sources. It then iterates over every artifact, extracting claims and resolving them through `resolveClaim` into canonical source paths. The critical design decision here is that edges are **not** created directly from raw claim strings — they're resolved through `resolveClaim` to ensure that two artifacts claiming the same document actually point to the same node.

The graph construction follows a two-pass approach. The first pass creates all document nodes (using `nodeKindFor` to classify them) and all artifact nodes. The second pass adds edges between artifacts and documents, and between documents themselves when one document's claim references another document's ID. This separation prevents dangling edges during construction and allows `assertGraphIntegrity` to run as a final validation.

### `nodeKindFor` and `splitDocumentId`

Node classification is handled by `nodeKindFor(record: { document: string; sectionId?: string }): NodeKind`. The function inspects the document string for a namespace prefix (e.g., `doc:`, `section:`) and returns the corresponding `NodeKind`. When `sectionId` is present, it forces a section kind regardless of the prefix — this handles the case where a document ID is bare but the caller knows it refers to a section.

`splitDocumentId(document: string): { kind: NodeKind; id: string }` is the inverse operation: it strips the namespace prefix and returns both the kind and the bare ID. The two functions exist separately because `nodeKindFor` is used by `buildGraph` where the document string is already structured, while `splitDocumentId` serves consumers that need to reconstruct the bare ID for lookup operations. The separation also keeps the kind-inference logic (which has special cases for section IDs) independent from the string-parsing logic.

## Code Dependency Graph

### `buildCodeGraph`

`buildCodeGraph(input: CodeGraphInput): KnowledgeGraph` constructs a graph from source files by analyzing their import and re-export statements. Unlike `buildGraph`, this function operates purely on file paths and source text — there are no claims or provenance documents involved.

The function first scans all input files to build a `knownFiles` set, which is essential for `resolveImportSpecifier` to distinguish between resolvable imports and external packages. It then processes each file, extracting imports via `extractImportsFromSource` and resolving each specifier through `resolveImportSpecifier`. The resolution is deliberately conservative: if an import cannot be resolved to a known file, it's dropped rather than creating a phantom node. This keeps the graph free of external dependencies while still capturing intra-repository relationships.

The graph's nodes are file paths, and edges represent import/export relationships. The function also handles re-exports (e.g., `export * from './foo'`) by treating them as imports for edge-creation purposes, since they create the same dependency relationship. This is why `extractImportsFromSource` returns both import and export specifiers — the caller doesn't need to distinguish between them for graph construction.

## Shared Utilities

### Integrity and Resolution

`assertGraphIntegrity(graph: KnowledgeGraph): void` is a defensive check that runs after both construction functions. It verifies that every edge's source and target refer to registered nodes, throwing if any dangling reference is found. This is critical because both `buildGraph` and `buildCodeGraph` create edges in a second pass, and the check catches bugs where an edge is added before its endpoint node exists.

The path resolution stack is designed for cross-platform consistency:

- `toPosix(path: string): string` normalizes Windows backslashes to forward slashes, since the graph stores all paths in POSIX form regardless of the host OS.
- `resolveRelative(fromFile: string, specifier: string): string` joins a specifier with its containing file's directory, handling `./` and `../` prefixes.
- `resolveImportSpecifier(fromFile: string, specifier: string, knownFiles: ReadonlySet<string>): string | null` is the main resolver. It tries the specifier as-is, then with each extension from `EXTENSIONS` appended, then as a directory index file. The `EXTENSIONS` constant (`[".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".py", ".rs", ".go"]`) reflects the repository's polyglot nature — the graph supports TypeScript, JavaScript, Python, Rust, and Go files.

The resolver returns `null` for non-resolvable specifiers (external packages, missing files), which `buildCodeGraph` uses to filter out external dependencies.

### Claim Resolution

`resolveClaim(claim: string, sourcesByDocument: ReadonlyMap<string, ReadonlySet<string>>): string[]` is the heart of `buildGraph`'s edge creation. It takes a claim string and resolves it to one or more canonical source paths. The resolution strategy:

1. If the claim is already a known source path (exact match in the map), return it.
2. If the claim is a bare document ID, look up all sources for that document and return them.
3. If the claim is path-like (checked by `isPathLike`), try to match it against known source paths by normalizing both sides.

`isPathLike(text: string): boolean` uses a heuristic: a string is path-like if it contains a slash or ends with a known extension (via `EXTENSIONS`). This distinguishes document IDs (which are typically bare identifiers) from file paths.

The `sourcesByDocument` map is built inside `buildGraph` before claim resolution begins, ensuring that all document-to-source mappings are available regardless of artifact processing order.

### Import Extraction

`extractImportsFromSource(source: string): string[]` is a lightweight regex-based extractor. It captures both `import ... from 'specifier'` and `export ... from 'specifier'` patterns, returning the specifier strings. The regex approach is intentional — a full AST parser would be overkill for graph construction, and the regex handles the vast majority of real-world import syntax. The function returns specifiers in source order, which `buildCodeGraph` processes sequentially.

The `BACKSLASH` constant (`"\\"`) exists solely for `toPosix` to replace Windows path separators. It's a named constant rather than an inline string literal to make the replacement operation self-documenting.

### Helper Functions

Two small helpers complete the utility set:

- `sourcePaths(record: { sources: readonly { path: string }[] }): string[]` extracts just the path strings from a sources array, used when building the `sourcesByDocument` map.
- `intersection(a: readonly string[], b: readonly string[]): string[]` computes set intersection, used by `resolveClaim` to find which known source paths match a claim.

Both are module-private (`-` in the evidence) because they're internal implementation details — `intersection` in particular is a generic utility that shouldn't leak into the public API surface.

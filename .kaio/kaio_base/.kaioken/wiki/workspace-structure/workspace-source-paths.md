# Workspace Overview

This repository is a monorepo whose top-level layout is defined by `package.json` scripts and the `workspaceSourcePaths` file. The latter is the single source of truth for what counts as "workspace code"; tooling consumes it to know which directories to scan, export, or validate.

## `package.json`

The root `package.json` is the entry point for all developer workflows. It contains scripts that orchestrate the rest of the repo — building, testing, linting, and generating wiki content. It is not a package in the publishable sense; it exists to tie together the workspace's many subpackages.

Notable responsibilities:

- Declares workspace dependencies that are shared across subpackages, keeping versions aligned.
- Defines `workspaces` so that `npm install` hoists and links local packages correctly.
- Hosts the `generate` script (or equivalent) that produces the wiki from source modules — this is where `workspaceSourcePaths` comes into play.

```json
// From the root package.json — illustrative of how scripts reference workspace tooling
"scripts": {
  "generate": "node scripts/generate.js --paths workspaceSourcePaths",
  "test": "node scripts/test.js --paths workspaceSourcePaths"
}
```

The `scripts/` directory referenced here is not part of any subpackage; it the root-level glue that imports modules from `src/` and runs them against the directories listed in `workspaceSourcePaths`.

## `workspaceSourcePaths`

This file is a plain, newline-separated (or array-formatted) list of directory paths that are considered part of the workspace's "source" surface. It is consumed by:

- The generation pipeline, to know which directories contain modules that should contribute to the wiki.
- The scan and search tooling, to limit indexing to only workspace-owned files.
- The export configuration, so that any public API surface is only published from within these paths.

It has **no declarations of its own** — it is pure data. That is intentional: keeping this list separate from code means you can adjust what counts as "workspace" without touching any module logic文化交流. When you add a new top-level source directory, you update `workspaceSourcePaths` and every consumer picks it up automatically.

Because it is data, it is also language-agnostic — no `require` or `import` is needed to read it. The generation scripts parse it once at startup and pass the resulting array of paths into every module that takes a `sourcePaths` argument. These modules then use glob patterns relative to those roots to find `.ts`/`.md` files, rather than hard-coding absolute locations.

### Why this shape

The primary motivation is that the repository mixes a lot of different concerns — documentation generation, graph indexing, skill synthesis, and a full evaluation harness. Each of those subsystems needs to know "where does our stuff live?" but they should not each bake in their own copy of that answer. `workspaceSourcePaths` provides a single, shared contract.

A secondary benefit: it makes the workspace testable in isolation. You can point the tools at a temporary directory that mimics the structure of `workspaceSourcePaths` and get a deterministic run without touching the real repo.

If you are modifying which directories participate in exports or scans, start here — not in any individual module.

### Declarations

The `workspaceSourcePaths` file declares no symbols of its own; it is purely a data file listing directory paths.

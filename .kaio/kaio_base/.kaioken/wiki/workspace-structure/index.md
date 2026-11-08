# Workspace Overview

This repository is a monorepo for **Kaioken**, a framework that models the production of technical documentation as a graph of operational steps. The top-level directory contains three files that set the stage for the entire codebase: `README.md`, `package.json`, and `workspaceSourcePaths`.

## README.md

The `README.md` is the canonical entry point for any engineer new to the repository. It explains the project's purpose, its high-level architecture (the "Kaioken graph" and its impact/provenance/planning stages), and how to run the development environment. It is not code, but it defines the vocabulary you will see repeated across every module. If you ever wonder why a module is shaped a certain way, the README's conceptual framing is the first place to check.

## package.json

The `package.json` is the npm manifest that ties the workspace together. It declares **two key things** that matter for your daily workflow:

1. **Workspace configuration**: The `workspaces` field lists paths to all packages in the monorepo. Each package is registered here, which means a single `npm install` at the root resolves dependencies across all of them. When you add a new module, you register it here, not in a nested config.

2. **Scripts**: The `scripts` section provides the orchestration commands. These are the commands you will invoke on a daily basis — never run a package's local script directly unless you have a specific reason.

The shape of `package.json` reflects a deliberate choice: **all packages share a single version and a single dependency tree**. This is why you will see shared devDependencies like `typescript` and `vitest` hoisted to the root. If you need a new library, add it to the root unless the package genuinely cannot share it.

## workspaceSourcePaths

This file is a plain-text manifest of absolute or relative paths that the repository tooling uses to **discover source directories** for indexing and graph building. It is read by the `graph` module's indexing stage (see the Knowledge Graph chapter) to know which folders contain documentation, code, or other artifacts that should be modeled as nodes.

The file exists as a separate artifact rather than being embedded in configuration because it is **environment-specific**: what is a valid source path on your laptop may be a mounted volume in CI. Keeping it in a dedicated file means the graph builder never needs to guess — it reads this list, filters out missing paths, and proceeds. When you create a new research or scan directory, you add its path here.

From an operational perspective, this is the only top-level file you will edit during routine development. `README.md` changes only with major architectural decisions, and `package.json` changes only when adding/removing dependencies or packages.

## Relationship to the rest of the repository

All three files are the **contract** between you and the build system. They contain no logic themselves but determine how the foundation, the `graph` core, and every downstream module (`impact`, `provenance`, `plan`, `serve`, `skillgen`, `research`, `scan`, `search`, `evals`) are resolved, built, and indexed. If you break `workspaceSourcePaths`, the graph will silently miss folders. If you break `package.json`, nothing compiles. If you ignore `README.md`, you will lose the conceptual map that makes the codebase comprehensible.

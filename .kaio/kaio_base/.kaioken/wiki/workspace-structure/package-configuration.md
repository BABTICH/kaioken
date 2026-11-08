# Workspace Overview

The repository root is a Node.js workspace that coordinates the Kaioken system's internal packages and their shared development lifecycle. Its `package.json` is the control plane: rather than shipping a single monolithic application, it declares the set of workspaces, pins the tooling versions used across the repo, and exposes the scripts that every contributor will run day-to-day.

## Workspace Layout

The root `package.json` defines this as a private package (it is never published to a registry) and uses the `workspaces` field to group the actual implementation packages. Those packages — the ones covered in the other chapters of this wiki — are the modules under the `packages` directory. This split is deliberate: the packages directory holds the standalone libraries and the `evals` benchmark harness. Keeping them as separate workspaces lets each package declare its own dependencies and script lifecycle, while the root orchestrates cross-package operations.

Two properties in the root `package.json` are worth calling out:

- **`private: true`** — prevents accidental publication of the root as if it were a distributable package.
- **`workspaces`** — the glob patterns are the source of truth for what the package manager will treat as workspace members and where it will manage shared dependencies.

## Scripts as Orchestration

The `scripts` block is the entry point for all common developer workflows. There is no build step at the root — each workspace owns its own compile step. Instead, the root scripts fall into two categories:

1. **Validation and quality gates** — scripts that run across all workspaces, such as formatting checks, linting, typechecking, and the test runner. These are typically invoked with `npm run <name>` and are expected to exit non-zero on failure, making them suitable for CI.
2. **Utility conveniences** — scripts that wrap common command sequences (e.g., a `dev` script that starts a watch-mode compiler for one of the packages, or a `clean` script that removes build artifacts).

The root deliberately avoids duplicating the per-package logic. Each script either delegates to a workspace's own script via `npm run --workspace <name>` or runs a tool (like a formatter) whose config lives at the repo root so all packages share the same settings. This keeps the root lightweight while ensuring consistency — a contributor does not need to remember which package owns a particular tool.

## Dependencies and Tooling

The root `package.json` declares a small, focused set of **devDependencies** (no runtime dependencies at the root — all runtime code lives in the workspaces). These devDependencies are the pinned versions of the compiler, linter, formatter, and test framework that are used everywhere. Pinning them at the root (instead of in each workspace) guarantees that every contributor and every CI run uses the exact same toolchain, preventing "works on my machine" drift caused by transitive version bumps.

Because the package manager hoists these devDependencies to the root, all workspaces can invoke them without declaring their own copies — and the root's `scripts` reference them directly. If a new tool is needed, it is added to this root devDependencies block; if a workspace needs a runtime dependency, it declares it in its own `package.json` and the root remains untouched.

## Key Files at the Root

Beyond `package.json`, the repository root holds a `README.md` that documents how to install, build, and test the entire workspace — it is the first thing a new contributor should read. Configuration files at the root (e.g., for the formatter, linter, and TypeScript) are shared by all workspaces; this is why the root is not just an empty shell but the single source of truth for coding standards across the repo.

The root deliberately has **no source code** — its only job is to wire the workspaces together. If you find yourself wanting to add a shared helper, put it in a dedicated shared package, not at the root, and declare it as a workspace dependency where needed.

## Working With the Workspace

When you clone the repo:

1. Run `npm install` at the root — this resolves the `workspaces` globs and installs all dependencies for every package, hoisting shared ones to the root.
2. Use the root scripts — e.g., `npm run lint` or `npm test` — which run the appropriate tool across all workspaces.
3. For a package-specific operation, use `npm run <script> --workspace <name>` rather than `cd`-ing into the package; this keeps the workspace context correct and avoids accidentally using stale root-level tools.

The root `package.json` is intentionally minimal — it treats each package as an independent unit while providing a single, reliable entry point for the whole system. If you need to change a build or test setting, amend the root devDependencies or the relevant root config file; if you need to change runtime behavior, edit the specific workspace and let the root orchestration pick it up automatically.

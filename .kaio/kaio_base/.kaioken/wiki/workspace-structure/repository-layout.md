# Workspace Overview

## Repository Layout

This repository is a Kaioken workspace: a single npm package that contains the framework source, a CLI (`kaio`), and a set of in-repo packages, each with its own `package.json`, TypeScript config, and tests. The top level is deliberately flat — everything you need to run, test, or build any part of the system lives in the root `package.json`, and the source is split into two sibling directories: `src/` for the framework and CLI, and `packages/` for the independent, publishable modules.

The root `package.json` defines the workspace with an `exports` map that exposes each module under a stable subpath. That subpath — not a deep file import — is the public contract every other chapter in this wiki refers to:

```json package.json
{
  "name": "kaioken",
  "version": "0.0.0",
  "license": "Apache-2.0",
  "type": "module",
  "exports": {
    ".": "./src/index.ts",
    "./package.json": "./package.json",
    "./cli": "./src/cli.ts",
    "./graph": "./src/graph/mod.ts",
    "./impact": "./src/impact/mod.ts",
    "./plan": "./src/plan/mod.ts",
    "./provenance": "./src/provenance/mod.ts",
    "./research": "./src/research/mod.ts",
    "./scan": "./src/scan/mod.ts",
    "./search": "./src/search/mod.ts",
    "./serve": "./src/serve/mod.ts",
    "./skillgen": "./src/skillgen/mod.ts",
    "./evals": "./packages/evals/src/index.ts"
  }
}
```

Each subpath ends with a `mod.ts` barrel except for the CLI (`src/cli.ts`) and the evals package (`packages/evals/src/index.ts`). This means the workspace root is importable as `kaioken`, `kaioken/cli`, `kaioken/graph`, etc. — and because the files are `.ts` under an `exports` map, consumers rely on the workspace's TypeScript setup to resolve them (the project is `"type": "module"`, so ESM-first tooling is assumed).

### Top-level structure

```
/
├── README.md
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── cli.ts
│   ├── graph/mod.ts
│   ├── impact/mod.ts
│   ├── plan/mod.ts
│   ├── provenance/mod.ts
│   ├── research/mod.ts
│   ├── scan/mod.ts
│   ├── search/mod.ts
│   ├── serve/mod.ts
│   ├── skillgen/mod.ts
│   ├── _pi/            # private infrastructure (see Foundation chapter)
│   └── ...             # supporting source files
└── packages/
    ├── evals/
    │   ├── package.json
    │   └── src/index.ts
    └── ...             # other packages
```

The `src/` directory contains the framework modules. Each module there is a co-located bundle: its `mod.ts` exports the public API for that subsystem, and private implementation files live next to it. `src/index.ts` is the single entry point for a plain `import * as kaioken from "kaioken"` — it aggregates the stable exports across all modules, so a consumer who does not want the subpath granularity gets a flat namespace.

`src/_pi` is the private infrastructure directory (extensions, skills, themes, and shared configuration artifacts). It is not exported via `package.json` `exports` — it is internal to the workspace and only referenced by modules through relative imports. When reading a chapter in this wiki that says "the graph module imports from `_pi`", that is what it means: `_pi` is reachable only inside `src/`, never through the public `exports` map.

`packages/` contains standalone packages that are part of the workspace but pushed to their own publishable unit. `packages/evals` is the one you will see in this wiki: its `src/index.ts` is exported as `kaioken/evals`. Unlike the `src/` modules, this package has its own `package.json` because it carries separate dependencies (for example, probe harnesses and benchmark fixtures). In-repo packages like this one are declared in the root workspace and resolved through the `exports` map, so you can `import { ... } from "kaioken/evals"` from anywhere in the repo.

### Design intent of this layout

Two constraints shaped this structure. First, the framework is built for **single-import ergonomics**: `kaioken` alone should give you the complete toolkit, yet deep imports (`kaioken/graph`, `kaioken/serve`, etc.) must be stable enough for a consumer to depend on without fearing breakage — hence the `exports` map guarding every subpath. Second, the workspace keeps private machinery in `src/_pi` intentionally unexported: it is not part of the public contract, and any change there is free to ripple through modules without a semver concern. That is why every other chapter in this wiki describes behavior through the `mod.ts` barrels and not through `_pi` internals — those chapters document the public surface, while `_pi` is implementation detail.

### Key files at a glance

- `README.md` — quick start, contribution notes, and a pointer to this wiki; nothing here is runtime code, but it is the first file a contributor reads.
- `package.json` — workspace definition, scripts, and the authoritative `exports` map shown above. If you add a new module, you add both the `src/<name>/mod.ts` and a new line in `exports`.
- `tsconfig.json` — shared TypeScript configuration for all `src/` modules and the `packages/` packages.
- `src/index.ts` — aggregate entry point; re-exports the public API of every module so a bare `import "kaioken"` works.

When you work on a feature, the rule of thumb is: write the implementation in `src/<module>/`, export it from that module's `mod.ts`, and if the consumer needs it via a subpath, add the line to `exports` in `package.json`. Do not export `_pi` internals, and do not add a deep import of a `packages/*` source file outside of that package's own `src/index.ts`.

The repository is a monorepo-of-one package: no `workspaces` field, no separate lockfiles per package, and a single `package.json` at the root governs everything. That keeps the build and test story simple — run everything from the root, and let the `exports` map decide what is public.

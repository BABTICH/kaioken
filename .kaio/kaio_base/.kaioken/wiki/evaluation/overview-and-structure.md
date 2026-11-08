# Evaluation Framework

The `evals` package is a standalone evaluation harness for validating the behavior of the Kaioken system's core infrastructure. It exists so that the project can run automated checks against the modules described in _Workspace Overview_, _Foundation_, and the core modules (`graph`, `impact`, `provenance`, `plan`, `verify`, `serve`, and `skillgen`) without entangling test code with production logic. The package is deliberately isolated: nothing in the main repository depends on it, and it imports from the rest of the workspace read-only.

## Purpose

Evals are not unit tests in the traditional sense — they are **benchmarks and fixtures** that exercise the modules together, simulating realistic workflows (e.g., indexing a document set, generating an impact report, or rendering a card). This lets a developer regression-check behavior after changes to any of the core modules. The package answers a practical question: "did my change to the graph module break the way plans are generated?"

## Layout

At the top level, the package contains four plain configuration and metadata files. There is no source code here; all actual `.ts` files live in subdirectories covered by companion chapters. The files are:

| File | Role |
|------|------|
| `README.md` | Human-readable entry point: usage instructions, example commands, and links to the relevant module chapters. |
| `package.json` | Defines the package's name, version, scripts (e.g., `bench`), and workspace dependencies. It declares what the evals import from other packages. |
| `tsconfig.json` | Compiler configuration for this package. It extends the root tsconfig but overrides the output directory and strictness as needed for test-only code. |
| `.gitignore` | Prevents generated artifacts (e.g., `dist/`, `coverage/`, or temporary fixture data) from being committed. |

### Why this shape

The package is **not a monorepo package** in the usual sense — it does not produce a distributable artifact. Instead, it is a development-time dependency. The `package.json` lists other workspace packages as dependencies so that when you run `npm run bench` from this directory, the TypeScript compiler and runtime can resolve imports from the rest of the repo.

The `tsconfig.json` is separate from the root `tsconfig` because evals require a different module resolution strategy (often `moduleResolution: "node"` to pick up workspace symlinks) and a looser `strict` setting to allow test scaffolding that is not production-grade.

The `.gitignore` is minimal — it only ignores build outputs and local test data, because the evals themselves are meant to be committed and versioned. You want a reviewable record of how the system is tested.

## Usage in the Project

In practice, an engineer runs the evals from the `evals/` directory:

```bash
npm run bench
```

This triggers a script (defined in `package.json`) that compiles the TypeScript, executes the benchmark harness, and prints a summary of pass/fail and timing results. The harness reads fixture data from JSON files (if any) and instantiates the actual `graph`, `impact`, or `plan` classes to validate output.

Because the package is isolated, you can modify core modules without worrying about evals breaking the build — the dependency direction is one-way. This keeps the production code clean and lets evals evolve independently as the system changes. However, if you change a public API in a core module, the evals will fail to compile — which is exactly what you want: it forces you to update the evals in the same change, ensuring the battery of checks stays aligned with the code it verifies.

## Relationship to Other Chapters

This overview only covers the package shell. The actual evaluation logic — how `Fixture`s are loaded and how benchmarks are structured — is described in the companion chapters for each module. When working on evals, you will primarily touch those modules and then return here to re-run `npm run bench` to verify nothing regressed.

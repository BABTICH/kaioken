# Evaluation Framework

The `evals` package is a self-contained harness for testing and benchmarking the Kaioken toolchain. It lives in `evals/` at the repository root, separate from the `src/` modules the wiki's other chapters cover. Its job is not to run the product, but to prove the product works — and to keep proving it as models, providers, and prompts change.

## Layout and Lifecycle

Three layers sit on top of each other:

1. **Eval files** (`evals/evals/*.eval.ts`) — one file per evaluation scenario. Each exports nothing by convention; they only call the harness at module scope.
2. **The harness** (`evals/src/harness.ts`) — the engine that discovers, runs, and scores evals. This is the only code eval files import.
3. **CLI plumbing** (`evals/src/cli.ts`, `evals/src/plan.ts`, `evals/src/report.ts`) — argument parsing, test planning, and result aggregation.

The eval files are the interesting part for a new contributor. They are TypeScript modules that execute top-level calls into `harness.ts`; there is no test framework, no `describe/it`, and no assertion library. The harness *is* the framework. Each `.eval.ts` file is named for what it verifies: `smoke.eval.ts` for basic connectivity, `documentation-audit.eval.ts` for doc completeness, and the `*.docs.eval.ts` files for model/provider documentation correctness.

## The Harness

`evals/src/harness.ts` is the public surface every eval file touches. It exports the configuration and runner types that define what an eval *is*. The core contract: an eval is a named set of probes against a fixture, run through a configured model provider, with results reported by a reporter.

The harness is shaped this way to solve a concrete problem: Kaioken's behavior depends on the LLM provider (OpenAI, Anthropic, or a custom endpoint) and on the model. The same probe can pass with one provider and fail with another. So the harness separates:

- **Probes** — units of verification (explicit questions, docs assertions, smoke checks).
- **Fixtures** — the input data each probe runs against (a document, a graph state, a prompt template).
- **Providers and models** — the LLM backend configuration, selectable per eval run.
- **Reporters** — how results are emitted (terminal, JSON, CI).

This separation is why `evals/evals/openai-provider.docs.eval.ts` and `evals/evals/custom-provider.docs.eval.ts` exist as separate files despite overlapping intent: each file pins the eval to the provider it documents. The same probe set runs against different providers in different files.

### Running an Eval

`evals/src/cli.ts` parses command-line arguments and delegates to the harness. It determines which eval files to load, which provider configuration to use, and how to report results. The CLI is thin — it does not contain eval logic. All execution decisions live in `harness.ts`, so the CLI stays a stable front door while the eval surface evolves.

### Planning Runs

`evals/src/plan.ts` sits between the CLI and the harness. It is responsible for *planning*: turning the requested eval names and options into an ordered list of runnable tasks. This indirection exists because evals can have setup dependencies (e.g., a documentation audit needs a built knowledge graph first) and because some evals are expensive and benefit from batching. The planner lets the harness stay deterministic: the same plan yields the same sequence of probes, which makes failures reproducible.

### Reporting Results

`evals/src/report.ts` aggregates probe outcomes into a final report. It handles the "did this eval pass or fail" question, including partial failures — a probe that times out is not the same as one that returns a wrong answer, and the report distinguishes them. Reporters format the aggregated results for humans (terminal output) or machines (structured output for CI), driven by the CLI's flags.

## The Eval Files

Each eval file in `evals/evals/` exercises a distinct slice of the product:

- `smoke.eval.ts` — the fastest, cheapest check: can we reach the provider, get a response, and get a well-formed completion? Smoke evals run first in CI and gate everything else.
- `documentation-audit.eval.ts` — verifies that the repository's own documentation (the wiki chapters listed in the overview) is present, coherent, and up to date. This eval is how the wiki holds itself accountable.
- `models.docs.eval.ts` — checks that model documentation is accurate: model names, capabilities, and limits as documented match what the provider actually reports.
- `openai-provider.docs.eval.ts` — runs documentation probes specifically against the OpenAI provider implementation, pinning the eval to that backend.
- `custom-provider.docs.eval.ts` — the same kind of probe, but against a custom/pluggable provider. This exists so the eval framework itself can be validated without depending on a specific vendor.
- `tui.docs.eval.ts` — verifies the terminal UI components (from the serve module) render correctly and that their documentation matches the implementation.
- `acme-server.ts` — not an eval file despite the sibling `.eval.ts` files; it spins up a fixture server used by eval fixtures, so evals have a real HTTP endpoint to probe against rather than mocks.

There is no shared test data directory in the evidence — fixtures are embedded in each eval file or started by `acme-server.ts`. That keeps each eval self-contained: you can run one eval file without booting the rest of the product.

## Configuration

`evals/package.json`, `evals/tsconfig.json`, and `evals/.gitignore` are the package's own configuration. `tsconfig.json` is strict — eval files are code, and the harness wants type errors to fail the run the same way a wrong probe answer does. `.gitignore` keeps eval artifacts (reports, generated fixtures) out of the tree.

## Working With This Package

If you add a new eval, you create an eval file in `evals/evals/` that imports from `evals/src/harness.ts` and invokes the harness at module scope with your probes and fixtures.

If your eval needs a live backend, add it to `acme-server.ts` so it boots alongside the other fixture endpoints.

Wire it into `evals/src/plan.ts` so the planner knows when it can run relative to other evals.

The most common change is adjusting a probe in an existing `.eval.ts` because a model's behavior drifted — the package is designed so that change is one file, and the harness and planner absorb the rest.

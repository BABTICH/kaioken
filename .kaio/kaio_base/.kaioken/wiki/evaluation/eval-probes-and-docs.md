# Eval Probes and Documentation Tests

The `evals/evals/` directory contains a set of evaluation probes that verify documentation correctness, model output integrity, and runtime smoke tests. These are not unit tests for logic — they are executable checks that the repository's *claims* hold up: that the OpenAI provider can be instantiated, that the custom provider produces expected outputs, that models referenced in docs exist, and that the TUI doesn't crash on startup.

## File Organization

Each `.eval.ts` file corresponds to a distinct concern:

- `documentation-audit.eval.ts` — checks that the `README.md` and key doc files contain the exact, expected code snippets and commands.
- `models.docs.eval.ts` — verifies that every model name mentioned in documentation is a valid, runnable model identifier.
- `openai-provider.docs.eval.ts` — validates that the OpenAI provider can be constructed and returns plausible completions.
- `custom-provider.docs.eval.ts` — validates that the custom provider (everything that isn't OpenAI) can be constructed and yields outputs matching the documented contract.
- `smoke.eval.ts` — a minimal sanity check that the core library loads and a trivial operation succeeds.
- `tui.docs.eval.ts` — ensures the terminal UI can start without throwing, and that the rendered output matches an expected shape.
- `acme-server.ts` — not a probe itself; this is a **utility module used by the probes** that spins up a mock HTTP endpoint to simulate the OpenAI API, so the OpenAI provider tests can run offline and deterministically.

## How Probes Are Structured

Each eval follows the same pattern:

1. **Arrange** — set up any required fixtures (e.g., start the mock `acme-server` or read a sample document).
2. **Act** — run the code path under test (e.g., construct a provider, call a completion method, render the TUI).
3. **Assert** — check that the result matches the documentation's promise, using `expect` from the test framework.

There is no shared base class; each file imports only what it needs directly from `../src` or from `node:http`. This keeps evals decoupled and allows them to be run in parallel without cross-contamination.

## The Acme Server Fixture

The `acme-server.ts` file exports a single factory function that creates an HTTP server listening on an ephemeral port. Its request handler inspects the URL path:

- If the path matches `/v1/chat/completions`, it returns a canned JSON response mimicking OpenAI's API shape.
- For any other path, it returns a 404 with a JSON error body.

The server exposes a `start()` method that returns a promise resolving to `{ port, baseUrl, shutdown }`. The probes use `baseUrl` to override the provider's default endpoint, so all OpenAI-provider tests run against the mock — no real network calls, no API keys required. This is why `acme-server` exists: to make the eval suite hermetic and fast.

## Custom and OpenAI Provider Docs Evals

These two files are the most substantive probes. They share a similar skeleton:

```ts
// evals/evals/custom-provider.docs.eval.ts (parallel in openai-provider.docs.eval.ts)
import { CustomProvider } from '../src/providers/custom';
import { expect } from 'vitest';
```

The eval constructs the provider with the exact options shown in the README's "Quick Start" section — no more, no less — and then calls the documented method (e.g., `complete(prompt)`). The assertion checks that the returned object has the documented fields: `text` and `model`. Crucially, it *also* validates that the provider works when the documentation snippet is copied verbatim, meaning the constructor arguments and default values must be stable.

### Why the split between custom and openai?

The `custom-provider` probe covers the generic interface contract: any provider implementation must accept the same options and return the same shape. The `openai-provider` probe specifically verifies that the OpenAI-compatible transport used by the mock server integrates correctly with the provider factory — that is, it exercises the request path that the `acme-server` simulates.

You cannot merge them because they authenticate differently: the custom provider may take an inline token or none; the OpenAI provider requires `apiKey` and `baseUrl`. The docs eval proves that the README's example for each works exactly as written.

## documentation-audit.eval.ts

This probe parses the repository's `README.md` to extract code fences. It then asserts:

- Every fenced block tagged `ts` or `js` that appears *inside a "Usage" or "Example" section* must execute without throwing when wrapped in an async IIFE.
- Every shell command (fenced as `bash`) must match a known-good command list — anything else fails the audit.

The rationale: documentation rots silently. A user copies a snippet, hits an API change, and the eval catches it before a human reports a broken tutorial.

## models.docs.eval.ts

This probe scans all markdown files in `docs/` (and the root README) for patterns like `model="gpt-4o"` or `model='claude-3-sonnet'`. It collects every quoted model identifier, then validates each against a registry imported from `../src/models`. Models that fail validation are reported with the filename and line number. This catches typos like `gtp-4o` or stale model names after a deprecation.

## smoke.eval.ts

The lightest probe — it imports the top-level package entry point and runs two assertions:

1. The default export is a function.
2. Calling that function with a minimal object (like `{ title: 'test' }`) returns an object with a `render()` method that yields a string.

This is the "did the build break" check. It runs in CI first, before the heavier docs evals, to fail fast if the core library doesn't load.

## tui.docs.eval.ts

The TUI probe starts by spawning a child process that runs `node dist/tui/index.js --help`. It captures stdout and stderr, then asserts:

- The process exits with code `0` (i.e., the CLI parser works).
- The help text contains the flags documented in `docs/tui.md` — at minimum `--port`, `--theme`, and `--no-browser`.

If the TUI's CLI contract drifts from its documentation, this eval fails. It does *not* test interactive keystrokes or rendering — that's left to the maintainer's manual QA, because automating a full terminal session is flaky in CI.

## Running the Evals

All probes live under `evals/evals/` and are executed with the repository's harness:

```bash
npm run evals
```

The harness runs each `.eval.ts` file sequentially, collects failures, and prints a diff of the actual vs. expected output. A non-zero exit code causes CI to fail.

**Key property:** every probe relies only on files in `evals/` and public exports from `src/`. There are no network calls (thanks to `acme-server`), no environment variables, and no filesystem writes outside of the eval's own temp directory. This ensures the suite is reproducible on any machine, including offline CI runners.

## Adding a New Probe

Follow the existing pattern:

1. Create `evals/evals/<name>.docs.eval.ts` (or `.eval.ts` for non-doc probes).
2. Import `expect` from `vitest`.
3. Write a single top-level `test('…', async () => { … })` that matches the two-line skeleton above.
4. If you need a mock server, reuse `acme-server.ts` rather than duplicating HTTP logic.

The eval runner picks up any file matching `*.eval.ts` — no registry to update. But keep one rule: each probe must map to a *user-visible claim* in the docs. If it doesn't, it belongs in the unit tests under `tests/`, not here.

## Why These Files Are Shaped This Way

The `acme-server.ts` mock exists because the OpenAI provider probe must not depend on live API availability — a transient network failure would make `npm run evals` flaky, which would erode trust in the CI signal. By simulating the wire protocol, the probe tests the *client's* behavior (request construction, response parsing) without the external dependency.

The `documentation-audit` and `models` probes are deliberately string-heavy. They don't test logic; they test *consistency between prose and code*. This is the repository's answer to "docs drift" — an automated lint on the documentation itself.

Finally, `smoke.eval.ts` exists to catch build regressions early. It's the cheapest to write and the most likely to run, so it guards the entire suite from false negatives (where all evals pass but the package is actually broken).

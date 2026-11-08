# Evaluation Framework: Harness and Benchmarking Tooling

The `evals` package is a self-contained benchmarking harness that executes the
probes defined elsewhere in the repository. It is deliberately decoupled from
the rest of the codebase: its four source files — `harness.ts`, `plan.ts`,
`report.ts`, and `cli.ts` — share no infrastructure with the `_pi` directory or
the Kaioken modules. This isolation keeps benchmark runs reproducible and lets
you swap in different probe sets without touching graph or impact code.

## `harness.ts` — The Core Execution Engine

The file `evals/src/harness.ts` contains the main orchestration logic. It is
shaped as a **director** that takes a *plan* (defined in `plan.ts`) and runs each
probe against the target codebase, collecting raw measurements. Its central
export is a function that:

1. Reads the plan from disk (or from an in-memory object).
2. For each probe, constructs an isolated execution context — a temporary
   working directory, a fresh process environment, and a set of fixture
   files — so that probe runs cannot contaminate one another.
3. Invokes the probe's entry point (typically a script or a function exported
   from a probe module) and captures stdout, stderr, exit code, and wall-clock
   time.
4. Applies a timeout guard: if a probe exceeds its allotted budget, the harness
   kills the subprocess and records a `timeout` diagnosis rather than hanging
   the whole benchmark.
5. Writes each probe's raw output to a per-run directory, preserving the exact
   input/output pairs for later investigation.

Why this shape? The harness deliberately avoids in-process execution. Running
every probe as a subprocess gives you hard isolation (a segfaulting probe
cannot crash the benchmark) and makes it trivial to add timeouts, memory
limits, or resource quotas later. The cost is a small constant overhead per
probe, which is acceptable because probe counts are usually in the tens, not
the thousands.

The harness also maintains a **run manifest**: a JSON object that records
metadata about the execution — the git commit of the repository under test,
the harness version, the timestamp, and the environment variables that were
active (with secrets redacted). This manifest is the ground truth for
reproducing a benchmark run.

## `plan.ts` — Describing What to Run

`evals/src/plan.ts` defines the **plan** data model and its validation logic.
A plan is a declarative description of a benchmark session. It contains:

- A list of **probe references**: each references a probe by ID and optionally
  pins a specific version or file hash.
- A **fixture specification**: a mapping from fixture names to file globs or
  inline content. Fixtures are the input data that probes consume; they might
  be small markdown snippets, JSON blobs, or even entire directory trees.
- A set of **execution parameters**: the timeout per probe, the number of
  retries for flaky probes, and a list of environment variables to inject.

The shape of `plan.ts` is driven by a single requirement: **a plan must be
serializable to JSON and diffable**. This lets you store plans in version
control, review changes to a benchmark, and replay an exact prior run. The
validation function in this file ensures that every probe reference points to a
registered probe, every fixture glob matches at least one real file, and all
timeout values are positive integers. It throws a descriptive error (rather
than silently defaulting) on any violation, because a misconfigured plan
produces misleading benchmark results.

A key decision: fixtures are **copied**, not symlinked. Copying guarantees that
a probe cannot accidentally modify the source fixture (which would poison later
runs). The copy is made into the per-run directory that `harness.ts` creates,
so the fixture lifecycle is fully managed by the harness.

## `report.ts` — Producing Human-Readable Output

`evals/src/report.ts` is the **report generator**. After the harness finishes
executing a plan, this module consumes the run manifest and the per-probe
results to produce a structured report. The report contains:

- A summary table: per probe, the status (pass/fail/timeout), the wall-clock
  time, and the number of retries used.
- A **diff-friendly** section that lists, for each probe, the exact inputs
  (fixture contents) and outputs (stdout/stderr) as a quoted block. This
  section is what you grep when investigating a regression.
- An **aggregation** section that computes simple statistics: total runtime,
  pass rate, and mean/median runtime per probe category (if probes declare a
  category in their metadata).

The report is written in **plain text Markdown** by default, but the renderer
is a strategy object: you can swap in a JSON renderer to get machine-readable
output for CI systems. The report deliberately does *not* include the git diff
of the repository under test — that is the provenance module's job (see the
Provenance and Staleness chapter). Instead, it records the git commit SHA and
leaves diffing to external tooling.

Why Markdown? It is human-readable in a terminal, renders on GitHub, and is
trivial to postprocess with `grep`/`awk`. The JSON mode exists for tooling, but
the default experience is a file you can open in any editor.

## `cli.ts` — The Command-Line Interface

`evals/src/cli.ts` is the thin entry point that ties everything together. It
parses argv flags and dispatches to the appropriate module:

- `--plan <path>`: points at a plan JSON file. Without this flag, the CLI looks
  for a default `plan.json` in the current directory.
- `--probe <id>`: runs only a single probe (useful for debugging a specific
  failure).
- `--output <dir>`: where to write the run manifest and the report. Defaults to
  `./eval-results/<timestamp>/`.
- `--json`: switches the report renderer to JSON mode.
- `--verbose`: enables per-probe streaming output to stdout (normally suppressed
  to keep the CLI quiet).

The CLI also performs a **preflight check** before invoking the harness: it
verifies that the plan file exists, that the output directory is creatable, and
that no probe ID collides with a fixture name. This check catches most user
errors before a potentially long benchmark run starts.

The CLI intentionally does **not** implement any probe logic. It is a pure
adapter over the harness + plan + report trio. This keeps the package
dependency-free (no argument-parsing library) and makes the CLI easy to test in
isolation.

## Putting It Together

A typical benchmark session flows as:

```
cli.ts --plan myplan.json --output ./runs/
```

1. `cli.ts` reads `myplan.json`, validates it via `plan.ts`.
2. `harness.ts` creates a fresh run directory, copies fixtures, executes each
   probe as a subprocess with timeouts, and records the manifest.
3. `report.ts` renders the results to `./runs/<timestamp>/report.md` (or
   `.json` with `--json`).
4. `cli.ts` prints a one-line summary (e.g., "12 probes, 10 passed, 1 failed,
   1 timeout") and exits with a nonzero code if any probe failed — so CI can
   gate on the result.

The design keeps each file single-purpose: `plan.ts` knows only about the
plan format, `harness.ts` knows only about execution, `report.ts` knows only
about rendering, and `cli.ts` knows only about argv parsing. This means you can
reuse any piece independently — for instance, you can run the harness from a
script that generates plans programmatically, or feed the report JSON into a
dashboard without going through the CLI at all.

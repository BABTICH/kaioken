# Foundation: The `_pi` Infrastructure

The `_pi` directory is the runtime foundation of the repository — it is not part of the `src` tree, but it is what makes the whole toolchain work as a self-contained product. It holds the configuration, prompt templates, skills, themes, and extension hooks that the Kaioken machinery consumes. The directory is deliberately structured to separate *content* (what the user writes or tweaks) from *code* (what the compiler and runtime interpret), so that you can customize the system without touching the executable.

## Layout and Purpose

The top-level entries under `_pi` are:

- `.pi` — the root marker for the infrastructure; it carries no code or declarations of its own, but it is the anchor path that all other `_pi`-relative tooling resolves against.
- `.pi/prompts` — a directory (again declaration-free) whose files are user-facing prompt templates. These are not executed directly; they are read by the `skillgen` and `serve` modules in `kaioken`, which treat the templates as data. The shapes of the templates are defined in `kaioken`’s code, not here.
- `.pi/skills` — a sibling of `prompts`, holding declarative skill definitions. The `skillgen` module in `kaioken` reads these to know what proposals, templates, and writing rules to apply. Again, the directory itself is empty of code; the logic lives in the `skillgen` module.
- `.pi/themes` — contains the visual themes used by the `serve` module when rendering the UI. The `serve` module picks up theme files from this path and compiles them into the component layer. No declarations are exported from here; all theme behavior is implemented in the `serve` module’s internal component logic.
- `.pi/extensions` — the extension registry. The `graph` module watches this directory to discover sidecar definitions for new document types or artifact kinds. Extensions are loaded dynamically at runtime, which is why this directory has no static exports.
- `.pi/extensions/kaioken` — the specific extension set for the Kaioken framework itself. This is where you would drop a custom `graph`-related or `impact`-related extension definition. The graph module loads everything under this path when building the knowledge graph.
- `.pi/git` — a hidden directory used by the `provenance` module to store Git state (e.g., commit hashes, staleness timestamps). It is not meant for human editing; if you delete it, the provenance layer will recreate it lazily.
- `.pi/npm` — the npm registry mirror used by the `research` and `scan` modules for fetching artifact metadata. It is an implementation detail of the fetch pipeline, and its contents are never type-checked or imported directly.

## Why This Shape

The decision to keep every `_pi` subdirectory free of exported declarations is intentional: it forces all logic into the `kaioken` modules (graph, impact, provenance, plan, serve, skillgen, etc.) and keeps `_pi` as a pure configuration surface. That means:

- You can rename or move `_pi` to another location and only the base path constant in `kaioken` changes — all the loading code stays put.
- The `skillgen` module can validate a skill definition without importing any code from `_pi`; it just parses the file.
- The `graph` module can scan `_pi/extensions/kaioken` for new document handlers without requiring a recompile, because everything is loaded at runtime by path, not by import.

As an engineer, you will most often touch `.pi/prompts` and `.pi/skills` when adjusting user-facing behavior, and `.pi/themes` when changing the look of served documentation. The code that reads those directories is in the `kaioken` package — look for the functions that take a path as an argument and follow their `fs` calls to see how these directories are consumed.

## Practical Notes

- When you add a new skill definition in `.pi/skills`, you must also register a corresponding template in `.pi/prompts` if the skill expects one; the `skillgen` writer will fail with a missing-template error otherwise.
- The `.pi/extensions/kaioken` directory is the only place where you can hook into the graph’s artifact handling without editing the `graph` module itself. Drop a JSON file that declares a new artifact kind and the graph will pick it up on the next index.
- If you ever need to reset provenance tracking, delete `.pi/git` — but know that the `provenance` module will immediately recreate it and treat all content as stale.
- The `.pi/npm` mirror is not a cache you can prune arbitrarily; if you remove it, the `research` fetch pipeline will rebuild it on demand, but you may lose offline capability in the meantime.

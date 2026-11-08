# Foundation: Extensions and Skills

## Purpose of `_pi`

The `_pi` directory houses the infrastructure that shapes the engineering environment: extensions modify the behavior of the tooling itself, while skills package domain-specific procedures. This section focuses on the `extensions` and `skills` subdirectories; the remaining configuration artifacts (such as theme and composition files) are covered elsewhere in this wiki.

## Extensions Directory

### `.pi/extensions`

This directory is the extension registry for the environment. The intent is for each subdirectory to represent a self-contained extension that hooks into the core pipeline. Currently, the only extension present is `kaioken`; the directory exists as a convention, meaning any new extension added later should follow the same pattern: a folder at `.pi/extensions/<extension-name>` with its own manifest and bindings. At this stage, no indexed declarations exist — the directory serves as the *container* that the build and runtime systems scan to discover extensions.

### `.pi/extensions/kaioken`

The `kaioken` extension is the bridge between the Kaioken framework and the surrounding `_pi` tooling. It exists to integrate Kaioken's graph, planning, and provenance concepts into the environment without polluting the core modules. The extension is deliberately isolated in its own folder so that:

- The core environment remains framework-agnostic — if a different graph engine is adopted, only this extension needs to change.
- The extension can be versioned and deployed independently of the rest of `_pi`.

No indexed declarations are present in this folder yet; the extension itself is a placeholder for the binding logic that will wire Kaioken's module outputs (such as `plan`, `impact`, and `serve` results) into the environment's configuration and skill system. When implemented, this extension should expose entry points that the core environment can call to initialize Kaioken-specific behaviors, but the contract is not yet formalized.

## Skills Directory

### `.pi/skills`

The `skills` directory is the home for reusable, procedural knowledge that the environment can load and execute. Unlike an extension — which changes how the tooling behaves — a skill encapsulates a *recipe* for performing a specific task, such as drafting an impact report, verifying a plan's traceability, or generating a research summary. The separation is intentional:

- **Extensions** alter the runtime's capabilities (e.g., adding a new graph operation).
- **Skills** define *what to do* with those capabilities in a given situation.

Skills are expected to be written as structured documents (likely in a template form) that reference modules from the Kaioken core — `graph`, `impact`, `provenance`, `plan`, `verify`, and `serve` — by name, but the folder currently contains no indexed declarations. This indicates the skill-writing workflow is still being bootstrapped: the `skillgen` module (covered in the Skill Generation chapter) is meant to write skills into this directory programmatically.

Until skills are authored, the directory acts as the canonical location where the environment will look for `.skill` definitions. The absence of files here is not a deficiency — it is the expected starting point, since skills are generated as part of the documentation workflow rather than hand-maintained.

## Relationship Between Extensions and Skills

Both directories serve the same architectural goal: keeping the core environment small and composable while allowing behavior to be added from the outside. Extensions change the *environment*; skills change the *workflow*. The `kaioken` extension, when fleshed out, will likely query the skills directory to learn which procedures are available and map them to Kaioken modules. For now, treating `.pi/extensions` as a capability registry and `.pi/skills` as a procedure library is the correct mental model — any new capability added to the environment should respect this boundary.

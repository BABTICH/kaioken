# Foundation

The `_pi` directory is the operational heart of the workspace — not framework code, but the *authoring layer* that tells the Kaioken engine how to behave, what to generate, and which templates to follow. It is split into five categories: **prompts**, **skills**, **themes**,, **extensions**, and **configuration** for git and npm. Each category has its own subdirectory and a distinct role in how the repository produces documentation, evaluates its own health, and presents output to humans.

## Why `_pi`

The underscore prefix is deliberate: it keeps the directory out of the way of normal source-tree scanning while remaining obvious to any engineer who opens the repository root. Nothing in `_pi` is executable at runtime — it is all declarative or template-based. The Kaioken engine reads these artifacts at build time, so changing a theme or prompt here does not require a code rebuild of the core modules.

## Structure

A minimal tree of the directory:

```
_pi/
├── prompts/       # text templates fed to LLM calls
├── skills/        # skill definitions that the skillgen module consumes
├── themes/        # visual/rendering templates for serve/eval output
├── extensions/    # plugin-style augmentation points
├── .git/          # git configuration (ignore rules, hooks)
└── npm/           # npm configuration (rc, overrides)
```

The categories are **not** mutually exclusive in effect: a prompt may be referenced by a skill definition, and a theme may be used by both the serve module and the evals package. But each artifact has one *home* directory, and the engine knows exactly where to look for each kind.

## Prompts

`prompts/` holds plain-text templates that the planning and skill-generation modules pass into LLM calls. These are not code — they are instruction strings with placeholders that Kaioken fills from the current graph state and provenance data. The shape of each template matters: too rigid and the engine cannot adapt to novel artifacts; too loose and the generated output drifts from the skill definitions. In practice, prompts are written once, reviewed with the same rigor as a code review, and rarely change without a corresponding change to a skill or the evaluation flows that exercise them.

## Skills

`skills/` contains skill definitions that the `skillgen` module reads. A skill is a declarative description of “how to perform a task” — e.g., “write a CHANGELOG for this PR”, “summarize a provenance chain”, or “draft a card for a graph node”. The skill files bind a name to a prompt (from `prompts/`) and, optionally, to validation rules that the verify step runs against the output. Because skills are separate from the engine’s code, adding a new skill does not require touching the Kaioken core — it is a pure data addition.

## Themes

`themes/` holds rendering templates that the serve module and the evals package use to present output to humans. Themes control the HTML/CSS/JS wrapper around the generated documentation, cards, and graphs. If you change a theme, you change the look of *every* artifact the workspace produces, without altering any of the underlying content-generation logic. Themes are structured so that an engineer can create a new theme by adding a directory here, without modifying the serve module’s code.

## Extensions

`extensions/` is the plugin seam. Anything the core modules do not natively handle — custom artifact types, custom provenance checks, custom eval probes — can be dropped in as an extension. The engine discovers extensions at startup and exposes them to the graph, plan, and eval modules as first-class plugins. The rule is: if a behavior is specific to *this* workspace (not to Kaioken generically), it belongs in `extensions/`, not in the core framework.

## Git and npm Configuration

The remaining configuration directories are conventional but purpose-specific:

- `.git/` — repository-local git settings that the scan and provenance modules rely on. This is **not** the standard `.git` directory of a clone; it is a *configuration* directory for the workspace’s own tooling, holding ignore patterns, hook scripts, and branch-related settings that the provenance/staleness logic consults.
- `npm/` — npm-level configuration for the workspace, including `.npmrc`-style overrides and package-scoped settings. The `npm/` directory is consulted by the evaluation framework when it needs to install or update fixtures, and by the serve module when it resolves runtime dependencies for the UI.

## Relationship to Core Modules

`_pi` is not referenced by the Kaioken core code itself — the core modules treat it as an opaque configuration source. Instead, each responsible module (plan, serve, skillgen, evals) reads the relevant subdirectory when it initializes. That separation is why the workspace can ship new prompts, skills, or themes without any pull request touching the `kaioken/` framework directory: the contract is “put a file in `_pi/<category>/`, and the engine will pick it up”.

## Working Rules

When you modify anything under `_pi`:

1. **Prompts**: never change a prompt without running the evaluation suite — prompts affect every downstream generation, and the evals package records golden outputs that will show drift.
2. **Skills**: adding a skill is safe; changing an existing skill’s prompt reference can break the verify step if the new prompt changes the output shape. Update related eval fixtures in the same commit.
3. **Themes**: test with both the serve module (end-user view) and the evals package (report view) — they share the theme but render different components.
4. **Extensions**: an extension must be self-contained; the engine loads them in directory order, so avoid cross-extension dependencies unless they are declared.

The directory is small today, but it is the primary place where workspace-specific behavior lives. When in doubt about where a piece of behavior should reside, ask: *is this a rule about this repository’s content, or a rule about Kaioken generically?* If the former, it goes in `_pi`.

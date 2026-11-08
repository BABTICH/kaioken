# Themes and Configuration Artifacts

The `.pi` directory is the home of customization and tooling setup for the repository. It feeds the Kaioken engine with user-facing presentation choices, version-control behavior, package-manager conventions, and LLM prompt templates. All four subdirectories are *declaration-free* — they contain data files (JSON, Markdown, YAML scripts) rather than exported symbols — so their value lies entirely in the file layout and the conventions they encode for the rest of the workspace.

## `themes` — Visual Theming

`themes` holds visual style definitions that the presentation layer (the `serve` module) reads at runtime. Because themes are data, not code, the directory is intentionally isolated from the `src` tree: swapping a theme never requires recompilation, and the `serve` module can apply a theme by loading a flat JSON file rather than importing a stylesheet module.

The theme files describe:
- **Color palettes** for light/dark mode and semantic states (e.g., warnings, errors, accents).
- **Typography tokens** (font families, sizes, weights) used across card and graph renderers.
- **Spacing and radius constants** that keep the UI components consistent without hardcoding pixel values in component code.

Keeping these values in a raw directory (rather than a TypeScript module) also lets non-engineers (docs writers, designers) edit the look without touching the build pipeline. The `serve` module treats files in `themes` as plain JSON blobs that it merges into its render context.

## `git` — Version-Control Configuration

`git` contains repository-wide Git hooks and settings that enforce verification workflows. The files here are shell scripts or Git config fragments that run *before* commits are finalized. Their role is to prevent incomplete or stale artifacts from entering the history.

Key responsibilities handled by these files:

- **Pre-commit validation**: runs the provenance and staleness checks (from the `provenance` module) on any artifact that is about to be committed, failing the commit if a file is older than its indexed source.
- **Hooks for `kaioken plan`**: ensures that a `plan` file exists and is up-to-date before a commit touches generated cards or plans.
- **Branch policy scripts**: optionally warn if a commit targets a protected branch without a corresponding knowledge-graph update.

The scripts are plain executable files — no exports — so the Git tooling can invoke them directly via the standard `.git/hooks` symlink mechanism. This design keeps the hooks trivially testable and replaceable.

## `npm` — Package-Manager Conventions

The `npm` directory centralizes npm-specific configuration that the Workspace Overview’s `package.json` would otherwise scatter across multiple fields. It contains:

- **`.npmrc` fragments** that pin registry URLs, auth tokens, or strict peer-dependency settings.
- **Patches** for transitive dependencies that require a one-off fix until upstream releases a correction. By keeping patches in `.pi/npm`, the root `package.json` can reference them via the `"pnpm.patchedDependencies"` field (or the npm equivalent), making the source of patches visible in one place.

This separation exists because the `npm` configurations are *operational* — they affect how the `skillgen` and `evals` packages install their dev dependencies — and should not be mixed with the workspace’s declared `package.json` dependencies. If a patch is no longer needed, it is deleted here rather than requiring a `package.json` rewrite.

## `prompts` — LLM Interaction Templates

`prompts` stores the Markdown prompt templates that the `skillgen` and `graph` modules send to language models. Unlike `themes`, these files are not configuration values but *content*: each file is a full instruction document that the engine fills with variables (e.g., `{{artifact_name}}`, `{{document_context}}`) before sending.

The prompts directory serves three purposes:

1. **Authorization of prompt changes**: prompts are versioned in Git like code, so a change to the system prompt is reviewable and diffable.
2. **Decoupling prompt wording from engine logic**: the `serve` and `skillgen` modules read prompts from disk, meaning a prompt tweak requires no rebuild.
3. **Provider-agnostic templating**: each prompt file is plain Markdown with a small set of `{{curly}}` placeholders, so the same prompt works with OpenAI, Anthropic, or a local model — the engine only substitutes the variables and passes the file as the `system` message.

The prompts are deliberately *not* stored in the `src` tree because they are user-facing text, and the repository owners expect the same prompt to be answerable by any model that the `research` module can invoke. Keeping them in a dedicated directory also makes it trivial to run evaluation fixtures against multiple prompt variants under the `evals` package.

## Relationship to the Rest of the Workspace

These four directories are the *runtime surface* of the Kaioken engine:

- `themes` → consumed by the `serve` module’s render context.
- `git` → consumed by Git itself, but its scripts call into the `provenance` and `plan` modules.
- `npm` → consumed by the package manager during install; affects `skillgen` and `evals` dependency resolution.
- `prompts` → consumed by `skillgen`, `graph`, and `research` when generating or enriching content.

Because none of these directories export symbols, they are invisible to the TypeScript compiler and the knowledge-graph indexer — but they are first-class inputs to the tooling that the rest of the repository documents. Any change here should be paired with an update to the corresponding module’s test fixtures (in `evals`) and the published docs in the `serve` UI.

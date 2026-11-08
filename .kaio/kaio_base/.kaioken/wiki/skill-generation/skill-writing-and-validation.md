# Skill Writing and Validation

## Skill Writing Pipeline

`writeSkill` is the main entry point for transforming a `SkillProposal` into a complete skill document. It takes a `WriteSkillInput` which bundles the proposal, scan results, module evidence, and optional notes, then returns a `WrittenSkill` containing the generated markdown body and metadata.

The flow begins with `skillExists` checking whether the skill name already exists in the repository by scanning for files under the skill directory. If it does, `writeSkill` short-circuits early — you shouldn't regenerate an existing skill without explicit intent.

`resolveSources` bridges the proposal's requested file list against what the repository actually contains. It filters the `scan.files` array to match the patterns in `wanted`, meaning the evidence passed to the LLM is always grounded in real files. This is a deliberate guard: the proposal may ask for files that don't exist, and shipping those hallucinations to the prompt would corrupt the output.

## Prompt Construction

`buildPrompt` assembles the full instruction for the LLM. It combines:

- The `WRITE_SYSTEM` constant, which instructs the model to produce a procedural markdown document with a strict shape: task title, prerequisites, numbered steps, conventions, verification, and common mistakes
- The proposal's task description
- Evidence gathered from `gatherModuleEvidence` (file contents and declarations)
- Any additional notes passed by the caller

The system prompt emphasizes grounding — the model must cite real files and functions from the sources, quote verbatim excerpts with paths, and never invent commands or steps. It also imposes a line count target (60–150 lines) to keep output focused.

After generation, `unfence` strips markdown code fences from the LLM's response. This is necessary because models sometimes wrap their output in ``` fences even when explicitly told not to, and you need the raw markdown body for validation and storage.

## Validation Loop

The validation stage catches two failure modes: fabricated file citations and non-existent verification commands.

### File Citation Validation

`citedButMissing` scans the generated body for backtick-quoted identifiers (treated as file paths) and cross-references them against the `known` map built from the scan results. It returns paths that don't exist in the repository.

When ungrounded citations are found, `buildCorrectionPrompt` constructs a follow-up using the `REPAIR_SYSTEM` constant, which instructs the LLM to "fix exactly what the report names: replace or remove citations to files that do not exist." The body and the list of missing paths are fed back, and `writeSkill` loops this repair cycle until no missing citations remain (or a retry limit is hit).

### Verification Command Validation

`extractPrescribedCommands` pulls candidate commands from the Verification section of the markdown. It uses heuristics in `isLikelyCommand` — checking against `COMMAND_RUNNERS`, a `Set` of known package managers and test runners (`npm`, `cargo`, `pytest`, `vitest`, etc.) — and also detects commands like `npm run build` by splitting on whitespace and checking the first token.

`validateVerificationCommands` then checks each candidate command for existence:

- For commands with a leading `./` (repo-relative executables), it calls `pathExists` to verify the file exists
- Otherwise it spawns `which <command>` (or the platform equivalent) to check if it's on `PATH`

This ensures the skill document's verification steps are actually runnable — a common failure where an agent copies a command from another project that doesn't exist here.

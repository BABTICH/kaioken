# Skill Generation

The skill generation system lives in `kaioken/skillgen/src` and has two phases: first, `propose.ts` discovers what skills the user's repository *could* benefit from; second, `write.ts` turns those proposals into actual skill files with verification harnesses. The split exists because proposal is cheap and stateless — pure discovery from repository structure — while writing is expensive and stateful, involving file I/O and command execution. This separation lets callers preview proposals without committing to a write, and lets the write pipeline retry or partially fail without re-running discovery.

## Proposals (`propose.ts`)

The proposal phase produces `SkillProposal` objects describing a skill that *could* exist. Its shape is deliberately flat and serializable — it's the contract between discovery and writing, and it never holds runtime resources.

```ts
// kaioken/skillgen/src/propose.ts:17-26
interface SkillProposal
```

A proposal carries everything the writer needs to know: the proposed name, a description, suggested file paths for the skill's implementation, an invocation example showing how the skill would be called, and a rationale. The field naming is paired with `WriteSkillInput` in `write.ts` — proposals are literally the "shopping list" that the writer consumes.

`proposeSkills` takes a `ProposeInput` (the repository root and any explicit skill names to focus on) and returns `Promise<SkillProposal[]>`. Its main workhorse is `discoverRepoCommands`, which scans the repository to build a functional vocabulary of what the project *does*. This is the key design decision: proposals aren't templated from a static list; they're generated from the actual commands present in the repo.

```ts
// kaioken/skillgen/src/propose.ts:68-119
async function discoverRepoCommands(root: string): Promise<string[]>
```

`discoverRepoCommands` reads `package.json` from the root, extracts script names and dependencies, and returns them as a deduplicated list of command-like strings. It filters out noise — e.g., lifecycle scripts like `preinstall` that aren't useful as skill invocations — and normalizes the output. The reason it returns bare strings rather than structured objects is that proposal logic only needs to pattern-match against command names; richer metadata would just be discarded.

`proposeSkills` then maps those discovered commands against a set of heuristics to produce `SkillProposal`s. For each command it recognizes a pattern for (test, lint, build, etc.), it constructs a proposal whose rationale cites the specific command it found. The `slug` function (lines 182-189) converts a command name like `"run test suite"` into a filesystem-safe identifier like `test-suite` — used both for the proposal name and later for the generated skill directory. It's a pure string transform with no I/O, kept separate so both proposal and write phases can share the same slug logic.

## Writing Skills (`write.ts`)

The write phase consumes proposals and produces concrete skill files on disk. `WriteSkillInput` mirrors `SkillProposal` but is explicitly the *input* boundary — it requires the target root directory, the skill name, and the verification commands, and optionally accepts a description and example invocation. The asymmetry from `SkillProposal` matters: proposals are speculative, while `WriteSkillInput` has already been validated and is ready to commit.

```ts
// kaioken/skillgen/src/write.ts:76-96
interface WrittenSkill
```

`WrittenSkill` is the return type: it reports the path where the skill was written, and a list of verification results. This lets the caller know not just *that* the skill was created, but whether its verification commands actually pass in the current environment — important because a generated skill whose verification fails is arguably worse than no skill.

The primary entry point is `writeSkill(input)`. It first calls `skillExists(root, name)` to check for a collision — the function returns the skill path if it exists, and `writeSkill` refuses to overwrite by default, returning early with an error. Collision avoidance is essential because the skill directory name is derived from `slug(name)`, so two different proposals could theoretically slug to the same directory unless explicitly guarded.

`skillExists` does a filesystem probe: it constructs the expected skill directory path from `name` and returns whether that path exists in `root`. This is deliberately simple — no globbing, no partial matching — because writers should fail loudly on ambiguity.

The rest of `writeSkill` builds the skill content from the input. It:

1. Constructs the skill directory under `root` using the slugged name.
2. Writes a `SKILL.md` file whose frontmatter includes the description and example, and whose body includes the verification command list.
3. Writes an `execute.sh` script that runs the verification commands.
4. Runs the verification commands via `validateVerificationCommands` and reports results in `WrittenSkill`.

The two helper functions underneath handle the command validation logic, which is the most safety-critical part of generation.

```ts
// kaioken/skillgen/src/write.ts:320-345
function extractPrescribedCommands(body: string): string[]
```

`extractPrescribedCommands` parses the `SKILL.md` body for commands that the skill *requires* to run — these are commands listed with a specific marker (e.g., in a "Verification" section or as fenced blocks). It returns them as a flat string array. The function is pure and deterministic because the same body must always yield the same command list; any parsing ambiguity would make generated skills non-reproducible.

```ts
// kaioken/skillgen/src/write.ts:356-459
async function validateVerificationCommands(
  root: string,
  commands: readonly string[],
): Promise<string[]>
```

`validateVerificationCommands` is the last line of defense. It takes the extracted commands and executes each one in the target `root` directory, collecting any that fail (non-zero exit) into the returned array. The reading of the return value is inverted from what you might expect: it returns the list of **failing** commands, not the passing ones. `writeSkill` uses this to populate the `verification` field on `WrittenSkill` — if the list is empty, verification passed; if not, the caller can see exactly which commands are broken in this environment. The function doesn't throw on failure because a skill can still be useful even if one verification command is temporarily broken (e.g., a linter not yet installed); it just records the friction point.

## Data flow

The two files form a pipeline you can describe end-to-end without touching other modules:

```
discoverRepoCommands(root)
        │  (string[] of commands)
        ▼
proposeSkills(ProposeInput)
        │  (SkillProposal[])
        ▼
writeSkill(WriteSkillInput)         ──►  skillExists(root, name)
        │                                  │
        │                                  ▼  (collision check)
        │                           build SKILL.md + execute.sh
        │                                  │
        ▼                                  ▼
validateVerificationCommands(root, cmds)
        │  (failing commands)
        ▼
WrittenSkill
```

The module boundary is intentionally coarse: `index.ts` re-exports the public API, but the guts are split into these two responsibilities so that tooling (like a CLI or a future web UI) can reuse `proposeSkills` without pulling in file-writing logic, and conversely can reuse `writeSkill` with hand-curated proposals that never went through discovery. The package's `tsconfig.json` and `package.json` carry no project-specific logic — they're standard build configuration for the skillgen package — so the interesting seams are entirely in these two files.

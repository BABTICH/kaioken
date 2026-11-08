# Skill Generation

## Proposal Generation

Skill generation begins with `proposeSkills` in `kaioken/skillgen/src/propose.ts`. This function takes a `ProposeInput` — which bundles repository evidence, available chapters, and planning notes — and asks a model to decide which skills a repository deserves. The output is a list of `SkillProposal` objects, each carrying a `name`, `description`, `task`, and `files` array.

The shape of the input is defined by `ProposeInput` (lines 53-66):

```ts
interface ProposeInput
```

It contains the `evidence` (a `RepositoryEvidence` object), `chapters` (readonly strings), `notes` (readonly strings), and optionally `commands` (readonly strings). The `commands` array can be pre-filled by `discoverRepoCommands`, which scans a root directory for package scripts and executable hints, returning an array of command strings an agent might run in this repo.

The core prompt is `PLAN_SYSTEM` (lines 28-51), a constant string that instructs the model to propose 5-12 skills. The prompt makes a hard distinction: skills are **task-oriented guides** ("how do I do X in THIS project"), not descriptions of what the code is — that is the wiki's job. It explicitly bans "architecture-overview" or "project-structure" style proposals and anything a general model already knows (e.g., "how to write Go"). Each proposal must include:

- `name`: kebab-case, verb-led, specific ("add-a-tui-command", not "tui")
- `description`: one or two sentences covering what it teaches AND the concrete triggers an agent should match against at runtime
- `task`: one sentence stating the task
- `files`: repo-relative paths of real examples an agent should imitate

The expected output format is strict JSON: `{"skills":[{"name":"...","description":"...","task":"...","files":["..."]}]}`.

### From Model Output to Clean Proposals

`proposeSkills` calls the model with a prompt assembled by `buildPlanPrompt` (lines 195-244). That function concatenates the repository evidence, chapter list, notes, and discovered commands into a single prompt string, ensuring the model sees the same repository context the wiki itself was built from. The result is then parsed and each candidate entry is passed through `normalise` (lines 161-179), which validates and coerces an unknown JSON value into a `SkillProposal` or returns `null` if the shape is wrong. Only non-null results survive.

`normalise` relies on two helpers:

- `text` (lines 191-193): coerces an unknown value to a string, returning `""` for anything that isn't a string
- `slug` (lines 182-189): takes a model-supplied name and makes it safe to use as a file name — lowercasing, replacing spaces and illegal characters, ensuring the result is something you can write to disk without escaping

The pipeline is deliberately defensive: model output is untrusted, variable, and occasionally malformed. `normalise` plus `slug` guarantee that a bad model response cannot crash the generator or produce a file name that breaks the skill loader.

### Why This Shape

The proposal step exists because skills are cheap to write but expensive to maintain. `PLAN_SYSTEM` steers the model toward *recurring tasks* a contributor actually performs — adding an endpoint, writing a test, debugging a build — and away from static descriptions that duplicate the wiki. The `files` field is the key to making each skill concrete: it points at real, repo-relative examples that the subsequent skill-writing step can imitate, so a skill for "add-a-database-migration" is grounded in the actual migration files this repository uses, not a generic recipe.

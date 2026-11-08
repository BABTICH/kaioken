# The Command-Line Interface

This chapter details how the `kaioken` CLI processes command-line arguments, dispatches commands, and resolves the AI model to use.  The CLI's structure is designed for extensibility; new commands can be added without significant refactoring.

The core of the command processing resides in `apps/cli/src/main.ts`.  The `USAGE` constant, defined as a multi-line string, documents the available commands and options. The `Flags` interface represents the parsed command-line arguments.

The `parseArgs` function, also in `main.ts`, handles argument parsing using a library not detailed in this wiki. It consumes the command-line arguments (`argv: string[]`) and returns a `Flags` object or `null` if parsing fails.

```ts
// apps/cli/src/main.ts:117-218
async function parseArgs(argv: string[]): Flags | null {
  // ... argument parsing logic ...
}
```

The `main` function, the entry point of the CLI, then orchestrates the command execution. It calls `resolveModelClient` and `resolveModel` to establish a connection to an AI model, which is a crucial step before executing commands that require model inference.

The `resolveModel` function in `apps/cli/src/model.ts` is responsible for determining which model to use. It first attempts to read a model specification from a local file (`.kaioken/model.json` by default), and falls back to an environment variable (`KAIOKEN_MODEL`) if that file isn't present.  If neither is available, it reports an error, halting execution because most commands require a model.

```ts
// apps/cli/src/model.ts:85-199
async function resolveModel(flags: Flags): Promise<ResolvedModel> {
  // ... model resolution logic ...
}
```

The `resolveModelClient` function similarly deals with resolving a model client. It is essentially the same function as `resolveModel`, but stops one step earlier to return the client directly.

The `synthesizeModel` function also in `model.ts` allows the construction of model descriptors for models that are not found in the provider catalogs. This is useful for configuration overriding or for local models.

After the model is resolved, the appropriate command handler is invoked based on the `flags.command` property.  Here's a brief overview of the main commands:

*   `scan`:  Initiates a repository scan, producing `.kaioken/scan.json` and `.kaioken/index.json` which are foundational for the entire tool.
*   `symbols <target>`: Resolves symbol declarations using the `SymbolIndex` (described in its own wiki chapter).
*   `search <query>`: Performs a ranked search across the indexed knowledge. Semantic ranking is enabled if an embedding provider is configured.
*   `serve`: Starts a local web server for browsing the indexed knowledge.
*   `plan`: Proposes a module tree, writing to `.kaioken/module-plan.yaml`.  The `--check` flag validates the plan against the scan data without involving a model.
*   `cards`: Generates knowledge cards based on the module plan, and validates their claims against the structural index.
*   `wiki`: Generates wiki chapters and sections, and verifies their claims.  `--plan` stops after outlining the wiki. `--check` validates against the scan, not a model.
*   `status`: Reports repository state and documentation drift.
*   `update`: Regenerates documents that have changed.  `--dry-run` shows what would be updated.
*   `chat`:  Allows interactive querying of the engine. The `--write` flag enables file modification, requiring confirmation for each change unless `--yes` is used. `--verify` forces a check, even if no changes were made.
*   `verify`: Executes the repository's build and test commands, reporting the outcome.  `--dry-run` simulates the process.
*   `graph`:  Generates a knowledge graph, written to `.kaioken/graph.json`.
*   `export`: Bundles the knowledge for external use.
*   `research <q>`: Performs web research and integrates the findings with the local knowledge base. Requires a search provider configured (either `TAVILY_API_KEY` or DuckDuckGo).

The `knowledge.ts` module, specifically the `loadKnowledge` function, is responsible for loading the indexed knowledge, including the results of the scan. This knowledge is then made available to the `chat` command and other tools that require access to the repository's metadata. `currentScan` is called by `loadKnowledge`.

```ts
// apps/cli/src/knowledge.ts:24-57
async function loadKnowledge(root: string, options: { force?: boolean } = {}, ): Promise<LoadedKnowledge> {
  // ... loading knowledge ...
}
```

Finally, `describeFailure` and `extractProviderMessage` in `model.ts` handle errors coming from the AI model providers. They provide human-readable descriptions of failure conditions, making debugging easier.

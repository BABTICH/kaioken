# Skill Generation

## Package and Module Exports

The skill generation system lives in `kaioken/skillgen`, a standalone package that exposes its API through `kaioken/skillgen/src/index.ts`. The package declares its boundaries in `kaioken/skillgen/package.json`: it names the entry point, the TypeScript compilation target, and the workspace dependencies it needs to function. Dependencies are declared here rather than inferred at build time, so that any consumer knows exactly what runtime and type-level requirements to satisfy.

`kaioken/skillgen/tsconfig.json` configures how the TypeScript compiler sees this package: strict settings, module resolution consistent with the rest of the workspace, and output paths that match the `package.json` entry. This matters because skill generation is a write-heavy subsystem — it produces `.skill.md` and `.template.md` files on disk — so the package's build must preserve source maps and declarations for downstream tooling, not just emit executable JavaScript.

The `index.ts` entry point is a re-export module that forwards to the building blocks implemented elsewhere in the package. All actual logic lives in the modules this entry point forwards to. When extending the skill generation system, add new exports to `index.ts` and update the `package.json` dependency list only if the new code pulls in a package not already declared. Keep the tsconfig in sync if you change the output layout.

As of this writing, the declarations in `index.ts` are not indexed, so no specific export names can be enumerated here. The `package.json` and `tsconfig.json` files also contain no indexed declarations.

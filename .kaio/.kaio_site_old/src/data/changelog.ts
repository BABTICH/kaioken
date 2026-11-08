export interface ChangelogEntry {
  version: string
  date: string
  highlights: {
    tag: "FEAT" | "PERF" | "FIX" | "MODELS"
    title: string
    description: string
  }[]
}

export const CHANGELOG: ChangelogEntry[] = [
  {
    version: "v2.0.0",
    date: "Latest Release",
    highlights: [
      {
        tag: "FEAT",
        title: "Isolated Git Worktree Sub-Agent Delegation",
        description: "Execute parallel tasks in sandboxed git worktrees without mutating your active branch.",
      },
      {
        tag: "FEAT",
        title: "Autonomous Go Verification Gate & Self-Healing Loop",
        description: "Auto-detects test suites and runs up to 3 diagnostic healing passes on compiler errors.",
      },
      {
        tag: "FEAT",
        title: "Extended Reasoning Chain-of-Thought Engine",
        description: "Full streaming reasoning tokens support with custom thinking budget controls and multi-turn branching.",
      },
      {
        tag: "PERF",
        title: "Symbol Call Graph Indexing 4× Speedup",
        description: "Parallel AST parsing with caching reduces initial repo indexing to sub-second times.",
      },
    ],
  },
  {
    version: "v1.9.4",
    date: "Previous Release",
    highlights: [
      {
        tag: "FEAT",
        title: "/impact Blast Radius Analyzer",
        description: "Detects dirty files, affected downstream callers, and stale knowledge cards before editing.",
      },
      {
        tag: "FEAT",
        title: "Agent Skills Specification Compatibility",
        description: "Loads procedural conventions from standard .agent/skills/ directory.",
      },
      {
        tag: "FIX",
        title: "Incremental Wiki Diff Invalidation",
        description: "Fixed boundary detection for monorepos with nested go.mod and package.json files.",
      },
    ],
  },
]

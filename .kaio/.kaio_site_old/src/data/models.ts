export interface EngineCore {
  name: string
  model: string
  badge: string
  type: "frontier" | "reasoning" | "open" | "local"
  description: string
}

export const SUPPORTED_MODELS: EngineCore[] = [
  {
    name: "Architectural AST Parser",
    model: "Symbol Call Graph Engine",
    badge: "Whole-Repo Scope",
    type: "frontier",
    description: "Deep codebase parsing that maps types, symbols, and cross-package caller hierarchies into a verified index.",
  },
  {
    name: "Autonomous Verification Gate",
    model: "Self-Healing Diagnostic Gate",
    badge: "Zero Regression",
    type: "reasoning",
    description: "Native test suite detection and multi-pass diagnostic repair loops that ensure every proposed edit builds clean.",
  },
  {
    name: "Git Worktree Delegation",
    model: "Parallel Worktree Sandboxing",
    badge: "Branch Safety",
    type: "frontier",
    description: "Spins up temporary git worktrees where sub-agents refactor and test without touching your active working branch.",
  },
  {
    name: "Deep Knowledge Engine",
    model: "5-Card Module Generator",
    badge: "Incremental Diff",
    type: "frontier",
    description: "Multi-pass architectural documentation with content-hashed incremental updates — unchanged code is never re-indexed.",
  },
  {
    name: "Air-Gapped Local Execution",
    model: "100% Offline Hardware Weights",
    badge: "100% Offline",
    type: "local",
    description: "Run entirely air-gapped on your local workstation or private datacenter. Zero data or telemetry leaves your machine.",
  },
  {
    name: "Unified Gateway Engine",
    model: "Universal Provider Compatibility",
    badge: "Zero Lock-In",
    type: "frontier",
    description: "Connect to any cloud reasoning backend or custom private endpoints. Bring your own keys with zero proxy markup.",
  },
]

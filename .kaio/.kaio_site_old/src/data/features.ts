export interface FeatureItem {
  id: string
  number: string
  title: string
  subtitle: string
  description: string
  tag: string
  terminalOutput: {
    command: string
    lines: {
      type: "user" | "tool" | "toolres" | "add" | "del" | "ok" | "dim" | "warn"
      text: string
    }[]
  }
}

export const FEATURES: FeatureItem[] = [
  {
    id: "worktree",
    number: "01",
    title: "Isolated Worktree Delegation",
    subtitle: "Safe sub-agents in parallel git worktrees",
    description:
      "Sub-agents operate in temporary git worktrees with full execution isolation. They compile, run tests, and benchmark without touching your main working tree until you approve the diff.",
    tag: "git worktree",
    terminalOutput: {
      command: "kaioken worktree spawn --task 'refactor auth jwt'",
      lines: [
        { type: "tool", text: "● git worktree add .kaioken/worktrees/agent-auth branch/refactor-auth" },
        { type: "toolres", text: "  ↳ isolated worktree created at commit 7fa90c" },
        { type: "tool", text: "● sub-agent running in parallel workspace..." },
        { type: "del", text: "-   legacyTokenVerify(token)" },
        { type: "add", text: "+   jwt.ValidateToken(token, signingKey)" },
        { type: "tool", text: "● checking git diff against main..." },
        { type: "ok", text: "✓ 2 files modified · 0 conflicts · ready for review" },
      ],
    },
  },
  {
    id: "verify",
    number: "02",
    title: "Autonomous Verification Gate",
    subtitle: "Self-healing test diagnostics before you review",
    description:
      "Detects build and test commands automatically across Go, Rust, TypeScript, and Python. If a test fails, Kaioken diagnoses the failure and attempts self-healing passes before asking for review.",
    tag: "self-healing loop",
    terminalOutput: {
      command: "kaioken verify --strict",
      lines: [
        { type: "tool", text: "● detected test suite: go test ./internal/..." },
        { type: "warn", text: "  ↳ pass 1: test failure in handler_test.go:42 (nil pointer)" },
        { type: "tool", text: "● background diagnostic agent analyzing stacktrace..." },
        { type: "add", text: "+   if req.Header == nil { req.Header = make(http.Header) }" },
        { type: "tool", text: "● re-running gate: go test ./internal/..." },
        { type: "ok", text: "✓ all 38 tests passed · hard Go verification gate OK" },
      ],
    },
  },
  {
    id: "wiki",
    number: "03",
    title: "Deep Architectural Wiki",
    subtitle: "Whole-codebase comprehension with 5-card schemas",
    description:
      "Scans the entire repository AST, organizes modules into modules.yaml, and generates dense knowledge cards verified against symbol ground truth at ×3–×10 depth.",
    tag: "modules.yaml",
    terminalOutput: {
      command: "kaioken wiki --depth 3",
      lines: [
        { type: "tool", text: "● indexing AST: 214 source files · 1,480 symbols" },
        { type: "tool", text: "● verifying modules.yaml against call graph..." },
        { type: "toolres", text: "  ↳ module [core/engine]: 5 cards generated" },
        { type: "toolres", text: "  ↳ module [api/gateway]: 5 cards generated" },
        { type: "ok", text: "✓ deep wiki written to .kaioken/wiki/ (commit af48b2 recorded)" },
      ],
    },
  },
  {
    id: "impact",
    number: "04",
    title: "Symbol Blast Radius (/impact)",
    subtitle: "Predict breakages before editing a single line",
    description:
      "Queries symbol dependencies across your entire codebase before proposing an edit. Identifies affected downstream callers, dirty packages, stale skills, and exact tests to re-run.",
    tag: "AST graph",
    terminalOutput: {
      command: "kaioken /impact internal/store/user.go:UpdateUser",
      lines: [
        { type: "tool", text: "● analyzing AST call graph for UpdateUser..." },
        { type: "toolres", text: "  ↳ 4 direct callers: auth/session.go, api/v1/user.go" },
        { type: "toolres", text: "  ↳ 2 test suites affected: store_test.go, user_e2e_test.go" },
        { type: "warn", text: "  ↳ 1 stale knowledge card detected: cards/store.md" },
        { type: "ok", text: "✓ impact report generated · recommended tests tagged" },
      ],
    },
  },
  {
    id: "skills",
    number: "05",
    title: "Procedural Agent Skills",
    subtitle: "Teaches the model your repository conventions",
    description:
      "A skill explains how to do a specific task in your repo — which files to touch, in what sequence, following which conventions. Conforms to the standard Agent Skills specification.",
    tag: ".agent/skills",
    terminalOutput: {
      command: "kaioken skills list",
      lines: [
        { type: "tool", text: "● reading .agent/skills catalog..." },
        { type: "toolres", text: "  ✓ add-endpoint      — create handler + router + openapi spec" },
        { type: "toolres", text: "  ✓ db-migration      — postgres goose migration pattern" },
        { type: "toolres", text: "  ✓ write-e2e-test     — playwright browser fixture recipe" },
        { type: "ok", text: "✓ 3 active skills loaded into prompt context" },
      ],
    },
  },
  {
    id: "dual-surface",
    number: "06",
    title: "Dual Surface (TUI + Studio)",
    subtitle: "High-velocity terminal + visual multi-pane GUI",
    description:
      "Stay in flow state with the blazing Lipgloss terminal for fast commands, or switch to the native Studio GUI for visual diff inspection, module dependency graphs, and concurrent agent monitoring.",
    tag: "TUI + Studio",
    terminalOutput: {
      command: "kaioken studio --open",
      lines: [
        { type: "tool", text: "● launching Kaioken Studio window..." },
        { type: "toolres", text: "  ↳ visual diff inspector active" },
        { type: "toolres", text: "  ↳ interactive module dependency graph loaded" },
        { type: "toolres", text: "  ↳ 2 concurrent sub-agent worktrees connected" },
        { type: "ok", text: "✓ studio running on local bridge" },
      ],
    },
  },
  {
    id: "zero-lockin",
    number: "07",
    title: "Zero Vendor Lock-in",
    subtitle: "Frontier reasoning engines or 100% offline weights",
    description:
      "Connect seamlessly to any cloud reasoning backend or run 100% locally with private hardware weights. Zero mandatory cloud proxies, zero telemetry, zero credit markups.",
    tag: "Open & Private",
    terminalOutput: {
      command: "kaioken config --engine",
      lines: [
        { type: "tool", text: "● active backend: Private Reasoning Engine" },
        { type: "toolres", text: "  ↳ mode: architectural-grounding (thinking: high)" },
        { type: "toolres", text: "  ↳ fallback: air-gapped-local-weights" },
        { type: "ok", text: "✓ zero lock-in · bring your own keys or local weights" },
      ],
    },
  },
]

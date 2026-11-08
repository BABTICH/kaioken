export interface FaqItem {
  question: string
  answer: string
  category?: string
}

export const FAQS: FaqItem[] = [
  {
    question: "How does Git Worktree isolation work in Kaioken v2?",
    answer:
      "When you delegate tasks or spawn sub-agents, Kaioken creates temporary git worktrees in `.kaioken/worktrees/`. Sub-agents checkout separate branches, run tests, and perform builds without modifying your current working directory. Your editor and git index remain completely untouched until you review and approve the unified diff patch.",
  },
  {
    question: "What is the Autonomous Verification Gate?",
    answer:
      "Unlike standard coding agents that assume generated code works without executing it, Kaioken auto-detects build/test setups (go.mod, Makefile, package.json, Cargo.toml, pyproject.toml). If tests or compilation fail, a background diagnostic agent analyzes the error stacktrace and attempts self-healing fixes before presenting the result.",
  },
  {
    question: "Can I use Kaioken 100% offline on local hardware?",
    answer:
      "Yes. Kaioken supports air-gapped local execution through any compatible local inference server (LM Studio, llama.cpp, or native local binaries). You can run open weights completely offline with zero internet access and zero telemetry.",
  },
  {
    question: "How does the Deep Wiki and Knowledge Engine work?",
    answer:
      "Kaioken scans your repository's AST and symbol call graphs to propose a modules.yaml map. It then generates dense 5-file knowledge cards per module. When code changes in subsequent commits, Kaioken performs git-diff incremental invalidation — revising only the specific cards invalidated by the diff rather than re-indexing everything.",
  },
  {
    question: "How does Kaioken compare to conventional coding assistants?",
    answer:
      "While conventional coding assistants operate directly on your active working tree and rely on shallow grep or semantic chunk retrieval, Kaioken v2 is architecturally grounded. It pairs full-AST symbol knowledge with isolated git worktrees, automated verification gates, and procedural repo skills.",
  },
  {
    question: "Is Kaioken open-source and hackable?",
    answer:
      "Yes. Kaioken is built in Go and licensed under MIT. It includes an open Agent Skills system, custom slash commands, MCP (Model Context Protocol) server integration, and sub-agent workflows you can customize in `.kaioken/`.",
  },
]

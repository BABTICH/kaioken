# Kaio Base (Kaioken Base Monorepo)

A deterministic truth layer and offline knowledge engine: an offline AST symbol oracle, provenance-tracked wiki cascade, and verification gate.

Exported directly from `kaiopi` with all core packages, Pi bridge extensions, packaging scripts, themes, and truth store.

---

## Repository Structure

```
kaio_base/
├── kaioken/                    # Core offline packages (18 packages + CLI)
│   ├── completion/             # @kaioken/completion: speculative completion & prompts
│   ├── evals/                  # @kaioken/evals: grounding evals suite & test harness
│   ├── gitops/                 # @kaioken/gitops: worktree delegation & clean merges
│   ├── graph/                  # @kaioken/graph: symbol & file dependency graph engine
│   ├── impact/                 # @kaioken/impact: blast radius & change impact analyzer
│   ├── index/                  # @kaioken/index: AST parser, symbol oracle & visibility
│   ├── modelport/              # @kaioken/modelport: model proxy & capability abstraction
│   ├── plan/                   # @kaioken/plan: module architecture & knowledge cards
│   ├── provenance/             # @kaioken/provenance: staleness detection & truth drift
│   ├── research/               # @kaioken/research: grounded web research & intelligence
│   ├── scan/                   # @kaioken/scan: repo discovery, entropy & risk shield
│   ├── search/                 # @kaioken/search: lexical indexing & BM25 retrieval
│   ├── serve/                  # @kaioken/serve: local interactive web UI & knowledge graph
│   ├── skillgen/               # @kaioken/skillgen: autonomous skill generator & critique
│   ├── skills/                 # @kaioken/skills: procedure catalog & execution engine
│   ├── verify/                 # @kaioken/verify: test runner, gate & repair loop
│   ├── verifycore/             # @kaioken/verifycore: anti-hallucination claim shield
│   ├── wiki/                   # @kaioken/wiki: chapter cascade & documentation generator
│   ├── bin.ts                  # Kaioken CLI unified entrypoint
│   ├── README.md               # Kaioken package documentation
│   └── CHANGELOG.md            # Release log
├── kaioken-pi/                 # Distributable Pi extension package
│   ├── extensions/             # Built runtime extension
│   ├── skills/                 # Agent skill procedures
│   ├── themes/                 # Kaioken dark & light themes
│   └── package.json            # kaioken-pi package manifest
├── bridge/                     # Pi Bridge Source (@kaioken/pi-bridge)
│   ├── commands/               # TUI commands & shortcuts
│   ├── hooks/                  # Grounding hooks & prompt injection
│   ├── prompts/                # Grounding prompts
│   ├── tools/                  # Anti-hallucination tools
│   ├── ui/                     # HUD widgets, command palette & themes
│   └── test/                   # Bridge integration tests
├── .pi/                        # Canonical Pi directory structure
│   ├── extensions/kaioken/     # Extension link path expected by Pi
│   └── themes/                 # kaioken.json & kaioken-light.json
├── .kaioken/                   # Grounded truth store
│   ├── cards/                  # Atomic knowledge cards
│   ├── wiki/                   # Generated wiki chapters & markdown
│   ├── search-index/           # BM25 search index
│   ├── index.json              # Full AST symbol index
│   ├── scan.json               # Repository scan & file discovery
│   ├── risk.json               # Security & entropy risk analysis
│   ├── provenance.json         # File hashes & staleness tracking
│   ├── verification.json       # Verification gates & repair state
│   ├── module-plan.yaml        # Module architecture plan
│   └── wiki-plan.yaml          # Wiki documentation plan
├── scripts/                    # Build, audit & verification scripts
│   ├── build-kaioken-pi.mjs    # Builds the publishable kaioken-pi package
│   ├── check-kaioken-offline.mjs # Verifies core packages make zero network calls
│   ├── check-kaioken-barrels.mjs # Verifies all bridge symbols are exported
│   ├── check-kaioken-pi-package.mjs # Validates packaged artifact integrity
│   ├── check-kaioken-fresh.mjs # Verifies index & provenance freshness
│   └── patch-kaioken-tests.mjs # Test patch utilities
├── docs/                       # Technical documentation
│   ├── phases/                 # Phase 0 to 9 implementation specifications
│   └── roadmap/                # Features 2000 UX & quality roadmap
└── .github/workflows/          # GitHub Actions CI
    └── kaioken.yml             # Offline CI verification workflow
```

---

## Core Invariants

1. **Deterministic Truth**: Core stays 100% offline — scan, symbols, search, serve, status, verify, graph require zero network and zero API keys.
2. **Negative Guarantees**: "Does not exist" is a first-class verified answer.
3. **Verification Gate**: No task is marked "done" without passing native test gates or explicit declaration.
4. **Flash-First Rational**: Skeletons always within token budget; detail expanded on demand.
5. **Radical Cost Transparency**: Multiplier and upfront token spend confirmation.

---

## Scripts & Commands

```bash
# Typecheck and verify zero-network invariant
npm run check:kaioken

# Build TypeScript packages
npm run build:kaioken

# Package kaioken-pi extension bundle
npm run pack:kaioken

# Run offline suite
npm run test:offline

# Run evals gate
npm run eval
```

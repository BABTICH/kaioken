Here are 74 features. #1 to #15 are the ones from my last message.

**Already suggested**

| # | Feature | Effort |
|---|---|---|
| 1 | MCP server (`kaioken mcp`) | 🟡 |
| 2 | LSP server | 🔴 |
| 3 | Semantic search (embeddings) | 🟡 |
| 4 | Secrets and security scan | 🟢 |
| 5 | Test generation | 🟡 |
| 6 | PR and changelog writer | 🟢 |
| 7 | Local models (Ollama / llama.cpp) | 🟡 |
| 8 | Cost and token tracker | 🟢 |
| 9 | Dead code and drift detector | 🟡 |
| 10 | Plugin system | 🔴 |
| 11 | Team shared memory | 🔴 |
| 12 | Onboarding generator | 🟢 |
| 13 | Memory export / import | 🟢 |
| 14 | Packaging, auto-update, shell completions | 🟢 |
| 15 | Benchmark suite | 🟡 |

**Agent and AI**

| # | Feature | What it does | Effort |
|---|---|---|---|
| 16 | Multi-agent orchestration | Planner, coder and reviewer agents working together | 🔴 |
| 17 | Approval gates | The agent shows its plan and waits for your OK before editing | 🟢 |
| 18 | Auto-fix CI | Reads failing CI logs, patches, pushes | 🔴 |
| 19 | Issue-to-PR agent | GitHub issue becomes a branch, a fix and a PR | 🔴 |
| 20 | Project rules file | `.kaioken/rules` for style and guardrails per repo | 🟢 |
| 21 | Smart model routing and fallback | Picks the model by task, cost and latency; falls back on errors | 🟡 |
| 22 | Prompt caching and context compression | Lower cost, bigger effective context | 🟡 |
| 23 | Session resume | `kaioken resume` continues a past session | 🟢 |
| 24 | Checkpoints and undo | Roll back any agent change | 🟡 |
| 25 | Background task queue | Run long tasks while you keep working | 🟡 |
| 26 | Self-critique loop | The agent reviews its own diff before applying it | 🟡 |
| 27 | Natural-language shell | `kaioken do "..."` turns a request into commands | 🟡 |

**Knowledge and memory**

| # | Feature | What it does | Effort |
|---|---|---|---|
| 28 | Architecture diagram generator | Mermaid diagrams from the code | 🟡 |
| 29 | Dependency graph visualizer | Module and import graph | 🟡 |
| 30 | Code ownership map | Who knows which file (git blame) | 🟢 |
| 31 | Hotspot and churn analysis | Files that change and break most | 🟢 |
| 32 | Auto ADRs | Records design decisions from commits and PRs | 🟡 |
| 33 | Glossary extraction | Domain terms and what they mean | 🟡 |
| 34 | API spec extractor | Generates OpenAPI from routes | 🟡 |
| 35 | Cross-repo impact analysis | A change here breaks which other repo | 🔴 |
| 36 | Staleness scoring and pruning | Marks old memories and cleans them | 🟢 |
| 37 | External ingestion | Pull issues, PRs and Slack threads into memory | 🔴 |
| 38 | Time-travel memory | Query what the codebase looked like at any commit | 🟡 |
| 39 | Memory confidence scores | Shows how sure kaioken is about each fact | 🟡 |

**Code quality and security**

| # | Feature | What it does | Effort |
|---|---|---|---|
| 40 | Complexity report | Maintainability score per file | 🟢 |
| 41 | License scanner | Flags incompatible dependency licenses | 🟢 |
| 42 | Dependency update bot | Safe upgrade PRs with impact check | 🟡 |
| 43 | SBOM generation | Software bill of materials | 🟢 |
| 44 | Semver breaking-change check | Detects public API breaks | 🟡 |
| 45 | Migration assistant | Framework and version upgrades | 🔴 |
| 46 | Refactor planner | Safe multi-file refactors with impact map | 🔴 |
| 47 | Performance hints | Spots slow patterns in code | 🟡 |
| 48 | Flaky test detector | Finds unstable tests across runs | 🟡 |
| 49 | Coverage gap analysis | Shows risky code with no tests | 🟡 |

**Integrations**

| # | Feature | What it does | Effort |
|---|---|---|---|
| 50 | VS Code extension | Sidebar for memory, review, impact | 🔴 |
| 51 | JetBrains plugin | Same for IntelliJ family | 🔴 |
| 52 | Neovim plugin | Fits terminal users | 🟡 |
| 53 | GitHub App / PR bot | Automatic review comments on PRs | 🔴 |
| 54 | GitLab and Bitbucket support | Beyond GitHub | 🟡 |
| 55 | Slack / Discord bot | Ask the codebase from chat | 🟡 |
| 56 | Jira / Linear integration | Link tasks to code impact | 🟡 |
| 57 | Browser extension | Explain any GitHub repo page | 🟡 |
| 58 | Obsidian / Notion export | Push memory into note tools | 🟢 |
| 59 | Webhooks and event API | Trigger external actions on events | 🟡 |

**Platform and developer experience**

| # | Feature | What it does | Effort |
|---|---|---|---|
| 60 | Config profiles | Switch work, personal and CI setups | 🟢 |
| 61 | Guided setup wizard | Interactive first-run configuration | 🟢 |
| 62 | Opt-in telemetry and crash reports | Know what breaks for users | 🟢 |
| 63 | HTTP API mode (`kaioken serve`) | Use kaioken as a service | 🟡 |
| 64 | TS and Python SDKs | Embed kaioken in other tools | 🟡 |
| 65 | Docker image and devcontainer | Easy, reproducible setup | 🟢 |
| 66 | Monorepo and workspace support | Scan multiple packages properly | 🟡 |
| 67 | Incremental scan cache | Much faster rescans | 🟡 |
| 68 | Sandboxed execution | Agent commands run in a container | 🔴 |
| 69 | Encrypted key vault | Safe storage for API keys | 🟡 |
| 70 | Localization | French and Arabic UI | 🟢 |

**Business and enterprise**

| # | Feature | What it does | Effort |
|---|---|---|---|
| 71 | Team dashboard and billing portal | Usage, members, invoices for the gateway | 🔴 |
| 72 | SSO and role-based access | Needed for company customers | 🔴 |
| 73 | Self-hosted gateway | Enterprise runs everything privately | 🔴 |
| 74 | Plugin and template marketplace | Community extensions, possible revenue | 🔴 |


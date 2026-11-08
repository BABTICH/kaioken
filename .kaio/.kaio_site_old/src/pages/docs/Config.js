import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import CodeBlock from "@/components/CodeBlock";
import { C, Callout, DocPage, H2, LI, P, UL } from "@/components/docs/parts";
import { PROVIDERS } from "@/data/content";
const NOTES_YAML = `notes:
  - "Real-time features follow the dual-router pattern: REST APIRouter plus a
     sibling ws_router authenticating via short-lived JWT in the token query param."
  - "Every admin mutation must be audit-logged via the AuditLog model."`;
const BLOCKS_YAML = `budget:
  warn_at: 5.00          # USD — warn past this
  hard_stop: 20.00       # USD — refuse to continue

memory:
  learn: 5               # turns before a session distils itself into a skill
  max_skills: 40

search:
  embed_model: local/embedding-v1
  embed_provider: local

compaction:
  reserve_tokens: 8000
  keep_recent_tokens: 24000`;
const GLOBAL_YAML = `default_provider: unified-router
default_engine: reasoning-engine-v2

keys:
  backend: sk-...
  tavily: tvly-...        # web search, for research

research:
  search_provider: tavily
  max_rounds: 4
  max_minutes: 20
  mode: auto              # auto | fast | deep
  verify: false
  max_cost_usd: 3.00

selfupdate:
  enabled: true
  channel: stable
  notify_only: false`;
export default function Config() {
    return (_jsxs(DocPage, { title: "Configuration", lead: "One YAML file per repo, at .kaioken/config.yaml. It holds the model, the scope excludes, and the steering notes injected into every prompt.", children: [_jsx(H2, { id: "create", children: "Creating it" }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "powershell", prompt: true, code: `kaioken init` }) }), _jsxs(P, { children: [_jsx(C, { children: "init" }), " is the whole first run, not just a file: it writes ", _jsx(C, { children: ".kaioken/config.yaml" }), ", scans the repo, and writes ", _jsx(C, { children: "AGENTS.md" }), " \u2014 the instruction file agents read before editing. ", _jsx(C, { children: "-force" }), " rewrites an existing ", _jsx(C, { children: "AGENTS.md" }), ". From inside the TUI,", " ", _jsx(C, { children: "/init" }), " does the same and ", _jsx(C, { children: "/config" }), " shows what is currently active."] }), _jsx(H2, { id: "providers", children: "Backends" }), _jsx(P, { children: "Kaioken is provider-agnostic over any standardized inference endpoint or local offline engine. Switch at runtime and available capabilities refresh dynamically." }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "tui", code: `/provider local-engine
/engine --list
/key                    # hidden prompt, in-memory only` }) }), _jsx("p", { className: "pt-4 font-mono text-[12.5px] text-kai-dim", children: PROVIDERS.join("  ·  ") }), _jsxs(Callout, { kind: "warn", title: "Free tiers", children: ["A tier ending in ", _jsx(C, { children: ":free" }), " caps parallelism at 2, because those tiers rate-limit hard and four parallel calls mostly buys 429s."] }), _jsx(H2, { id: "notes", children: "The steering-notes channel" }), _jsxs(P, { children: ["The most valuable idea borrowed from Qoder's ", _jsx(C, { children: "wiki_plan.yaml" }), ": ", _jsx(C, { children: "notes" }), " in", " ", _jsx(C, { children: "config.yaml" }), " are ", _jsx("strong", { children: "authoritative instructions injected into every prompt" }), " ", "\u2014 use them for tribal knowledge the code does not state."] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "config.yaml", code: NOTES_YAML }) }), _jsxs(P, { children: ["Edit them from the TUI with ", _jsx(C, { children: "/notes add <text>" }), ", review with ", _jsx(C, { children: "/notes" }), ", and wipe with ", _jsx(C, { children: "/notes clear" }), "."] }), _jsx(H2, { id: "routing", children: "Engine routing" }), _jsxs(P, { children: ["Map specific operational roles onto different engines in ", _jsx(C, { children: "config.yaml" }), ". Supported roles are", " ", _jsx(C, { children: "plan" }), ", ", _jsx(C, { children: "edit" }), ", ", _jsx(C, { children: "task" }), ", ", _jsx(C, { children: "compact" }), ", ", _jsx(C, { children: "impact" }), ", and ", _jsx(C, { children: "summarize" }), ". If a role is unset, it falls back to the primary ", _jsx(C, { children: "engine" }), "."] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "config.yaml", code: `models:
  plan: reasoning-engine-v2
  edit: reasoning-engine-v2
  compact: fast-engine-mini
  task: reasoning-engine-v2` }) }), _jsx(H2, { id: "budget", children: "Budgets, memory and search" }), _jsxs(P, { children: ["Three optional blocks turn behaviour that is otherwise implicit into something you set. All of them live in the same per-repo ", _jsx(C, { children: "config.yaml" }), "."] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "config.yaml", code: BLOCKS_YAML }) }), _jsxs(UL, { children: [_jsxs(LI, { children: [_jsx(C, { children: "budget" }), " \u2014 ", _jsx(C, { children: "warn_at" }), " prints a warning past that spend, ", _jsx(C, { children: "hard_stop" }), " refuses to continue. On OpenRouter the real USD figure is available, so this is a guardrail rather than an estimate."] }), _jsxs(LI, { children: [_jsx(C, { children: "memory" }), " \u2014 ", _jsx(C, { children: "learn" }), " is how many turns a session needs before it distils itself into a skill at close; ", _jsx(C, { children: "disable" }), " turns the whole mechanism off, and", " ", _jsx(C, { children: "max_skills" }), " caps how many it may keep."] }), _jsxs(LI, { children: [_jsx(C, { children: "search" }), " \u2014 set ", _jsx(C, { children: "embed_model" }), " (with its own provider and base URL when it differs) and ", _jsx(C, { children: "kaioken index" }), " adds a semantic half to the lexical index. Leave it out and search stays purely offline."] }), _jsxs(LI, { children: [_jsx(C, { children: "compaction" }), " \u2014 ", _jsx(C, { children: "reserve_tokens" }), " and ", _jsx(C, { children: "keep_recent_tokens" }), " tune when context reduction kicks in and how much recent conversation it protects."] })] }), _jsx(H2, { id: "global", children: "The global file" }), _jsxs(P, { children: ["Machine-wide settings live in ", _jsx(C, { children: "~/.kaioken/config.yaml" }), ", separate from any repo: the default provider and model, the per-provider API keys, self-update behaviour, extra local endpoints, and the research defaults."] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "~/.kaioken/config.yaml", code: GLOBAL_YAML }) }), _jsxs(Callout, { kind: "note", title: "Research needs a search key", children: ["The web-search key is separate from the backend key. ", _jsx(C, { children: "tavily" }), ", ", _jsx(C, { children: "firecrawl" }), ",", " ", _jsx(C, { children: "brave" }), " and ", _jsx(C, { children: "exa" }), " are supported; with Firecrawl in the set its scrape API reads the pages too."] }), _jsx(H2, { id: "editable", children: "Files meant to be edited" }), _jsxs(UL, { children: [_jsxs(LI, { children: [_jsx(C, { children: "config.yaml" }), " \u2014 model, scope excludes, steering notes."] }), _jsxs(LI, { children: [_jsx(C, { children: "modules.yaml" }), " \u2014 the proposed module tree. Boundaries are yours to own."] }), _jsxs(LI, { children: [_jsx(C, { children: "wiki_plan.yaml" }), " \u2014 the proposed wiki outline."] }), _jsxs(LI, { children: [_jsx(C, { children: "architecture.md" }), " \u2014 the shared brief and glossary injected into every chapter. Edit it and every chapter inherits the correction on the next run."] })] }), _jsx(H2, { id: "cost", children: "Cost controls" }), _jsxs(UL, { children: [_jsx(LI, { children: "A wiki run prints its estimated calls and tokens before starting." }), _jsx(LI, { children: "Past a threshold it asks for confirmation." }), _jsxs(LI, { children: [_jsx(C, { children: "/cost" }), " shows what the current session has spent; ", _jsx(C, { children: "kaioken usage" }), " aggregates it across sessions by operation, model and workspace."] }), _jsxs(LI, { children: [_jsx(C, { children: "/compact" }), " summarizes the conversation to reclaim context rather than paying for it repeatedly."] }), _jsx(LI, { children: "Low temperature (0.2) throughout \u2014 output should be factual, not creative." })] }), _jsx(H2, { id: "sessions", children: "Sessions" }), _jsxs(P, { children: ["Conversations are saved per repo under ", _jsx(C, { children: ".kaioken/sessions/" }), ". ", _jsx(C, { children: "/sessions" }), " lists them, ", _jsx(C, { children: "/resume <id>" }), " reopens one, and ", _jsx(C, { children: "/new" }), " starts fresh while saving the current one."] })] }));
}

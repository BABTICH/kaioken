export const GITHUB_URL = "https://github.com/babtix/kaioken";
export const DOCS_URL = "/docs";
export const DISCORD_URL = "https://discord.gg/kaioken";
export const X_URL = "https://x.com/babtix";
export const NEWS_URL = "https://kaioken-news.vercel.app";
export const PORTFOLIO_URL = "https://babtich.vercel.app/";
export const TAGLINE = "THE AUTONOMOUS ARCHITECTURAL CODING AGENT";
export const VERSION = "v2.0.0";
export const BUILDER_NAME = "BABTICH EL HABIB";
export const ASCII_LOGO = `██╗  ██╗  █████╗  ██╗  ██████╗  ██╗  ██╗ ███████╗ ███╗   ██╗
██║ ██╔╝ ██╔══██╗ ██║ ██╔═══██╗ ██║ ██╔╝ ██╔════╝ ████╗  ██║
█████╔╝  ███████║ ██║ ██║   ██║ █████╔╝  █████╗   ██╔██╗ ██║
██╔═██╗  ██╔══██║ ██║ ██║   ██║ ██╔═██╗  ██╔══╝   ██║╚██╗██║
██║  ██╗ ██║  ██║ ██║ ╚██████╔╝ ██║  ██╗ ███████╗ ██║ ╚████║
╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═════╝  ╚═╝  ╚═╝ ╚══════╝ ╚═╝  ╚═══╝`;
export const BUILDER_ART = `██████╗  █████╗ ██████╗ ████████╗██╗ ██████╗██╗  ██╗    ███████╗██╗         ██╗  ██╗ █████╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██║██╔════╝██║  ██║    ██╔════╝██║         ██║  ██║██╔══██╗██╔══██╗██║██╔══██╗
██████╔╝███████║██████╔╝   ██║   ██║██║     ███████║    █████╗  ██║         ███████║███████║██████╔╝██║██████╔╝
██╔══██╗██╔══██║██╔══██╗   ██║   ██║██║     ██╔══██║    ██╔══╝  ██║         ██╔══██║██╔══██║██╔══██╗██║██╔══██╗
██████╔╝██║  ██║██████╔╝   ██║   ██║╚██████╗██║  ██║    ███████╗███████╗    ██║  ██║██║  ██║██████╔╝██║██████╔╝
╚═════╝ ╚═╝  ╚═╝╚═════╝    ╚═╝   ╚═╝ ╚═════╝╚═╝  ╚═╝    ╚══════╝╚══════╝    ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚═╝╚═════╝`;
export const INSTALL_COMMANDS = {
    curl: "curl -fsSL https://kaioken.dev/v2/install.sh | bash",
    brew: "brew install babtix/tap/kaioken",
    go: "go install github.com/babtix/kaioken/v2@latest",
    npm: "npm i -g @kaioken/agent",
};
export const PROVIDERS = [
    "Unified Router",
    "Local Offline",
    "Custom Gateway",
    "Private Cluster",
    "Air-Gapped",
];
export const FEATURES = [
    {
        icon: "MessageSquareCode",
        title: "Chat agent",
        description: "Interact with verified code intelligence in your session. It can read, list, search, write and edit files, and run commands — all confined to the target repo.",
        highlights: [
            "Streams token by token, rendered as markdown",
            "Every change lands as a diff you approve",
            "/yolo auto-approves when you trust the run",
        ],
        tone: "blue",
    },
    {
        icon: "BrainCircuit",
        title: "Knowledge engine",
        description: "Scan a repo, split it into modules with the architectural planner, then generate dense knowledge cards per module — with a human-in-the-loop planning step you actually own.",
        highlights: [
            "Fixed five-file card schema per module",
            "modules.yaml is meant to be edited",
            "Content-hash incrementality — unchanged modules are never re-billed",
        ],
        tone: "orange",
    },
    {
        icon: "BookOpenText",
        title: "Deep wiki",
        description: "Multi-pass generation: a global plan, per-section plans, then long-form documents — verified against the code index rather than trusted.",
        highlights: [
            "×3 exhaustive coverage by default",
            "×4+ adds critique-and-revise",
            "×10 corrects every grounding failure found",
        ],
        tone: "amber",
    },
    {
        icon: "Wrench",
        title: "Skills",
        description: "The wiki explains what a codebase contains. A skill explains how to do a task in it — which files to touch, in what order, following which local conventions.",
        highlights: [
            "Agent Skills format, loaded on description match",
            "Procedural, not descriptive",
            "A catalog README the agent reads first",
        ],
        tone: "green",
    },
    {
        icon: "Zap",
        title: "Incremental updates",
        description: "A full wiki run is expensive. Once one completes, update git-diffs the repo against the recorded commit and revises only the documents the diff invalidates.",
        highlights: [
            "Revisions, not rewrites",
            "Provenance footer records real sources",
            "Post-commit hook refreshes automatically",
        ],
        tone: "orange",
    },
    {
        icon: "Globe",
        title: "Wiki server",
        description: "Reading a two-thousand-line chapter in an editor is rough. serve renders the generated wiki as a local site you can actually navigate.",
        highlights: [
            "Sidebar, working links, full-text search",
            "Mermaid diagrams rendered",
            "/serve runs it in the background from the TUI",
        ],
        tone: "amber",
    },
    {
        icon: "ShieldCheck",
        title: "Self-verification loop",
        description: "Detects build/test commands, runs a background agent to diagnose and fix failures, then enforces a Go-native gate so claims are verified.",
        highlights: [
            "Automatic command detection (go.mod, Makefile, package.json)",
            "Background agent attempts up to 3 diagnostic passes",
            "Hard Go verification gate before completion",
        ],
        tone: "blue",
    },
    {
        icon: "GitFork",
        title: "Worktree delegation",
        description: "Delegate sub-tasks to writable sub-agents operating in isolated git worktrees. Changes land in the main repo only when you approve the combined diff.",
        highlights: [
            "Isolated execution in temp git worktree",
            "Per-edit user approval namespace",
            "Diff patch review before landing",
        ],
        tone: "green",
    },
    {
        icon: "FolderGit2",
        title: "Hub & drift poller",
        description: "Track freshness across your entire codebase portfolio using a global cross-repo registry and live working tree drift notification poller.",
        highlights: [
            "Global registry at ~/.kaioken/hub.yaml",
            "CI staleness check via status -check",
            "Live drift polling with kaioken watch",
        ],
        tone: "orange",
    },
];
export const PIPELINE = [
    {
        cmd: "kaioken init",
        title: "Configure",
        body: "Creates .kaioken/config.yaml — review settings, scope excludes, and steering notes.",
    },
    {
        cmd: "kaioken scan",
        title: "Inventory",
        body: "Sanity-check exactly what will be analyzed before execution begins.",
    },
    {
        cmd: "kaioken plan",
        title: "Propose modules",
        body: "The planner proposes modules.yaml. Module boundaries are a judgment call the maintainer should own — so edit it.",
        human: true,
    },
    {
        cmd: "kaioken generate",
        title: "Generate cards",
        body: "Parallel card generation across modules, on the fixed five-file schema.",
    },
    {
        cmd: "kaioken wiki",
        title: "Deep wiki",
        body: "Global plan → per-section plans → long-form docs, at ×3 depth by default. Records the commit it documents.",
    },
    {
        cmd: "kaioken update",
        title: "Stay current",
        body: "git-diffs against that baseline and revises only what the change actually invalidated.",
    },
];
export const COMMAND_GROUPS = [
    {
        id: "knowledge",
        label: "Knowledge",
        icon: "BookOpenText",
        blurb: "Build and refresh everything the engine generates.",
        commands: [
            {
                name: "/research",
                args: "[xN] <question>",
                summary: "Research and ground claims: the router picks the fast loop or the deep multi-agent path, grounds every claim, and writes a cited report to .kaioken/research/.",
            },
            {
                name: "/wiki",
                args: "[xN] [force|update|retry]",
                summary: "DEEP wiki: global plan → per-section plans → long docs. ×3 is the default depth.",
            },
            {
                name: "/update",
                args: "[<base-rev>]",
                summary: "Incremental: git-diff against the commit the wiki was built from, and revise only the invalidated documents.",
            },
            { name: "/skills", args: "[force|list|name]", summary: "Build task guides an agent loads while working here." },
            {
                name: "/impact",
                args: "<change description>",
                summary: "Predict the blast radius of a change before editing: symbols, files, modules, stale docs and skills, tests to re-run — each claim checked against the symbol index.",
            },
            { name: "/publish", args: "[-out <dir>]", summary: "Render the wiki as a static HTML site — no server needed." },
            { name: "/pack", args: "[-out <file>]", summary: "Bundle knowledge into a portable .tar.gz archive." },
            { name: "/onboard", args: "[force]", summary: "Write ONBOARDING.md assembled from wiki, cards, and skills." },
            { name: "/scan", summary: "Scan the repo, print an inventory." },
            { name: "/plan", summary: "Planner proposes modules.yaml — edit it, then generate." },
            { name: "/cards", args: "[force|id]", summary: "Generate knowledge cards — all, one module, or force a rebuild." },
            { name: "/status", summary: "Per-module freshness." },
            { name: "/wiki retry", summary: "Regenerate only the sections that failed last run." },
        ],
    },
    {
        id: "agent",
        label: "Agent control",
        icon: "Waypoints",
        blurb: "How much the agent may do, how you steer it, and how a wrong turn gets rewound.",
        commands: [
            {
                name: "/mode",
                args: "[build|plan|general|explore|review|prism]",
                summary: "Switch the permission mode: build is full access, plan/explore/review/prism are read-only (prism automatically grounds answers in imported documents), and general keeps every tool but always asks first.",
            },
            { name: "/undo", summary: "Roll back the last applied change." },
            { name: "/diff", summary: "Review the changes the agent has proposed or applied." },
            { name: "/compact", summary: "Summarize the conversation to reclaim context." },
            { name: "/yolo", summary: "Auto-approve file changes and commands for this session." },
            { name: "/thinking", args: "[off|low|medium|high]", summary: "Set how much reasoning budget is spent before answering." },
            { name: "/stop", summary: "Cancel whatever is in flight; streamed text is kept." },
            { name: "/queue", args: "[clear]", summary: "Type while it works and the message is queued as steering — this shows or drops what is waiting." },
            { name: "/fork", args: "[turns]", summary: "Rewind the last N user turns and retry differently — the old turns stay as a branch." },
            { name: "/tree", args: "[n [summarize]]", summary: "List conversation branches and switch between them." },
            { name: "/verify", summary: "Run build/test commands and auto-fix failures, then re-run them in plain Go as the gate." },
            { name: "/templates", summary: "List prompt templates — /t:<name> key=value expands and sends one." },
            { name: "/learn", summary: "Distill this session into a skill the agent loads next time." },
        ],
    },
    {
        id: "repo",
        label: "Repo & session",
        icon: "FolderGit2",
        blurb: "Point at code, browse output, keep conversations.",
        commands: [
            { name: "/new", summary: "Start a fresh session — the current one is saved." },
            { name: "/sessions", summary: "List saved conversations." },
            { name: "/resume", args: "[id]", summary: "Reopen a saved conversation — no id opens a searchable picker." },
            { name: "/switch", args: "[id]", summary: "Save this session, then open another." },
            { name: "/notes", args: "[add <t>|clear]", summary: "View or edit steering notes injected into prompts." },
            { name: "/init", args: "[force]", summary: "Full first-run setup: config.yaml, a scan, and AGENTS.md." },
            { name: "/draft", args: "[base]", summary: "Draft conventional commit message + PR description grounded in diff." },
            { name: "/import", args: "<path>", summary: "Bring an external transcript in as a new session." },
            { name: "/repo", args: "<path>", summary: "Point at a different repository." },
            { name: "/handoff", args: "[session-id]", summary: "Write a continuation briefing from a saved session." },
            { name: "/hook", args: "[install|remove]", summary: "Refresh the wiki automatically after every commit." },
            { name: "/serve", args: "[port]", summary: "Browse the generated wiki in a browser · /serve stop." },
            { name: "/hub", args: "[list|add|remove|status]", summary: "Cross-repo registry — track and check freshness across multiple repos." },
        ],
    },
    {
        id: "model",
        label: "Engines & runtime",
        icon: "Cpu",
        blurb: "Multi-backend support, private endpoints, local hardware inference.",
        commands: [
            { name: "/engine", args: "[id|list]", summary: "Set the active engine — no id opens the picker." },
            { name: "/cost", summary: "Calls, tokens and spend so far — budget.warn_at turns it into a guardrail." },
            { name: "/key", args: "[value]", summary: "Set API key in-memory — blank opens a hidden prompt." },
            { name: "/provider", args: "[name|list]", summary: "Switch execution backend." },
            { name: "/config", summary: "Show active configuration and per-operation roles." },
            { name: "/session", summary: "Stats for the current session — length, context used." },
            { name: "/theme", args: "[default|light|highcontrast]", summary: "Switch the colour palette." },
        ],
    },
    {
        id: "extensions",
        label: "Extensions",
        icon: "Puzzle",
        blurb: "Community packages installed from GitHub releases, inert until you trust them.",
        commands: [
            {
                name: "/ext",
                args: "[browse|install|remove|update|search|trust|…]",
                summary: "Manage extensions. Declarative ones only contribute skills; mcp and wasm ones stay inert until /ext trust shows you exactly what they would run.",
            },
            { name: "/x", args: "[ext command [args]]", summary: "Run a command a trusted wasm extension contributed, in its sandbox." },
        ],
    },
    {
        id: "session",
        label: "Learning & utility",
        icon: "Terminal",
        blurb: "The manual is in the app, plus the small commands you reach for mid-run.",
        commands: [
            { name: "/help", summary: "List every command." },
            { name: "/clear", summary: "Clear the screen — the conversation is untouched." },
            { name: "/copy", summary: "Copy the last reply to the clipboard." },
            { name: "/tutorial", args: "[chapter|command]", summary: "Guided walkthrough — an overview, a chapter, or a single command." },
            { name: "/explain", args: "[command|all]", summary: "The full reference page for one command, or the whole manual." },
            { name: "/version", summary: "Version, Go build and platform." },
            { name: "/quit", summary: "Exit the TUI." },
        ],
    },
];
export const MULTIPLIER = [
    { level: "×1", behavior: "Public surface and main flow, section docs only." },
    { level: "×2", behavior: "Adds subsection documents and diagrams." },
    { level: "×3", behavior: "Exhaustive coverage of every declaration in scope.", isDefault: true },
    {
        level: "×4+",
        behavior: "Adds a critique-and-revise cycle: score the draft against a rubric, fix the gaps, cut the padding.",
    },
    { level: "×10", behavior: "Additionally corrects every grounding failure verification found." },
];
export const QUALITY = [
    {
        icon: "ScanText",
        claim: "The engine sees whole files, not their edges",
        body: "internal/codemap parses every source file into a skeleton — package, imports, and every declaration with its line range. Prompts get every declaration in scope for a few hundred tokens per file, plus full bodies of the most relevant code. Files too large to include whole contribute complete functions, never arbitrary byte slices.",
    },
    {
        icon: "GitMerge",
        claim: "Chapters agree with each other",
        body: "Sections generate in parallel and can only see sibling titles, which produces the same concept explained three times in three vocabularies. A prior pass writes an authoritative brief — architecture, key flows, a glossary of canonical terms — to .kaioken/architecture.md, injected verbatim into every later prompt.",
    },
    {
        icon: "ShieldCheck",
        claim: "Claims are checked, not trusted",
        body: "Every file path, symbol, line anchor and quoted excerpt a document asserts is checked against the code index. Excerpts must actually appear at the lines they cite. A prompt asking for no hallucinations is a request; checking every claim is a guarantee.",
    },
    {
        icon: "Ruler",
        claim: "Length follows substance",
        body: "Line-count targets create pad, so the prompt specifies coverage instead: every exported declaration, endpoint, config key and model in scope must be documented. A shorter chapter that covers everything beats a longer one that repeats itself.",
    },
    {
        icon: "Boxes",
        claim: "Framework facts are extracted, not guessed",
        body: "Real routes, models, CLI commands and environment variables are pulled from the source — Express, FastAPI, Flask, Spring, Gin, Rails and friends — so an API reference lists actual endpoints.",
    },
    {
        icon: "ClipboardCheck",
        claim: "The plan is validated",
        body: "Planning sees a structural skeleton of the repo, not just a directory listing. Kaioken then reports what percentage of scanned files the plan claims, and which directories the misses cluster in — a plan that ignores a third of the codebase is visible before generation spends tokens on it.",
    },
];
export const DESIGN_DECISIONS = [
    {
        title: "Human-in-the-loop plan",
        body: "modules.yaml is meant to be edited — module boundaries are a judgment call the maintainer should own.",
    },
    {
        title: "Fixed card schema",
        body: "Agents can rely on the same five files existing for every module.",
    },
    {
        title: "Content-hash incrementality",
        body: "state.json stores a sha256 over each module's scoped files; unchanged modules are never re-billed.",
    },
    {
        title: "Structure before text",
        body: "A file's skeleton always fits the budget, so nothing in scope is ever invisible — detail is what gets rationed, never coverage.",
    },
    {
        title: "Provenance over prose",
        body: "Documents record their sources in a machine-readable footer, so incremental updates do not depend on writing a tidy Referenced Files section.",
    },
    {
        title: "Low temperature (0.2)",
        body: "Cards should be factual, not creative.",
    },
    {
        title: "Hard Go verification gate",
        body: "The claim that code passes is verified by running detected build/test commands natively before exit.",
    },
    {
        title: "Worktree sub-agent isolation",
        body: "Sub-agents run in temporary git worktrees, preventing untrusted draft edits from corrupting your working tree.",
    },
];
export const OUTPUT_TREE = [
    { name: ".kaioken/", depth: 0, kind: "dir" },
    { name: "config.yaml", note: "scope excludes, per-operation roles, steering notes", depth: 1, kind: "edit" },
    { name: "modules.yaml", note: "planner-proposed module tree — EDIT before generating", depth: 1, kind: "edit" },
    { name: "wiki_plan.yaml", note: "planner-proposed wiki outline — EDIT before generating", depth: 1, kind: "edit" },
    { name: "architecture.md", note: "shared brief + glossary injected into every chapter", depth: 1, kind: "edit" },
    { name: "wiki_state.yaml", note: "the commit the wiki reflects (+ failed sections)", depth: 1, kind: "file" },
    { name: "state.json", note: "per-module source hashes → incremental updates", depth: 1, kind: "file" },
    { name: "risk.json", note: "scan risk flags (secrets, credential files, large binaries)", depth: 1, kind: "file" },
    { name: "search_index.json", note: "lexical (+ optional vector) index behind kaioken search", depth: 1, kind: "file" },
    { name: "site/", note: "static HTML wiki export generated by /publish", depth: 1, kind: "dir" },
    { name: "sessions/", note: "saved chat conversations + digests → /resume, recall", depth: 1, kind: "dir" },
    { name: "handoffs/", note: "continuation briefings generated by /handoff", depth: 1, kind: "dir" },
    { name: "research/", note: "cited reports from /research — markdown, json, and PDF for deep runs", depth: 1, kind: "dir" },
    { name: "impact/", note: "saved blast-radius predictions, scored by impact -compare", depth: 1, kind: "dir" },
    { name: "templates/", note: "parameterized prompts sent with /t:<name>", depth: 1, kind: "edit" },
    { name: "skills/", note: "task guides an agent loads while working", depth: 1, kind: "dir" },
    { name: "wiki/", note: "the deep wiki: one folder per section, plus CHANGELOG.md", depth: 1, kind: "dir" },
    { name: "KNOWLEDGE.md", note: "index an agent reads first", depth: 1, kind: "file" },
    { name: "knowledge/<module>/", depth: 1, kind: "dir" },
    { name: "_module.yaml", note: "metadata (scope, generated_at)", depth: 2, kind: "file" },
    { name: "overview.md", depth: 2, kind: "file" },
    { name: "architecture.md", depth: 2, kind: "file" },
    { name: "conventions.md", depth: 2, kind: "file" },
    { name: "tech_stack.md", depth: 2, kind: "file" },
    { name: "setup_commands.md", note: "only when the module has unique commands", depth: 2, kind: "file" },
];
export const QUICK_START = `# build (Go >= 1.24)
cd cli
go build -o kaioken.exe ./cmd/kaioken

# set your API key
$env:KAIOKEN_API_KEY = "sk-..."

cd path\\to\\your\\repo
kaioken init                 # creates .kaioken/config.yaml
kaioken scan                 # sanity-check what will be analyzed
kaioken plan                 # planner proposes modules.yaml — edit it if you like
kaioken generate             # parallel card generation
kaioken wiki                 # deep multi-pass wiki (×3 by default)
kaioken update               # after code changes: git-diff-driven refresh`;
export const ROADMAP = [
    "Specialist sub-agents: a refactor agent, a test writer, a debug agent",
    "A migration agent that upgrades dependencies and applies codemods",
    "Card self-iteration — cards that revise themselves as the module drifts",
    "More export targets beyond agent-md, context-md, and repo-wiki",
];
export const RESEARCH = [
    {
        title: "One question, one command",
        body: "kaioken research \"…\" (or /research in the TUI) — searches and analyzes your repository and verified sources.",
        tone: "orange",
    },
    {
        title: "A router sizes it up first",
        body: "A routing call decides the shape of the run: a quick lookup takes the fast path, a multi-part question earns the deep one. You can pin either with -mode.",
        tone: "amber",
    },
    {
        title: "Fast path — one tight loop",
        body: "Search, read, reason, repeat until the gaps close. Cheap, quick, and predictable.",
        tone: "orange",
    },
    {
        title: "Deep path — parallel workers",
        body: "A supervisor splits the question into subquestions and hands each to an isolated worker that researches and compresses its own findings.",
        tone: "amber",
    },
    {
        title: "Grounded before it ships",
        body: "Every claim is tied to a numbered source that was actually read, and a separate pass checks the draft against the raw text.",
        tone: "orange",
    },
];
export const RESEARCH_EXAMPLE = `kaioken research "what changed in Go 1.24 GC?"
kaioken research x3 -mode deep "compare OSS auth designs"

/research x2 is solar cheaper than nuclear?

# closed the terminal mid-run? continue it:
kaioken research -resume <run id>

# -verify cross-checks load-bearing claims
# reports land in .kaioken/research/`;
export const RESEARCH_NOTES = [
    "A fast run that turns out thin escalates to deep — reusing everything it already fetched instead of restarting.",
    "Every fetched source is treated as data, never instructions, and is sanitised before execution.",
    "State checkpoints to disk at every phase, so an interrupted run resumes where it left off.",
    "Cost stays line-itemised: searches, pages read and tokens — one honest number at the end.",
];

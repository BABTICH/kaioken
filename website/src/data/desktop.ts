/**
 * Kaioken Desktop — content for /desktop.
 *
 * Every claim here is read off the shipped Rust app rather than imagined:
 * routes and shortcuts from kaioRust/src/desktop/ui/app.js, window facts from
 * kaioRust/src/desktop/gui.rs, the wire protocol from protocol.rs/ipc.rs, and
 * the memory model from the watcher + SQLite engine.
 */

export const DESKTOP_REPO_PATH = "https://github.com/BABTICH/kaioken/tree/master/kaioRust/src/desktop"

/**
 * The /desktop wordmark, from website/DESKTOP APP-logo.html.
 *
 * The source art sets DESKTOP and APP side by side, 97 columns wide — which on
 * a phone means 6px type. Stacked, the widest block is 64 columns, so the same
 * logo reads at 9px in a 375px viewport. One string rather than two so the
 * red→orange sweep runs across both blocks instead of restarting.
 *
 * The D's top row starts at column 0: the two spaces in front of it in the
 * source file are that file's own HTML indentation, not part of the letter.
 * The A's single leading space is real — that is the ANSI Shadow letterform.
 */
export const DESKTOP_ART = `██████╗  ███████╗ ███████╗ ██╗  ██╗ ████████╗  ██████╗  ██████╗
██╔══██╗ ██╔════╝ ██╔════╝ ██║ ██╔╝ ╚══██╔══╝ ██╔═══██╗ ██╔══██╗
██║  ██║ █████╗   ███████╗ █████╔╝     ██║    ██║   ██║ ██████╔╝
██║  ██║ ██╔══╝   ╚════██║ ██╔═██╗     ██║    ██║   ██║ ██╔═══╝
██████╔╝ ███████╗ ███████║ ██║  ██╗    ██║    ╚██████╔╝ ██║
╚═════╝  ╚══════╝ ╚══════╝ ╚═╝  ╚═╝    ╚═╝     ╚═════╝  ╚═╝
 █████╗  ██████╗  ██████╗
██╔══██╗ ██╔══██╗ ██╔══██╗
███████║ ██████╔╝ ██████╔╝
██╔══██║ ██╔═══╝  ██╔═══╝
██║  ██║ ██║      ██║
╚═╝  ╚═╝ ╚═╝      ╚═╝`

/* ── the window itself ──────────────────────────────────────────────────── */

export const WINDOW = {
  /** tao window configuration in src/desktop/gui.rs */
  width: 1360,
  height: 880,
  minWidth: 1080,
  minHeight: 680,
  /** decorations:false — the title bar is drawn by the app, not the OS */
  customChrome: true,
  theme: "Dark + Light",
  /** design.css resolves the cockpit to ink-950 */
  background: "#0a0a0b",
}

/* ── surfaces (the nav rail, in rail order) ─────────────────────────────── */

export interface Surface {
  /** lucide-react icon name, resolved through components/Icon */
  icon: string
  label: string
  /** the keyboard shortcut, empty when the route has none */
  key: string
  headline: string
  body: string
  /** three concrete things the screen actually does */
  points: string[]
  tone: "orange" | "amber" | "blue" | "green" | "sage"
}

export const SURFACES: Surface[] = [
  {
    icon: "MessageSquare",
    label: "Chat",
    key: "Ctrl+1",
    headline: "The agent, with a deny-first approval card",
    body: "Streaming thoughts, tool results and session history in one thread. Every proposed tool call arrives as a card with its policy tier, target and diff — Enter approves, Escape denies, and dismissing is never authorization.",
    points: [
      "Sessions switch from the sidebar; compact a long conversation in place",
      "WRITE_PROMPT and DANGEROUS proposals wait for an explicit decision",
      "Auto-approve and allow-shell are per-turn toggles, off by default",
    ],
    tone: "blue",
  },
  {
    icon: "Radar",
    label: "Research",
    key: "Ctrl+2",
    headline: "Ask the web, get a cited dossier",
    body: "The power meter is the actual budget: ×1–×9 stay on the normal path, ×10 runs the advanced dossier. The run streams its steps, keeps a receipt, and lands every source as a chip beside the answer.",
    points: [
      "Live step list with queries, pages read and grounded claims",
      "Run receipt records path, rounds, duration and cost",
      "Past research reopens from history without starting over",
    ],
    tone: "amber",
  },
  {
    icon: "BookOpen",
    label: "Wiki",
    key: "Ctrl+3",
    headline: "Read the generated wiki in place",
    body: "A grouped chapter tree, search, table of contents and rendered markdown with provenance chips — no separate serve process to start and lose behind the IDE.",
    points: [
      "Chapters grouped by section, with reading time per page",
      "TOC follows the active heading; code fences keep their language label",
      "Docs health shows total, fresh and stale counts",
    ],
    tone: "orange",
  },
  {
    icon: "Waypoints",
    label: "Graph",
    key: "Ctrl+4",
    headline: "See the AST, not a guess",
    body: "The symbol topology held in .agent/memory.sqlite: functions, structs, enums and traits, with caller and dependency edges. Filter, zoom, then read deterministic documentation for the node you picked.",
    points: [
      "Kind filters and search dim everything irrelevant",
      "Inspector lists callers, dependencies and Level N docs",
      "Every node carries its file path and caller count",
    ],
    tone: "sage",
  },
  {
    icon: "Layers",
    label: "Cards",
    key: "Ctrl+5",
    headline: "Failure memory, card by card",
    body: "Every critic failure becomes a card: severity, occurrences, gotchas and anti-pattern rules. The rules the agent retrieves before it repeats the mistake are the same ones shown here.",
    points: [
      "Filter by severity or what is still being watched",
      "Detail names the module, last sighting and prevention rule",
      "Cards accumulate from critic runs, not hand-written notes",
    ],
    tone: "green",
  },
  {
    icon: "Code2",
    label: "Editor",
    key: "Ctrl+6",
    headline: "Code, diff, and a terminal rail",
    body: "Open files in tabs beside the tree. Flip between code and a syntax-highlighted diff, read the review risk score, and drop a terminal drawer underneath — two terminals, one window.",
    points: [
      "Tabs and tree share dirty markers and close controls",
      "Code / Diff segmented view with language and path chips",
      "Ctrl+S saves; Ctrl+` opens a real terminal, not a log view",
    ],
    tone: "blue",
  },
  {
    icon: "Globe",
    label: "Browser",
    key: "Ctrl+7",
    headline: "Tabs, in the app",
    body: "A tabbed surface for docs and web pages, plus an API inspector for the loopback daemon: method, status, latency, headers and JSON body. Pages that refuse framing open in the system browser instead.",
    points: [
      "Omnibox with back, forward, reload and external open",
      "API inspector captures request and response side by side",
      "New-tab page keeps project destinations one click away",
    ],
    tone: "amber",
  },
  {
    icon: "Zap",
    label: "Activity",
    key: "Ctrl+8",
    headline: "Runs, events, one ledger",
    body: "Start a wiki, cards, skills, scan, update or doctor run and watch it stream. The same panel shows memory learning stats, the safety audit ledger and which daemon the app is talking to.",
    points: [
      "Pipeline strip runs init → scan → plan → generate → wiki → update",
      "Depth slider from ×1 fast to ×10 exhaustive",
      "Live per-run progress and logs; history keeps the outcome",
    ],
    tone: "orange",
  },
  {
    icon: "Puzzle",
    label: "Extensions",
    key: "Ctrl+9",
    headline: "Capabilities you opt into",
    body: "Four tabs — installed packages, the built-in tool registry, MCP servers and hooks, and the registry. Executable packages stay inert until you trust that exact installed version.",
    points: [
      "Seven built-in tools across READ_ONLY, WRITE_PROMPT and DANGEROUS",
      "Trust is version-pinned; an update asks again",
      "Install from owner/repo@version, one install per machine",
    ],
    tone: "sage",
  },
  {
    icon: "Wallet",
    label: "Cost",
    key: "",
    headline: "Where the money went",
    body: "Total spend, calls, tokens and local calls across 7, 30, 90 days or all time, scoped to one repo or every repo. Breakdowns by operation, model, provider and workspace sit above a daily bar chart.",
    points: [
      "Provider-reported and catalog-estimated spend are separated",
      "Pricing catalog freshness is stated, not implied",
      "Per-workspace bars answer which repo is expensive",
    ],
    tone: "green",
  },
  {
    icon: "Settings",
    label: "Settings",
    key: "Ctrl+,",
    headline: "One config, one daemon",
    body: "Permission mode, model and base URL, research provider and page reader, API key presence, daemon mode and watcher debounce, and theme — saved back to the daemon configuration.",
    points: [
      "Read only, edit with approval, or full auto",
      "Keys are never echoed back; the UI shows presence only",
      "Embedded or daemon mode, with endpoint and debounce",
    ],
    tone: "blue",
  },
]

/* ── architecture ───────────────────────────────────────────────────────── */

export interface Layer {
  id: string
  title: string
  subtitle: string
  detail: string
  tone: "orange" | "amber" | "blue" | "green"
  /** what the layer is made of, shown as small tags */
  parts: string[]
}

export const LAYERS: Layer[] = [
  {
    id: "cockpit",
    title: "WebView cockpit",
    subtitle: "the surface",
    detail:
      "Vanilla HTML, CSS and an ES state machine, compiled into the binary with include_str!. No React, no bundler, no dev server. Rust pushes a full snapshot; the UI deep-merges it by revision and keeps selection, drafts and toasts.",
    tone: "blue",
    parts: ["HTML/CSS/ES", "include_str!", "snapshot merge", "dark + light"],
  },
  {
    id: "host",
    title: "tao + wry host",
    subtitle: "the native shell",
    detail:
      "tao owns the window and event loop; wry embeds the WebView the OS already ships — WebView2, WKWebView or WebKitGTK. Borderless chrome, drag, 5px resize hit-testing, minimize and maximize are Rust events, not browser chrome.",
    tone: "orange",
    parts: ["tao 0.28", "wry 0.42", "WebView2 / WKWebView", "borderless chrome"],
  },
  {
    id: "daemon",
    title: "kaioken watch",
    subtitle: "the shared daemon",
    detail:
      "A long-running Rust process that watches the workspace, re-parses changed ASTs and serves line-delimited JSON-RPC on 127.0.0.1:47890. Requests carry an id and a 10s deadline; FileChanged, MemoryUpdated and Heartbeat arrive as broadcast events. No daemon? The desktop starts an embedded runtime and says so.",
    tone: "amber",
    parts: ["127.0.0.1:47890", "JSON-RPC", "broadcast events", "embedded fallback"],
  },
  {
    id: "disk",
    title: "SQLite memory",
    subtitle: "one source of truth",
    detail:
      "Tree-sitter parses Rust, TypeScript/JS and Python; SHA-256 hashes skip unchanged files. .agent/memory.sqlite runs in WAL mode and holds files, symbols, edges, docs, failures and audit entries — the state the desktop, TUI and CLI all read.",
    tone: "green",
    parts: [".agent/memory.sqlite", "WAL", "tree-sitter", "SHA-256"],
  },
]

/* ── what the GUI wins, and what it does not ────────────────────────────── */

export interface Comparison {
  job: string
  tui: string
  desktop: string
  /** which surface actually wins this row */
  winner: "tui" | "desktop" | "tie"
}

export const COMPARISON: Comparison[] = [
  {
    job: "Chatting with the agent",
    tui: "Streams in the terminal",
    desktop: "The same agent, rendered markdown",
    winner: "tie",
  },
  {
    job: "Approving a write",
    tui: "y/n over a unified patch",
    desktop: "Tiered card, diff, Enter or Escape",
    winner: "desktop",
  },
  {
    job: "Reading a wiki chapter",
    tui: "Not possible — use the preview",
    desktop: "Tree, search and TOC in-app",
    winner: "desktop",
  },
  {
    job: "Inspecting the symbol graph",
    tui: "Memory commands only",
    desktop: "Topology, callers and dependencies",
    winner: "desktop",
  },
  {
    job: "Editing files between passes",
    tui: "Alt-tab to an editor",
    desktop: "Tabs, diff mode, save and terminal",
    winner: "desktop",
  },
  {
    job: "Watching a long run",
    tui: "One foreground pipeline",
    desktop: "Live progress, logs and history",
    winner: "desktop",
  },
  {
    job: "Knowing what you spent",
    tui: "A usage command, when you remember",
    desktop: "Cost dashboard and status bar",
    winner: "desktop",
  },
  {
    job: "Working over SSH on a box with no GUI",
    tui: "Exactly what it is for",
    desktop: "Wrong tool — use the TUI",
    winner: "tui",
  },
  {
    job: "Scripting it in CI",
    tui: "The binary and its subcommands",
    desktop: "Not the point",
    winner: "tui",
  },
]

/* ── principles ─────────────────────────────────────────────────────────── */

export const PRINCIPLES: { icon: string; title: string; body: string }[] = [
  {
    icon: "Cpu",
    title: "Native, not bundled",
    body: "The window is the OS WebView driven by Rust. No Electron, no Chromium download, no Node runtime and no npm install.",
  },
  {
    icon: "GitMerge",
    title: "One state model",
    body: "Rust owns the snapshot; the WebView merges it by revision. A state push never reloads the page, so selection and drafts survive.",
  },
  {
    icon: "ShieldCheck",
    title: "Deny-first approvals",
    body: "Writes and shell commands arrive as proposals. Escape denies, dismissal is never authorization, and DANGEROUS asks even under full-auto.",
  },
  {
    icon: "Plug",
    title: "Loopback only",
    body: "The daemon binds 127.0.0.1, never a public interface. The desktop shares that socket and the same SQLite memory as the TUI.",
  },
]

/* ── shortcuts (mirrors kaioRust/src/desktop/ui/app.js) ─────────────────── */

export const SHORTCUT_GROUPS: { group: string; items: { keys: string; label: string }[] }[] = [
  {
    group: "General",
    items: [
      { keys: "Ctrl+K", label: "Command palette" },
      { keys: "Ctrl+P", label: "Quick open file" },
      { keys: "Ctrl+Shift+S", label: "Run codebase scan" },
      { keys: "Ctrl+Shift+D", label: "Run environment doctor" },
      { keys: "?", label: "Shortcut help" },
      { keys: "Esc", label: "Close dialog or deny tool" },
    ],
  },
  {
    group: "Navigation",
    items: [
      { keys: "Ctrl+1…9", label: "Jump to a surface" },
      { keys: "Ctrl+,", label: "Settings" },
    ],
  },
  {
    group: "Chat & Agent",
    items: [
      { keys: "Ctrl+N", label: "New chat session" },
      { keys: "Ctrl+Shift+C", label: "Compact conversation" },
      { keys: "Enter", label: "Send message / approve proposal" },
      { keys: "Alt+Enter", label: "New line" },
      { keys: "Esc", label: "Deny proposal" },
    ],
  },
  {
    group: "Editor & Terminal",
    items: [
      { keys: "Ctrl+S", label: "Save file" },
      { keys: "Ctrl+`", label: "Toggle terminal" },
    ],
  },
  {
    group: "Browser",
    items: [
      { keys: "Ctrl+L", label: "Focus address bar" },
      { keys: "Ctrl+T", label: "New tab" },
      { keys: "Ctrl+W", label: "Close tab" },
    ],
  },
  {
    group: "Research",
    items: [
      { keys: "×1–×9", label: "Normal budget" },
      { keys: "×10", label: "Advanced dossier" },
      { keys: "Enter", label: "Start research" },
      { keys: "Esc", label: "Stop or close" },
    ],
  },
]

/* ── platforms and building ─────────────────────────────────────────────── */

export interface Platform {
  id: "windows" | "macos" | "linux"
  label: string
  /** the system WebView each platform provides */
  artifacts: string
  note: string
}

export const PLATFORMS: Platform[] = [
  {
    id: "windows",
    label: "Windows",
    artifacts: "WebView2 · Edge runtime",
    note: "Borderless window with tao-managed drag, resize and maximize. WebView2 is already present on Windows 10/11.",
  },
  {
    id: "macos",
    label: "macOS",
    artifacts: "WKWebView · universal binary",
    note: "System WebView, dark and light palettes, Apple Silicon and Intel from one cargo build.",
  },
  {
    id: "linux",
    label: "Linux",
    artifacts: "WebKitGTK 4.1",
    note: "Needs libwebkit2gtk-4.1. AppImage and .deb packaging are not wired yet, so run the release binary.",
  },
]

/** Honest status of distribution — implemented, still growing. */
export const DISTRIBUTION_NOTE =
  "Kaioken Desktop is implemented in Phase 24 and ships inside the Rust binary. The window, IPC, embedded fallback, approval flow, memory graph, diff review and docs health are live; chat history, research, wiki, extensions and cost panels still render seeded showcase data until their telemetry backends are wired to IPC. There are no signed installers yet — build from source and run kaioken desktop."

export const BUILD_STEPS = `# prerequisites: Rust stable + the OS WebView runtime
# Windows: WebView2 · macOS: WKWebView · Linux: libwebkit2gtk-4.1

git clone https://github.com/BABTICH/kaioken
cd kaioken/kaioRust

cargo run -- watch --daemon --ipc-port 47890   # optional shared daemon
cargo run -- desktop --tab chat                 # connect, or fall back embedded
cargo run -- desktop --embedded --tab graph

cargo build --release                           # target/release/kaioken`

export const RUN_PROOF = `# one binary: the daemon, the window, the terminal UI
kaioken watch --daemon
#   JSON-RPC on 127.0.0.1:47890
#   FileChanged · MemoryUpdated · Heartbeat

kaioken desktop
#   probes the daemon for 1.5s
#   no answer? embedded runtime, with a warning

kaioken desktop --headless
#   prints the cockpit as a terminal frame`

/* ── numbers worth stating once ─────────────────────────────────────────── */

export const DESKTOP_STATS: { value: string; label: string }[] = [
  { value: "11", label: "surfaces" },
  { value: "1", label: "Rust binary" },
  { value: "0", label: "Node runtimes" },
  { value: "3", label: "WebView backends" },
]

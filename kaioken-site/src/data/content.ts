export interface FeatureItem {
  id: string;
  num: string;
  title: string;
  image: string;
  description: string;
  badge?: string;
}

export const NAV_LINKS = [
  { label: 'Babtix', href: 'https://github.com/babtix' },
  { label: 'Docs', href: 'https://github.com/babtix/kaioken#readme' },
  { label: 'Wiki', href: '#preview' },
];

export const SOCIAL_LINKS = {
  discord: 'https://discord.gg/nousresearch',
  x: 'https://x.com/babtix',
  github: 'https://github.com/babtix/kaioken',
};

export const TERMINAL_COMMANDS = {
  macLinux: 'curl -fsSL https://kaioken.dev/install.sh | bash',
  windows: 'irm https://kaioken.dev/install.ps1 | iex',
  npm: 'npm install -g @kaioken/cli',
};

export const FEATURES: FeatureItem[] = [
  {
    id: 'ingest',
    num: '#1 INGEST',
    title: 'AST Code Parser',
    image: '/assets/feature-ingest.jpg',
    description:
      'Tree-sitter structural AST indexing across TypeScript, Go, Python, and Rust. Declarations and symbol skeletons are cached with definitive zero-token lookups.',
  },
  {
    id: 'remember',
    num: '#2 REMEMBER',
    title: 'Content-Hash Memory',
    image: '/assets/feature-remember.jpg',
    description:
      'SHA-256 provenance tracks every declaration. Invalidation is strictly governed by source file AST drift, eliminating documentation rot forever.',
  },
  {
    id: 'multiply',
    num: '#3 MULTIPLY',
    title: 'Kaioken Multiplier',
    image: '/assets/feature-multiplier.jpg',
    description:
      'Dial compute intensity from ×1 to ×10. Controls search breadth, crawl depth, token budgets, and adversarial verification passes with total cost transparency.',
  },
  {
    id: 'verify',
    num: '#4 VERIFY',
    title: 'Hard Grounding Gate',
    image: '/assets/feature-verify.jpg',
    description:
      'Claims are checked, never trusted. The SymbolOracle validates existence, line anchors, and runs native npm/go/cargo test suites before completion.',
  },
  {
    id: 'research',
    num: '#5 RESEARCH',
    title: 'Deep Autonomous Web',
    image: '/assets/feature-research.jpg',
    description:
      'Multi-hop web research agents search, extract, sanitize, and verify external sources, anchoring citations against live cryptographic page hashes.',
  },
  {
    id: 'delegate',
    num: '#6 DELEGATE',
    title: 'Subagent Swarms',
    image: '/assets/feature-delegate.jpg',
    description:
      'Isolated subagents with dedicated execution terminals, sandbox namespaces, and zero-context-cost pipeline RPC protocols for massive scale.',
  },
];

export const OS_DOWNLOADS = [
  {
    os: 'Mac OS',
    version: 'macOS 12+ • Apple Silicon & Intel',
    art: '/assets/platform-mac.jpg',
    buttonText: 'Download Desktop App',
    href: 'https://github.com/babtix/kaioken/releases/download/v2.0.0/Kaioken-Setup.dmg',
    icon: 'apple',
  },
  {
    os: 'Windows',
    version: 'Windows 10/11 • x64 & ARM64',
    art: '/assets/platform-win.jpg',
    buttonText: 'Download Desktop App',
    href: 'https://github.com/babtix/kaioken/releases/download/v2.0.0/Kaioken-Setup.exe',
    icon: 'windows',
  },
  {
    os: 'Linux',
    version: 'Any Distro • AppImage & CLI',
    art: '/assets/platform-linux.jpg',
    buttonText: 'Install via Terminal',
    href: '#install',
    icon: 'linux',
  },
];

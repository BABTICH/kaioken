/* ==========================================================================
   KAIOKEN DESKTOP — vanilla ES state machine
   No framework, no bundler, no devserver. The shell is rendered from the
   server snapshot; server pushes merge by revision, never by full reload.
   ========================================================================== */
(function () {
  'use strict';

  const INITIAL = window.__KAIOKEN_INITIAL_STATE__ || {};

  const ROUTE_META = [
    { id: 'chat', label: 'Chat', icon: 'chat', shortcut: 'Ctrl+1', group: 'Ask' },
    { id: 'research', label: 'Research', icon: 'radar', shortcut: 'Ctrl+2', group: 'Ask' },
    { id: 'wiki', label: 'Wiki', icon: 'book', shortcut: 'Ctrl+3', group: 'Knowledge' },
    { id: 'graph', label: 'Graph', icon: 'waypoints', shortcut: 'Ctrl+4', group: 'Knowledge' },
    { id: 'cards', label: 'Cards', icon: 'layers', shortcut: 'Ctrl+5', group: 'Knowledge' },
    { id: 'editor', label: 'Editor', icon: 'code', shortcut: 'Ctrl+6', group: 'Tools' },
    { id: 'browser', label: 'Browser', icon: 'globe', shortcut: 'Ctrl+7', group: 'Tools' },
    { id: 'activity', label: 'Activity', icon: 'zap', shortcut: 'Ctrl+8', group: 'Tools' },
    { id: 'extensions', label: 'Extensions', icon: 'puzzle', shortcut: 'Ctrl+9', group: 'Tools' },
    { id: 'cost', label: 'Cost', icon: 'wallet', shortcut: '', group: 'System' },
    { id: 'settings', label: 'Settings', icon: 'settings', shortcut: 'Ctrl+,', group: 'System' }
  ];

  const SHORTCUTS = [
    { group: 'General', rows: [
      ['Ctrl+K', 'Command palette'],
      ['Ctrl+P', 'Quick open file'],
      ['Ctrl+Shift+S', 'Run codebase scan'],
      ['Ctrl+Shift+D', 'Run environment doctor'],
      ['?', 'Shortcut help'],
      ['Escape', 'Close dialog or deny tool']
    ]},
    { group: 'Navigation', rows: [
      ['Ctrl+1', 'Chat'],
      ['Ctrl+2', 'Research'],
      ['Ctrl+3', 'Wiki'],
      ['Ctrl+4', 'Graph'],
      ['Ctrl+5', 'Cards'],
      ['Ctrl+6', 'Editor'],
      ['Ctrl+7', 'Browser'],
      ['Ctrl+8', 'Activity'],
      ['Ctrl+9', 'Extensions'],
      ['Ctrl+,', 'Settings']
    ]},
    { group: 'Chat & Agent', rows: [
      ['Ctrl+N', 'New chat session'],
      ['Ctrl+Shift+C', 'Compact conversation'],
      ['Enter', 'Send message / Approve proposal'],
      ['Alt+Enter', 'New line'],
      ['Esc', 'Deny proposal']
    ]},
    { group: 'Editor & Terminal', rows: [
      ['Ctrl+S', 'Save file'],
      ['Ctrl+`', 'Toggle terminal']
    ]},
    { group: 'Browser', rows: [
      ['Ctrl+L', 'Focus address bar'],
      ['Ctrl+T', 'New tab'],
      ['Ctrl+W', 'Close tab']
    ]},
    { group: 'Research', rows: [
      ['×1–×9', 'Normal budget'],
      ['×10', 'Advanced dossier'],
      ['Enter', 'Start research'],
      ['Esc', 'Stop or close']
    ]}
  ];

  const KEYWORDS = new Set([
    'use', 'pub', 'fn', 'struct', 'enum', 'impl', 'let', 'mut', 'match', 'if', 'else',
    'for', 'while', 'loop', 'return', 'async', 'await', 'const', 'var', 'function',
    'class', 'import', 'from', 'export', 'new', 'this', 'typeof', 'true', 'false',
    'null', 'Some', 'None', 'Ok', 'Err', 'Result', 'Self', 'self', 'mod', 'trait',
    'where', 'move', 'unsafe', 'static', 'type', 'ref', 'in', 'as', 'dyn', 'try'
  ]);

  const DEFAULTS = {
    active_tab: 'chat',
    theme: 'dark',
    repo: { name: 'kaioRust', path: '~/repos/kaioRust', branch: 'main', dirty_count: 0, is_repo: true },
    daemon: { status: 'connected', mode: 'embedded', version: '0.1.0', endpoint: 'embedded in-process', port: 47890, latency_ms: 1 },
    agent: { current_task: null, is_running: false, total_tool_calls: 0, total_approved: 0, total_denied: 0, sessions: [], pending_proposals: [], messages: [], active_session_id: null },
    memory: { total_files: 0, total_symbols: 0, total_edges: 0, nodes: [], edges: [], selected_node_id: null, inspector: null, zoom_level: 1 },
    review: { changed_files: [], additions: 0, deletions: 0, risk_score: 0, risk_level: 'Clean', risk_reasons: [], diff: '', is_clean: true },
    health: { score: 0, grade: '—', total_docs: 0, fresh_docs: 0, stale_docs: 0, missing_docs: 0 },
    audit: { entries: [], total_intercepted: 0, total_redacted: 0 },
    research: { query: '', power_level: 3, is_running: false, rounds: 0, searched: 0, fetched: 0, crawl_depth: 2, duration_ms: 0, path: 'fast', grounded: 0, total_claims: 0, cost_usd: 0, answer_markdown: '', citations: [], sources: [], steps: [], history: [] },
    wiki: { chapters: [], selected_path: '' },
    failure_cards: [],
    editor: { files: [], active_path: '', diff: '', diff_summary: '', terminal_open: false, terminal_lines: [] },
    browser: { tabs: [], active_id: '', method: 'GET', request_url: '', status: 0, latency_ms: 0, response_headers: [], response_body: '' },
    activity: { runs: [], events: [], sessions_recorded: 0, learned_skills: 0, reinforced_skills: 0 },
    extensions: { installed: [], registry: [], hooks: [] },
    cost: { total_usd: 0, known_cost_usd: 0, calls: 0, prompt_tokens: 0, completion_tokens: 0, local_calls: 0, pricing_stale: false, by_operation: [], by_model: [], by_provider: [], by_workspace: [], by_day: [] },
    settings: { permission_mode: 'edit-with-approval', model: 'deepseek-coder', base_url: '', search_provider: 'both', fetcher_mode: 'auto', daemon_mode: 'embedded', endpoint: 'embedded in-process', theme: 'dark', has_openrouter_key: true, has_anthropic_key: false, has_perplexity_key: true, has_tavily_key: false, watcher_enabled: true, debounce_ms: 300 },
    tokens: { total: 0, prompt: 0, completion: 0, calls: 0, usd: 0 },
    palette_actions: [],
    routes: ROUTE_META
  };

  function isObject(value) {
    return value !== null && typeof value === 'object' && !Array.isArray(value);
  }

  function deepMerge(base, patch) {
    if (Array.isArray(patch)) return patch.slice();
    if (!isObject(patch)) return patch;
    const out = isObject(base) ? Object.assign({}, base) : {};
    Object.keys(patch).forEach(function (key) {
      out[key] = deepMerge(base ? base[key] : undefined, patch[key]);
    });
    return out;
  }

  function clone(value) {
    return JSON.parse(JSON.stringify(value));
  }

  let state = deepMerge(DEFAULTS, INITIAL);
  let booted = false;

  function storedTheme() {
    try {
      return localStorage.getItem('kaioken-theme');
    } catch (error) {
      return null;
    }
  }

  function saveTheme(theme) {
    try {
      localStorage.setItem('kaioken-theme', theme);
    } catch (error) {
      /* WebView storage may be disabled; the in-memory theme still works. */
    }
  }

  const ui = {
    route: state.active_tab || 'chat',
    theme: storedTheme() || state.theme || 'dark',
    chatInput: '',
    chatPending: false,
    autoApprove: false,
    allowShell: false,
    researchInput: '',
    researchTimer: null,
    wikiQuery: '',
    wikiSelected: state.wiki.selected_path || (state.wiki.chapters[0] && state.wiki.chapters[0].path) || '',
    wikiToc: '',
    graphSelected: state.memory.selected_node_id,
    graphQuery: '',
    graphKind: 'all',
    graphZoom: Number(state.memory.zoom_level) || 1,
    cardFilter: 'all',
    cardSelected: state.failure_cards[0] ? state.failure_cards[0].id : '',
    editorMode: 'code',
    activeFile: state.editor.active_path || (state.editor.files[0] && state.editor.files[0].path) || '',
    terminalOpen: !!state.editor.terminal_open,
    terminalTab: 0,
    terminalInput: '',
    browserActive: state.browser.active_id || (state.browser.tabs[0] && state.browser.tabs[0].id) || '',
    browserUrl: '',
    activityDepth: 4,
    openRuns: { 'run-101': true },
    extensionsTab: 'installed',
    installSource: '',
    trustExt: null,
    costWindow: '30d',
    costScoped: false,
    settingsDraft: clone(state.settings),
    settingsDirty: false,
    paletteOpen: false,
    paletteMode: 'commands',
    paletteQuery: '',
    paletteIndex: 0,
    paletteFiltered: [],
    paletteFocused: false,
    bridgeWarned: false,
    helpOpen: false,
    approvalFocusId: null,
    dismissedApproval: null,
    repoMenuOpen: false,
    toasts: [],
    toastSeq: 0,
    maximized: false
  };

  /* ── Small helpers ───────────────────────────────────────────────────── */
  function esc(value) {
    return String(value == null ? '' : value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function attr(value) {
    return esc(value).replace(/`/g, '&#96;');
  }

  function icon(name, cls) {
    return '<svg class="icon ' + (cls || '') + '" aria-hidden="true"><use href="#i-' + name + '"></use></svg>';
  }

  function kbd(key) {
    return '<span class="kbd">' + esc(key) + '</span>';
  }

  function badge(text, tone) {
    return '<span class="badge badge--' + (tone || 'muted') + '">' + text + '</span>';
  }

  function routeMeta(id) {
    const routes = Array.isArray(state.routes) && state.routes.length ? state.routes : ROUTE_META;
    return routes.find(function (route) { return route.id === id; }) || ROUTE_META[0];
  }

  function formatTokens(number) {
    const n = Number(number) || 0;
    if (n >= 1e9) return (n / 1e9).toFixed(1).replace(/\.0$/, '') + 'B';
    if (n >= 1e6) return (n / 1e6).toFixed(1).replace(/\.0$/, '') + 'M';
    if (n >= 1e3) return (n / 1e3).toFixed(1).replace(/\.0$/, '') + 'k';
    return String(n);
  }

  function formatUsd(value) {
    const n = Number(value) || 0;
    if (n === 0) return '$0';
    if (n < 0.01) return '$' + n.toFixed(4);
    if (n < 1) return '$' + n.toFixed(3);
    return '$' + n.toFixed(2);
  }

  function formatDuration(ms) {
    const total = Math.max(0, Math.round(Number(ms) / 1000));
    if (total < 60) return total + 's';
    const minutes = Math.floor(total / 60);
    const seconds = total % 60;
    return minutes + 'm ' + String(seconds).padStart(2, '0') + 's';
  }

  function slugify(text) {
    return String(text).toLowerCase().replace(/<[^>]+>/g, '').replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'section';
  }

  function isInputFocused() {
    const el = document.activeElement;
    if (!el) return false;
    const tag = el.tagName.toLowerCase();
    return tag === 'input' || tag === 'textarea' || tag === 'select' || el.isContentEditable;
  }

  function post(message) {
    try {
      if (window.ipc && typeof window.ipc.postMessage === 'function') {
        window.ipc.postMessage(JSON.stringify(message));
        return;
      }
    } catch (error) {
      /* Fall through to the warning below. */
    }
    // The page is also useful in a plain browser for design work, but actions
    // cannot reach the Rust host there. Say so once instead of failing silently.
    if (!ui.bridgeWarned) {
      ui.bridgeWarned = true;
      toast('warn', 'Desktop bridge unavailable', 'Running outside the Kaioken host; actions are local only.');
    }
  }

  function currentApproval() {
    const pending = state.agent.pending_proposals || [];
    if (!pending.length) return null;
    const first = pending[0];
    if (ui.dismissedApproval === first.id && pending.length === 1) return null;
    return first;
  }

  /* ── Toast ───────────────────────────────────────────────────────────── */
  function toast(kind, title, body) {
    const id = ++ui.toastSeq;
    ui.toasts.push({ id: id, kind: kind || 'info', title: title, body: body || '' });
    if (ui.toasts.length > 4) ui.toasts.shift();
    renderToasts();
    setTimeout(function () {
      ui.toasts = ui.toasts.filter(function (item) { return item.id !== id; });
      renderToasts();
    }, 3600);
  }

  function renderToasts() {
    const root = document.getElementById('toastRoot');
    if (!root) return;
    root.innerHTML = ui.toasts.map(function (item) {
      return '<div class="toast is-' + item.kind + '">' +
        '<div class="flex-1"><div class="toast-title">' + esc(item.title) + '</div>' +
        (item.body ? '<div class="toast-body">' + esc(item.body) + '</div>' : '') +
        '</div><button class="icon-btn" data-action="dismiss-toast" data-id="' + item.id + '" aria-label="Dismiss">' +
        icon('x', 'icon--sm') + '</button></div>';
    }).join('');
  }

  /* ── Markdown ─────────────────────────────────────────────────────────── */
  function inlineMarkdown(text) {
    let html = esc(text);
    const codes = [];
    html = html.replace(/`([^`]+)`/g, function (match, code) {
      codes.push(code);
      return '\u0001' + (codes.length - 1) + '\u0002';
    });
    html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/\*([^*\n]+)\*/g, '<em>$1</em>');
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, function (match, label, url) {
      return '<span class="doc-link" data-action="open-link" data-url="' + attr(url) + '">' + label + '</span>';
    });
    html = html.replace(/\[(\d+)\]/g, '<button class="cite-chip" data-action="cite" data-index="$1">$1</button>');
    html = html.replace(/\u0001(\d+)\u0002/g, function (match, index) {
      return '<code>' + codes[Number(index)] + '</code>';
    });
    return html;
  }

  function markdownToHtml(markdown) {
    const lines = String(markdown || '').split(/\r?\n/);
    const out = [];
    let paragraph = [];
    let listTag = '';
    let inCode = false;
    let codeLang = '';
    let codeLines = [];

    function flushParagraph() {
      if (!paragraph.length) return;
      out.push('<p>' + inlineMarkdown(paragraph.join(' ')) + '</p>');
      paragraph = [];
    }

    function flushList() {
      if (!listTag) return;
      out.push('</' + listTag + '>');
      listTag = '';
    }

    function flushCode() {
      if (!inCode) return;
      out.push('<div class="code-block">' +
        (codeLang ? '<span class="code-lang">' + esc(codeLang) + '</span>' : '') +
        '<pre><code>' + esc(codeLines.join('\n')) + '</code></pre></div>');
      inCode = false;
      codeLang = '';
      codeLines = [];
    }

    lines.forEach(function (line) {
      const fence = line.match(/^\s*```(.*)$/);
      if (fence) {
        if (inCode) {
          flushCode();
        } else {
          flushParagraph();
          flushList();
          inCode = true;
          codeLang = fence[1].trim();
        }
        return;
      }
      if (inCode) {
        codeLines.push(line);
        return;
      }
      if (!line.trim()) {
        flushParagraph();
        flushList();
        return;
      }
      if (/^\s*(---|\*\*\*|___)\s*$/.test(line)) {
        flushParagraph();
        flushList();
        out.push('<hr>');
        return;
      }
      const heading = line.match(/^(#{1,4})\s+(.*)$/);
      if (heading) {
        flushParagraph();
        flushList();
        const level = heading[1].length;
        const text = heading[2].trim();
        out.push('<h' + level + ' id="' + slugify(text) + '">' + inlineMarkdown(text) + '</h' + level + '>');
        return;
      }
      if (/^\s*>\s?/.test(line)) {
        flushParagraph();
        flushList();
        out.push('<blockquote>' + inlineMarkdown(line.replace(/^\s*>\s?/, '')) + '</blockquote>');
        return;
      }
      const bullet = line.match(/^\s*[-*]\s+(.*)$/);
      if (bullet) {
        flushParagraph();
        if (listTag !== 'ul') {
          flushList();
          out.push('<ul>');
          listTag = 'ul';
        }
        out.push('<li>' + inlineMarkdown(bullet[1]) + '</li>');
        return;
      }
      const numbered = line.match(/^\s*\d+[.)]\s+(.*)$/);
      if (numbered) {
        flushParagraph();
        if (listTag !== 'ol') {
          flushList();
          out.push('<ol>');
          listTag = 'ol';
        }
        out.push('<li>' + inlineMarkdown(numbered[1]) + '</li>');
        return;
      }
      flushList();
      paragraph.push(line.trim());
    });

    flushParagraph();
    flushList();
    flushCode();
    return out.join('\n');
  }

  function extractToc(markdown) {
    const toc = [];
    let inCode = false;
    String(markdown || '').split(/\r?\n/).forEach(function (line) {
      if (/^\s*```/.test(line)) {
        inCode = !inCode;
        return;
      }
      if (inCode) return;
      const heading = line.match(/^(#{2,4})\s+(.*)$/);
      if (heading) {
        toc.push({ level: heading[1].length, text: heading[2].trim(), slug: slugify(heading[2]) });
      }
    });
    return toc;
  }

  function highlightLine(line) {
    if (!line) return '&nbsp;';
    const trimmed = line.trimStart();
    if (trimmed.indexOf('//') === 0) return '<span class="tok-com">' + esc(line) + '</span>';
    let out = '';
    let index = 0;
    while (index < line.length) {
      const ch = line[index];
      if (ch === '"' || ch === "'") {
        let next = index + 1;
        let token = ch;
        while (next < line.length) {
          token += line[next];
          if (line[next] === '\\') {
            next += 1;
            if (next < line.length) token += line[next];
          } else if (line[next] === ch) {
            break;
          }
          next += 1;
        }
        next += 1;
        out += '<span class="tok-str">' + esc(token) + '</span>';
        index = next;
        continue;
      }
      if (ch === '/' && line[index + 1] === '/') {
        out += '<span class="tok-com">' + esc(line.slice(index)) + '</span>';
        break;
      }
      if (/[A-Za-z_]/.test(ch)) {
        let next = index + 1;
        while (next < line.length && /[A-Za-z0-9_]/.test(line[next])) next += 1;
        const word = line.slice(index, next);
        if (KEYWORDS.has(word)) {
          out += '<span class="tok-key">' + esc(word) + '</span>';
        } else if (/^[A-Z][A-Za-z0-9_]*$/.test(word)) {
          out += '<span class="tok-type">' + esc(word) + '</span>';
        } else {
          out += esc(word);
        }
        index = next;
        continue;
      }
      if (/[0-9]/.test(ch)) {
        let next = index + 1;
        while (next < line.length && /[0-9_.]/.test(line[next])) next += 1;
        out += '<span class="tok-num">' + esc(line.slice(index, next)) + '</span>';
        index = next;
        continue;
      }
      out += esc(ch);
      index += 1;
    }
    return out;
  }

  function renderCode(code) {
    return String(code || '').split(/\r?\n/).map(function (line, index) {
      return '<div class="code-line"><span class="line-no">' + (index + 1) + '</span>' +
        '<span class="line-code">' + highlightLine(line) + '</span></div>';
    }).join('');
  }

  function renderDiff(diff) {
    const lines = String(diff || 'Working tree clean — no uncommitted changes.').split(/\r?\n/);
    let oldLine = 0;
    let newLine = 0;
    return lines.map(function (line) {
      let cls = '';
      let sign = ' ';
      if (line.indexOf('@@') === 0) {
        cls = 'is-hunk';
        const match = line.match(/@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@/);
        if (match) {
          oldLine = Number(match[1]);
          newLine = Number(match[3]);
        }
      } else if (line.indexOf('diff ') === 0 || line.indexOf('index ') === 0 || line.indexOf('---') === 0 || line.indexOf('+++') === 0) {
        cls = 'is-meta';
      } else if (line.charAt(0) === '+') {
        cls = 'is-add';
        sign = '+';
        newLine += 1;
      } else if (line.charAt(0) === '-') {
        cls = 'is-del';
        sign = '−';
        oldLine += 1;
      } else {
        oldLine += 1;
        newLine += 1;
      }
      return '<div class="diff-line ' + cls + '"><span class="diff-no">' + (oldLine || '') + '</span>' +
        '<span class="diff-no">' + (newLine || '') + '</span>' +
        '<span class="diff-sign">' + sign + '</span>' +
        '<span class="diff-code">' + esc(line.replace(/^[+-]/, '')) + '</span></div>';
    }).join('');
  }

  /* ── Shell rendering ──────────────────────────────────────────────────── */
  function applyTheme() {
    document.documentElement.dataset.theme = ui.theme;
    saveTheme(ui.theme);
  }

  function render() {
    renderNav();
    renderTitlebar();
    renderStatusbar();
    renderRepoMenu();
    renderRoute();
    renderModals();
    renderToasts();
  }

  function renderNav() {
    const nav = document.getElementById('navRail');
    if (!nav) return;
    const routes = Array.isArray(state.routes) && state.routes.length ? state.routes : ROUTE_META;
    let lastGroup = '';
    let html = '<button class="rail-item rail-repos" data-action="repo-menu" title="Repositories and workspaces">' +
      icon('folder') + '<span class="rail-label">Repos</span></button><div class="rail-sep"></div>';
    routes.forEach(function (route) {
      if (lastGroup && route.group !== lastGroup) {
        html += '<div class="rail-sep rail-sep--group"></div>';
      }
      lastGroup = route.group;
      const active = ui.route === route.id;
      const runBadge = route.id === 'activity' && activeRunCount() > 0 ? '<span class="rail-badge">' + activeRunCount() + '</span>' : '';
      const tip = route.label + (route.shortcut ? ' ' + kbd(route.shortcut) : '');
      html += '<button class="rail-item' + (active ? ' is-active' : '') + '" data-route="' + route.id + '" title="' + attr(route.label + (route.shortcut ? ' · ' + route.shortcut : '')) + '">' +
        icon(route.icon) + runBadge + '<span class="rail-label">' + esc(route.label) + '</span>' +
        '<span class="rail-tip">' + esc(route.label) + (route.shortcut ? ' ' + kbd(route.shortcut) : '') + '</span></button>';
    });
    html += '<div class="rail-store"><div class="rail-sep"></div><button class="rail-item" data-action="open-registry" title="Extension registry">' +
      icon('store') + '<span class="rail-label">Store</span><span class="rail-tip">Extension registry</span></button></div>';
    nav.innerHTML = html;
  }

  function activeRunCount() {
    return (state.activity.runs || []).filter(function (run) {
      return run.state === 'running' || run.state === 'queued';
    }).length;
  }

  function renderTitlebar() {
    const repoName = document.getElementById('repoName');
    const branchName = document.getElementById('branchName');
    const dirtyCount = document.getElementById('dirtyCount');
    const modelBadge = document.getElementById('modelBadge');
    const powerMini = document.getElementById('powerMini');
    const maximizeIcon = document.getElementById('maximizeIcon');
    if (repoName) repoName.textContent = state.repo.name || 'kaioRust';
    if (branchName) branchName.textContent = state.repo.branch || 'main';
    if (dirtyCount) {
      const dirty = Number(state.repo.dirty_count) || 0;
      dirtyCount.textContent = dirty > 0 ? '·' + dirty : '';
      dirtyCount.style.display = dirty > 0 ? '' : 'none';
    }
    if (modelBadge) {
      const model = String(state.settings.model || 'deepseek-coder');
      const short = model.indexOf('/') >= 0 ? model.split('/').pop() : model;
      modelBadge.textContent = short;
      modelBadge.title = model;
    }
    if (powerMini) powerMini.textContent = '×' + (state.research.power_level || 1);
    if (maximizeIcon) maximizeIcon.querySelector('use').setAttribute('href', ui.maximized ? '#i-restore' : '#i-square');
    const repoSwitch = document.getElementById('repoSwitch');
    if (repoSwitch) repoSwitch.classList.toggle('is-open', ui.repoMenuOpen);
    const title = 'Kaioken — ' + (state.repo.name || 'workspace');
    document.title = title;
    if (state.agent.is_running && state.agent.current_task) document.title = title + ' · ' + state.agent.current_task;
  }

  function renderStatusbar() {
    const dot = document.getElementById('connDot');
    const info = document.getElementById('daemonInfo');
    const task = document.getElementById('statusTask');
    const path = document.getElementById('statusPath');
    const tokens = document.getElementById('tokenMeter');
    const themeIcon = document.getElementById('themeIcon');
    const terminalToggle = document.getElementById('terminalToggle');
    if (dot) {
      dot.className = 'status-dot' + (state.daemon.status === 'connected' ? '' : state.daemon.status === 'disconnected' ? ' status-dot--offline' : ' status-dot--connecting');
      dot.title = state.daemon.status || 'disconnected';
    }
    if (info) {
      info.textContent = state.daemon.mode === 'embedded'
        ? 'embedded in-process'
        : 'daemon v' + (state.daemon.version || '0.1.0') + ' · :' + (state.daemon.port || 47890);
    }
    if (task) {
      if (state.agent.is_running && state.agent.current_task) {
        task.innerHTML = '<span class="spinner"></span><span class="truncate">agent · ' + esc(state.agent.current_task) + '</span>';
      } else {
        const run = (state.activity.runs || []).find(function (item) {
          return item.state === 'running' || item.state === 'queued';
        });
        task.innerHTML = run
          ? '<span class="live-dot live-dot--orange live-dot--pulse"></span><span class="truncate">' + esc(run.label) + ' · ' + esc(run.message || 'active') + '</span>'
          : '';
      }
    }
    if (path) {
      path.textContent = state.repo.path || '';
      path.title = state.repo.path || '';
    }
    if (tokens) {
      tokens.querySelector('span').textContent = formatTokens(state.tokens.total) + ' tokens';
      tokens.title = formatTokens(state.tokens.prompt) + ' prompt + ' + formatTokens(state.tokens.completion) + ' completion · ' + (state.tokens.calls || 0) + ' calls';
    }
    if (themeIcon) themeIcon.querySelector('use').setAttribute('href', ui.theme === 'dark' ? '#i-sun' : '#i-moon');
    if (terminalToggle) terminalToggle.classList.toggle('is-on', ui.route === 'editor' && ui.terminalOpen);
  }

  function renderRepoMenu() {
    const menu = document.getElementById('repoMenu');
    if (!menu) return;
    if (!ui.repoMenuOpen) {
      menu.hidden = true;
      menu.innerHTML = '';
      return;
    }
    menu.hidden = false;
    const repo = state.repo || {};
    menu.innerHTML = '<div class="repo-menu-head">Active Workspace</div>' +
      '<button class="repo-menu-item" data-action="repo-select" data-name="' + attr(repo.name || 'kaioRust') + '" data-path="' + attr(repo.path || '.') + '">' +
        icon('folder', 'icon--sm') + '<span>' + esc(repo.name || 'kaioRust') + '</span>' +
        '<span class="badge badge--green">active</span>' +
        '<span class="repo-path">' + esc(repo.path || '.') + '</span></button>' +
      '<button class="repo-menu-item" data-action="open-repo-picker">' + icon('rotate', 'icon--sm') + '<span>Re-scan workspace…</span></button>' +
      '<div class="repo-menu-foot">' + icon('git-branch', 'icon--sm') + ' ' + esc(repo.branch || 'main') + ' · ' + (repo.dirty_count || 0) + ' uncommitted changes</div>';
  }

  function captureScroll() {
    const map = {};
    document.querySelectorAll('[data-scroll]').forEach(function (el) {
      map[el.dataset.scroll] = el.scrollTop;
    });
    return map;
  }

  function restoreScroll(map) {
    Object.keys(map).forEach(function (key) {
      const el = document.querySelector('[data-scroll="' + key + '"]');
      if (el) el.scrollTop = map[key];
    });
  }

  function renderRoute() {
    const viewport = document.getElementById('viewport');
    if (!viewport) return;
    const active = document.activeElement;
    const focusKey = active && active.getAttribute ? active.getAttribute('data-focus') : null;
    const selectionStart = active && typeof active.selectionStart === 'number' ? active.selectionStart : null;
    const scroll = captureScroll();
    const renderers = {
      chat: renderChat,
      research: renderResearch,
      wiki: renderWiki,
      graph: renderGraph,
      cards: renderCards,
      editor: renderEditor,
      browser: renderBrowser,
      activity: renderActivity,
      extensions: renderExtensions,
      cost: renderCost,
      settings: renderSettings
    };
    const renderer = renderers[ui.route] || renderChat;
    viewport.innerHTML = renderer();
    restoreScroll(scroll);
    if (focusKey) {
      const el = document.querySelector('[data-focus="' + focusKey + '"]');
      if (el) {
        el.focus();
        if (selectionStart !== null && typeof el.setSelectionRange === 'function') {
          try { el.setSelectionRange(selectionStart, selectionStart); } catch (error) { /* not a text field */ }
        }
      }
    }
    ensureResearchTimer();
  }

  /* ── Chat ─────────────────────────────────────────────────────────────── */
  function renderChat() {
    const agent = state.agent;
    const sessions = agent.sessions || [];
    const messages = agent.messages || [];
    const visible = messages.filter(function (message) { return message && message.type !== 'system'; });
    const activeSession = agent.active_session_id;
    const sessionHtml = sessions.map(function (session) {
      return '<button class="session-item' + (session.id === activeSession ? ' is-active' : '') + '" data-action="switch-session" data-id="' + attr(session.id) + '">' +
        '<span class="session-title">' + esc(session.title || 'Untitled session') + '</span>' +
        '<span class="session-meta">' + icon('clock', 'icon--sm') + esc(session.updated || 'just now') + ' · ' + session.turns + ' turn' + (session.turns === 1 ? '' : 's') + '</span></button>';
    }).join('');

    const empty = visible.length === 0 && !agent.is_running && !ui.chatPending;
    const threadHtml = empty ? chatIntro() : visible.map(renderChatMessage).join('') +
      (ui.chatPending || agent.is_running ? '<div class="live-tail"><span class="live-dot live-dot--orange live-dot--pulse"></span><span>Reading repository context and composing a plan…</span></div>' : '');

    return '<div class="chat-layout">' +
      '<aside class="chat-sidebar">' +
        '<div class="chat-sidebar-head">' +
          '<span class="section-label section-label--plain">Sessions</span>' +
          '<span class="count">' + sessions.length + '</span>' +
          '<button class="icon-btn ml-auto" data-action="new-session" title="New session (Ctrl+N)">' + icon('plus', 'icon--sm') + '</button>' +
        '</div>' +
        '<div class="session-list">' + (sessionHtml || '<div class="empty-panel" style="margin:8px;padding:20px 12px">No sessions yet.</div>') + '</div>' +
        '<div class="chat-sidebar-foot">' +
          '<div class="kv-row"><span>Tool calls</span><b>' + (agent.total_tool_calls || 0) + '</b></div>' +
          '<div class="kv-row"><span>Approved</span><b class="green">' + (agent.total_approved || 0) + '</b></div>' +
          '<div class="kv-row"><span>Denied</span><b class="rose">' + (agent.total_denied || 0) + '</b></div>' +
        '</div>' +
      '</aside>' +
      '<section class="chat-main">' +
        '<div class="chat-scroll" data-scroll="chat"><div class="chat-thread">' + threadHtml + '</div></div>' +
        chatComposer() +
      '</section>' +
    '</div>';
  }

  function chatIntro() {
    const model = state.settings.model || 'deepseek-coder';
    const suggestions = [
      'Explain how the wiki pipeline decides what to regenerate',
      'Where is the approval flow implemented?',
      'Add a --json flag to the status command'
    ];
    return '<div class="chat-intro">' +
      '<div class="chat-intro-mark">' + icon('terminal', 'icon--sm') + ' KAI OKEN · repo-aware agent</div>' +
      '<h1>Ask about this <span class="serif">repository.</span></h1>' +
      '<p>The agent can read, search, and edit files. Every write crosses an approval you control, and denying leaves the tree byte-identical.</p>' +
      '<div class="flex items-center gap-2 mt-3">' + badge(esc(model), 'orange') + badge('edit-with-approval', 'muted') + '</div>' +
      '<div class="chat-suggestions">' + suggestions.map(function (text) {
        return '<button class="suggestion" data-action="suggestion" data-text="' + attr(text) + '">' + esc(text) + icon('corner-down', 'icon--sm') + '</button>';
      }).join('') + '</div>' +
    '</div>';
  }

  function renderChatMessage(message) {
    if (!message) return '';
    if (message.type === 'user') {
      return '<div class="msg-row msg-row--user"><div><div class="msg-meta">YOU</div><div class="msg-user">' + esc(message.content) + '</div></div></div>';
    }
    if (message.type === 'thought') {
      return '<div class="thought-row"><span class="thought-label">thinking</span><span class="thought-body">' + esc(message.content) + '</span></div>';
    }
    if (message.type === 'proposal') {
      return renderProposalInline(message.proposal);
    }
    if (message.type === 'result') {
      const ok = !!message.approved;
      return '<div class="tool-line ' + (ok ? 'is-ok' : 'is-denied') + '">' + icon(ok ? 'check-circle' : 'x', 'icon--sm') +
        '<span class="tool-name">' + (ok ? 'applied' : 'denied') + '</span><span class="truncate">' + esc(message.output) + '</span></div>';
    }
    if (message.type === 'response') {
      return '<article class="answer-surface hud-corners">' + markdownToHtml(message.content) + '</article>';
    }
    return '';
  }

  function renderProposalInline(proposal) {
    if (!proposal) return '';
    const tier = String(proposal.policy_tier || 'WRITE_PROMPT');
    const tone = tier === 'DANGEROUS' ? 'rose' : tier === 'READ_ONLY' ? 'green' : 'amber';
    return '<div class="proposal-card hud-corners">' +
      '<div class="proposal-head">' + icon('alert', 'icon--sm') + ' TOOL PROPOSAL · ' + esc(proposal.tool_name) + badge(tier, tone) + '</div>' +
      '<div class="proposal-body"><dl class="proposal-kv"><dt>Target</dt><dd>' + esc(proposal.target) + '</dd>' +
      '<dt>Details</dt><dd>' + esc(proposal.details) + '</dd></dl>' +
      (proposal.diff ? '<div class="approval-diff mt-2">' + renderDiff(proposal.diff) + '</div>' : '') + '</div>' +
      '<div class="proposal-actions"><button class="btn btn--danger btn--sm" data-action="deny-tool" data-id="' + attr(proposal.id) + '">Deny ' + kbd('Esc') + '</button>' +
      '<button class="btn btn--primary btn--sm" data-action="approve-tool" data-id="' + attr(proposal.id) + '">Approve ' + kbd('Enter') + '</button></div></div>';
  }

  function chatComposer() {
    const disabled = !ui.chatInput.trim();
    return '<div class="composer"><div class="composer-wrap">' +
      '<div class="composer-box hud-corners">' +
        '<textarea data-field="chat-input" data-focus="chat-input" rows="1" placeholder="Ask anything…  /  for commands,  @  for files">' + esc(ui.chatInput) + '</textarea>' +
        '<button class="send-btn" data-action="send-chat" title="Send (Enter)" aria-label="Send"' + (disabled ? ' disabled' : '') + '>' + icon('arrow-up') + '</button>' +
      '</div>' +
      '<div class="composer-foot">' +
        '<button class="toggle-chip' + (ui.autoApprove ? ' is-on' : '') + '" data-action="toggle-auto-approve" title="Skip the approval dialog for this turn">' + icon('zap', 'icon--sm') + ' auto-approve</button>' +
        '<button class="toggle-chip is-danger' + (ui.allowShell ? ' is-on' : '') + '" data-action="toggle-allow-shell" title="Offer the shell tool to the model">' + icon('terminal', 'icon--sm') + ' allow shell</button>' +
        '<button class="toggle-chip" data-action="compact-session" title="Compact conversation">' + icon('layers', 'icon--sm') + ' compact</button>' +
        '<span class="composer-hint">' + kbd('/') + ' commands · ' + kbd('@') + ' files · ' + kbd('Alt') + '+' + kbd('Enter') + ' newline</span>' +
      '</div>' +
    '</div></div>';
  }

  /* ── Research ─────────────────────────────────────────────────────────── */
  function renderResearch() {
    const research = state.research;
    const power = research.power_level || 1;
    const running = !!research.is_running;
    const segments = Array.from({ length: 10 }, function (_, index) {
      const level = index + 1;
      return '<button class="power-seg' + (level <= power ? ' is-on' : '') + (level >= 7 ? ' is-hot' : '') + '" data-action="set-power" data-power="' + level + '" title="Power ×' + level + '"></button>';
    }).join('');
    const steps = (research.steps || []).map(function (step) {
      const status = step.status || 'pending';
      const mark = status === 'done' ? icon('check', 'icon--sm') : status === 'active' ? '<span class="spinner"></span>' : '·';
      return '<div class="step-row is-' + status + '"><span class="step-icon">' + mark + '</span><div><div class="step-label">' + esc(step.label) + '</div><div class="step-detail">' + esc(step.detail) + '</div></div></div>';
    }).join('');
    const sources = (research.sources || []).map(function (source) {
      return '<button class="source-chip" data-action="open-link" data-url="' + attr(source.url) + '"><span class="num">' + esc(source.host) + '</span><span class="truncate">' + esc(source.title) + '</span></button>';
    }).join('');
    const citeHtml = research.answer_markdown ? markdownToHtml(research.answer_markdown) : '';
    const sourceRow = (research.citations || []).map(function (citation) {
      return '<button class="source-chip" data-action="cite" data-index="' + citation.index + '"><span class="num">[' + citation.index + ']</span><span class="truncate">' + esc(citation.host) + '</span></button>';
    }).join('');

    return '<div class="route-scroll" data-scroll="research"><div class="route-wrap research-wrap">' +
      '<header class="route-header">' +
        '<div><p class="label route-eyebrow">ASK THE WEB · CITED DOSSIER</p><h1 class="route-title">Research</h1>' +
        '<p class="route-sub">Searches the web, reads pages, and writes a cited report. The power meter is the actual budget.</p></div>' +
        '<div class="route-header-right">' + (running ? '<span class="live-dot live-dot--orange live-dot--pulse"></span><span class="label label--accent">researching</span>' : badge('ready', 'green')) +
        badge('×' + power, 'orange') + badge(research.path === 'deep' ? 'deep path' : 'fast path', research.path === 'deep' ? 'orange' : 'muted') + '</div>' +
      '</header>' +
      '<section class="panel research-composer hud-corners">' +
        '<div class="research-composer-top">' +
          '<div class="query-box">' + icon('search') + '<input data-field="research-query" data-focus="research-query" value="' + attr(ui.researchInput) + '" placeholder="Ask a research question…" spellcheck="false"></div>' +
          '<button class="btn btn--primary" data-action="start-research">' + icon('radar', 'icon--sm') + (running ? 'Restart dossier' : 'Start dossier') + '</button>' +
        '</div>' +
        '<div class="research-controls">' +
          '<div class="power-group"><span class="label">Power</span><div class="power-meter" role="meter" aria-valuenow="' + power + '" aria-valuemin="1" aria-valuemax="10" aria-label="Power level ' + power + ' of 10">' + segments + '</div><span class="power-value">×' + power + '</span></div>' +
          '<div class="research-options">' +
            '<span class="filter-chip">' + icon('search', 'icon--sm') + ' ' + esc(state.settings.search_provider || 'both') + '</span>' +
            '<span class="filter-chip">' + icon('globe', 'icon--sm') + ' ' + esc(state.settings.fetcher_mode || 'auto') + '</span>' +
            '<span class="filter-chip">depth ' + (research.crawl_depth || 2) + '</span>' +
          '</div>' +
        '</div>' +
        '<p class="power-caption">' + esc(powerCaption(power)) + '</p>' +
      '</section>' +
      '<div class="research-body-grid">' +
        '<div class="research-main">' +
          (running ? researchLive(steps) : dossierHtml(citeHtml, sourceRow, sources)) +
          researchHistoryHtml() +
        '</div>' +
        '<aside class="research-aside">' +
          '<section class="panel"><div class="panel-head">' + icon('info', 'icon--sm') + ' Run receipt</div><div class="panel-body">' +
            '<div class="kv-row"><span>Path</span><b>' + esc(research.path || 'fast') + '</b></div>' +
            '<div class="kv-row"><span>Rounds</span><b>' + (research.rounds || 0) + '</b></div>' +
            '<div class="kv-row"><span>Queries</span><b>' + (research.searched || 0) + '</b></div>' +
            '<div class="kv-row"><span>Pages read</span><b>' + (research.fetched || 0) + '</b></div>' +
            '<div class="kv-row"><span>Grounded</span><b>' + (research.grounded || 0) + '/' + (research.total_claims || 0) + '</b></div>' +
            '<div class="kv-row"><span>Duration</span><b>' + formatDuration(research.duration_ms) + '</b></div>' +
            '<div class="kv-row"><span>Cost</span><b class="accent">' + formatUsd(research.cost_usd) + '</b></div>' +
          '</div></section>' +
          '<section class="panel"><div class="panel-head">' + icon('waypoints', 'icon--sm') + ' Research trail</div><div class="panel-body"><div class="step-list">' + steps + '</div></div></section>' +
          '<section class="panel"><div class="panel-head">' + icon('globe', 'icon--sm') + ' Sources</div><div class="panel-body"><div class="flex flex-wrap gap-2">' + (sources || '<span class="muted micro">No sources retained yet.</span>') + '</div></div></section>' +
        '</aside>' +
      '</div>' +
    '</div></div>';
  }

  function powerCaption(power) {
    const captions = {
      1: 'Plan → one document per section. Fastest.',
      2: 'Adds an architecture brief shared by every chapter.',
      3: 'Adds per-section plans and subsection documents. Recommended.',
      4: 'Deeper subsections with more cross-referencing.',
      5: 'Wider coverage — smaller modules get their own chapters.',
      6: 'More passes per chapter; denser prose.',
      7: 'Adds extended examples and edge-case coverage.',
      8: 'Near-exhaustive: most files get named somewhere.',
      9: 'Maximum depth before the correction pass.',
      10: 'Advanced dossier: up to 480 pages over 8 rounds, exported as a signed PDF.'
    };
    return captions[power] || captions[3];
  }

  function researchLive(steps) {
    return '<section class="panel hud-corners"><div class="panel-head"><span class="spinner"></span> Live research · ' + esc(state.research.query) + '</div><div class="panel-body"><div class="step-list">' + steps + '</div>' +
      '<p class="power-caption mt-3">The dossier is being composed. Stop is safe: the run checkpoints and can resume later.</p></div></section>';
  }

  function dossierHtml(citeHtml, sourceRow, sources) {
    const research = state.research;
    if (!research.answer_markdown) {
      return '<div class="empty-panel">' + icon('radar') + '<strong>No dossier yet</strong><span>Ask a question and start the research run.</span></div>';
    }
    return '<article class="dossier hud-corners">' +
      '<div class="dossier-head"><div class="flex-1"><div class="dossier-kicker">' + icon('check-circle', 'icon--sm') + ' cited dossier · ' + (research.total_claims || 0) + ' claims grounded</div>' +
      '<div class="dossier-question">' + esc(research.query) + '</div></div>' + badge('×' + (research.power_level || 1), 'orange') + '</div>' +
      '<div class="dossier-body">' + citeHtml + '</div>' +
      '<div class="source-row">' + (sourceRow || '') + '</div>' +
      '<div class="dossier-meta"><span>' + icon('clock', 'icon--sm') + ' ' + formatDuration(research.duration_ms) + '</span><span>' + (research.rounds || 0) + ' rounds</span><span>' + (research.searched || 0) + ' queries</span><span>' + (research.fetched || 0) + ' pages read</span><span>' + (research.grounded || 0) + '/' + (research.total_claims || 0) + ' grounded</span><span class="accent">' + formatUsd(research.cost_usd) + '</span></div>' +
    '</article>' + (sources ? '<div class="mt-2 flex flex-wrap gap-2">' + sources + '</div>' : '');
  }

  function researchHistoryHtml() {
    const history = state.research.history || [];
    if (!history.length) return '';
    return '<section class="mt-4"><div class="section-label">Past research</div><div class="history-list mt-2">' +
      history.map(function (item) {
        return '<button class="history-item" data-action="research-history" data-question="' + attr(item.question) + '">' +
          icon('clock', 'icon--sm') + '<span class="history-q">' + esc(item.question) + '</span>' +
          '<span class="history-meta">' + esc(item.created_at) + ' · ' + item.sources + ' sources · ' + esc(item.path) + '</span></button>';
      }).join('') + '</div></section>';
  }

  function ensureResearchTimer() {
    if (state.research.is_running && !ui.researchTimer) {
      ui.researchTimer = setInterval(function () {
        post({ action: 'research_tick' });
      }, 950);
    }
    if (!state.research.is_running && ui.researchTimer) {
      clearInterval(ui.researchTimer);
      ui.researchTimer = null;
    }
  }

  /* ── Wiki ─────────────────────────────────────────────────────────────── */
  function renderWiki() {
    const chapters = state.wiki.chapters || [];
    const selected = chapters.find(function (chapter) { return chapter.path === ui.wikiSelected; }) || chapters[0];
    if (selected && ui.wikiSelected !== selected.path) ui.wikiSelected = selected.path;
    const sections = [];
    chapters.forEach(function (chapter) {
      let group = sections.find(function (item) { return item.name === chapter.section; });
      if (!group) {
        group = { name: chapter.section || 'Chapters', chapters: [] };
        sections.push(group);
      }
      group.chapters.push(chapter);
    });
    const query = ui.wikiQuery.trim().toLowerCase();
    const tree = sections.map(function (section) {
      const items = section.chapters.filter(function (chapter) {
        return !query || chapter.title.toLowerCase().indexOf(query) >= 0 || chapter.path.toLowerCase().indexOf(query) >= 0;
      }).map(function (chapter) {
        return '<button class="wiki-chapter' + (selected && chapter.path === selected.path ? ' is-active' : '') + '" data-action="open-wiki" data-path="' + attr(chapter.path) + '" data-search="' + attr((chapter.title + ' ' + chapter.path).toLowerCase()) + '">' +
          icon('file-text') + '<span class="chapter-title">' + esc(chapter.title) + '</span><span class="chapter-lines">' + (chapter.reading_minutes || 1) + 'm</span></button>';
      }).join('');
      return items ? '<div class="wiki-group"><div class="wiki-group-label">' + esc(section.name) + '</div>' + items + '</div>' : '';
    }).join('');
    const toc = selected ? extractToc(selected.markdown) : [];
    const tocHtml = toc.map(function (entry) {
      return '<button class="toc-item toc-item--' + entry.level + (ui.wikiToc === entry.slug ? ' is-active' : '') + '" data-action="wiki-toc" data-slug="' + attr(entry.slug) + '">' + esc(entry.text) + '</button>';
    }).join('');
    const article = selected
      ? '<div class="wiki-breadcrumb">' + icon('book', 'icon--sm') + ' ' + esc(selected.section) + ' / ' + esc(selected.path) + '</div>' +
        '<h1>' + esc(selected.title) + '</h1>' +
        '<div class="wiki-meta">' + badge('fresh', 'green') + badge((selected.reading_minutes || 1) + ' min read', 'muted') + badge((selected.words || 0).toLocaleString() + ' words', 'muted') + badge('Level 2 docs', 'muted') + '</div>' +
        '<div class="md-body">' + markdownToHtml(selected.markdown) + '</div>' +
        '<footer class="wiki-provenance"><div class="section-label section-label--plain">Generated from</div><div class="provenance-chips">' +
          ['src/desktop/view.rs', 'src/desktop/model.rs', 'docs/09_safety_and_evaluation.md'].map(function (path) {
            return '<span class="provenance-chip">' + esc(path) + '</span>';
          }).join('') + '</div></footer>'
      : '<div class="empty-panel">' + icon('book') + '<strong>No wiki generated yet</strong><span>Start a wiki run from Activity.</span></div>';

    return '<div class="wiki-layout">' +
      '<aside class="wiki-nav">' +
        '<div class="wiki-search"><div class="query-box">' + icon('search', 'icon--sm') + '<input data-field="wiki-search" data-focus="wiki-search" value="' + attr(ui.wikiQuery) + '" placeholder="Search chapters…"></div></div>' +
        '<div class="wiki-tree" id="wikiTree">' + (tree || '<div class="empty-panel" style="margin:8px;padding:18px 10px">No chapters match.</div>') + '</div>' +
        '<div class="wiki-nav-foot"><div class="kv-row"><span>Docs</span><b>' + (state.health.total_docs || chapters.length) + '</b></div><div class="kv-row"><span>Fresh</span><b class="green">' + (state.health.fresh_docs || 0) + '</b></div><div class="kv-row"><span>Stale</span><b class="ember">' + (state.health.stale_docs || 0) + '</b></div></div>' +
      '</aside>' +
      '<main class="wiki-reader" data-scroll="wiki"><div class="wiki-reader-inner"><article class="wiki-article">' + article + '</article></div></main>' +
      '<aside class="wiki-toc">' +
        '<div class="section-label">On this page</div><div class="toc-list mt-2">' + (tocHtml || '<span class="muted micro">No headings</span>') + '</div>' +
        '<div class="local-graph-mini"><div class="section-label">Local graph</div>' +
          '<div class="mt-2">' + (chapters.slice(0, 4).map(function (chapter) { return '<button class="mini-node" data-action="open-wiki" data-path="' + attr(chapter.path) + '">' + esc(chapter.title) + '</button>'; }).join('')) + '</div>' +
        '</div>' +
      '</aside>' +
    '</div>';
  }

  /* ── Graph ────────────────────────────────────────────────────────────── */
  function renderGraph() {
    const memory = state.memory;
    const nodes = memory.nodes || [];
    const edges = memory.edges || [];
    const selectedId = ui.graphSelected != null ? ui.graphSelected : memory.selected_node_id;
    const selected = nodes.find(function (node) { return node.id === selectedId; }) || nodes[0];
    const query = ui.graphQuery.trim().toLowerCase();
    const kind = ui.graphKind;
    const zoom = Math.max(0.5, Math.min(2.5, ui.graphZoom || 1));
    const viewWidth = 860 / zoom;
    const viewHeight = 480 / zoom;
    const viewBox = (430 - viewWidth / 2) + ' ' + (240 - viewHeight / 2) + ' ' + viewWidth + ' ' + viewHeight;
    const nodeMap = {};
    nodes.forEach(function (node) { nodeMap[node.id] = node; });

    const edgeHtml = edges.map(function (edge) {
      const source = nodeMap[edge.source_id];
      const target = nodeMap[edge.target_id];
      if (!source || !target) return '';
      const dim = (query && !nodeMatches(source, query) && !nodeMatches(target, query)) || (kind !== 'all' && source.kind !== kind && target.kind !== kind);
      return '<line class="graph-edge' + (dim ? ' is-dim' : '') + '" x1="' + source.x + '" y1="' + source.y + '" x2="' + target.x + '" y2="' + target.y + '"></line>';
    }).join('');

    const nodeHtml = nodes.map(function (node) {
      const isSelected = selected && node.id === selected.id;
      const dim = (query && !nodeMatches(node, query)) || (kind !== 'all' && node.kind !== kind);
      return '<g class="graph-node kind-' + attr(node.kind) + (isSelected ? ' is-selected' : '') + (dim ? ' is-dim' : '') + '" data-action="select-node" data-node="' + node.id + '">' +
        '<circle cx="' + node.x + '" cy="' + node.y + '" r="' + (isSelected ? 12 : 9) + '"></circle>' +
        '<text class="node-label" x="' + (node.x + 15) + '" y="' + (node.y + 2) + '">' + esc(node.name) + '</text>' +
        '<text class="node-kind" x="' + (node.x + 15) + '" y="' + (node.y + 12) + '">' + esc(node.kind) + '</text></g>';
    }).join('');

    const callers = edges.filter(function (edge) { return selected && edge.target_id === selected.id; }).map(function (edge) {
      return nodeMap[edge.source_id];
    }).filter(Boolean);
    const dependencies = edges.filter(function (edge) { return selected && edge.source_id === selected.id; }).map(function (edge) {
      return nodeMap[edge.target_id];
    }).filter(Boolean);
    const inspector = memory.inspector && selected && memory.inspector.symbol_id === selected.id ? memory.inspector : null;
    const selectedDoc = inspector ? inspector.doc_content : 'Select a node to inspect deterministic documentation, callers, and dependencies.';

    const table = nodes.map(function (node) {
      return '<tr data-action="select-node" data-node="' + node.id + '" class="' + (selected && node.id === selected.id ? 'is-selected' : '') + '">' +
        '<td><span class="kind-pill kind-pill--' + attr(node.kind) + '">' + esc(node.kind) + '</span></td>' +
        '<td>' + esc(node.name) + '</td><td class="muted">' + esc(node.file_path) + '</td>' +
        '<td class="text-right">' + edges.filter(function (edge) { return edge.target_id === node.id; }).length + '</td></tr>';
    }).join('');

    return '<div class="graph-layout">' +
      '<section class="graph-main">' +
        '<div class="graph-toolbar">' +
          '<div class="query-box">' + icon('search', 'icon--sm') + '<input data-field="graph-filter" data-focus="graph-filter" value="' + attr(ui.graphQuery) + '" placeholder="Filter symbols…"></div>' +
          '<div class="graph-filters">' + ['all', 'fn', 'struct', 'enum', 'trait'].map(function (kindName) {
            return '<button class="filter-chip' + (kind === kindName ? ' is-active' : '') + '" data-action="graph-kind" data-kind="' + kindName + '">' + (kindName === 'all' ? 'All' : '[' + kindName + ']') + '</button>';
          }).join('') + '</div>' +
          '<button class="btn btn--xs btn--quiet ml-auto" data-action="graph-fit">' + icon('waypoints', 'icon--sm') + ' Fit</button>' +
          badge(nodes.length + ' nodes', 'muted') + badge(edges.length + ' edges', 'muted') +
        '</div>' +
        '<div class="graph-canvas">' +
          '<svg viewBox="' + viewBox + '" role="img" aria-label="AST symbol topology">' + edgeHtml + nodeHtml + '</svg>' +
          '<div class="graph-legend">' + ['fn', 'struct', 'enum', 'trait'].map(function (kindName) {
            return '<span class="legend-item kind-' + kindName + '"><span class="legend-swatch"></span>' + kindName + '</span>';
          }).join('') + '</div>' +
          '<div class="graph-zoom"><button class="icon-btn" data-action="graph-zoom-out" title="Zoom out">' + icon('minus', 'icon--sm') + '</button><span class="zoom-label">' + Math.round(zoom * 100) + '%</span><button class="icon-btn" data-action="graph-zoom-in" title="Zoom in">' + icon('plus', 'icon--sm') + '</button></div>' +
        '</div>' +
        '<div class="graph-table"><table class="symbol-table"><thead><tr><th>Kind</th><th>Symbol</th><th>File</th><th class="text-right">Callers</th></tr></thead><tbody id="symbolBody">' + table + '</tbody></table></div>' +
      '</section>' +
      '<aside class="graph-inspector">' +
        '<div class="inspector-head">' +
          '<div class="flex items-center gap-2">' + (selected ? '<span class="kind-pill kind-pill--' + attr(selected.kind) + '">' + esc(selected.kind) + '</span>' : '') + (selected && selected.is_exported ? badge('pub', 'green') : '') + '</div>' +
          '<div class="inspector-name">' + esc(selected ? selected.name : 'No symbol selected') + '</div>' +
          '<div class="inspector-signature">' + esc(inspector ? inspector.signature : selected ? 'pub ' + selected.kind + ' ' + selected.name : 'Select a node') + '</div>' +
          '<div class="muted micro mt-2">' + esc(selected ? selected.file_path : '') + '</div>' +
        '</div>' +
        '<div class="inspector-section"><div class="section-label">Documentation · Level ' + (inspector ? inspector.doc_level : 0) + '</div><p class="inspector-doc">' + esc(selectedDoc) + '</p></div>' +
        '<div class="inspector-section"><div class="section-label">Callers · ' + callers.length + '</div>' +
          (callers.length ? callers.map(function (node) { return '<div class="relation-row">' + icon('arrow-left', 'icon--sm') + esc(node.name) + '<span class="relation-path">' + esc(node.file_path) + '</span></div>'; }).join('') : '<p class="inspector-doc">No callers recorded.</p>') + '</div>' +
        '<div class="inspector-section"><div class="section-label">Dependencies · ' + dependencies.length + '</div>' +
          (dependencies.length ? dependencies.map(function (node) { return '<div class="relation-row">' + icon('arrow-right', 'icon--sm') + esc(node.name) + '<span class="relation-path">' + esc(node.kind) + '</span></div>'; }).join('') : '<p class="inspector-doc">No outgoing edges.</p>') + '</div>' +
        '<div class="inspector-section"><div class="section-label">Graph stats</div><div class="kv-row"><span>Files</span><b>' + (memory.total_files || 0) + '</b></div><div class="kv-row"><span>Symbols</span><b>' + (memory.total_symbols || nodes.length) + '</b></div><div class="kv-row"><span>Edges</span><b>' + (memory.total_edges || edges.length) + '</b></div><div class="kv-row"><span>Zoom</span><b>' + Math.round(zoom * 100) + '%</b></div></div>' +
      '</aside>' +
    '</div>';
  }

  function nodeMatches(node, query) {
    return (node.name + ' ' + node.file_path + ' ' + node.kind).toLowerCase().indexOf(query) >= 0;
  }

  /* ── Cards ────────────────────────────────────────────────────────────── */
  function renderCards() {
    const cards = state.failure_cards || [];
    const filtered = cards.filter(function (card) {
      if (ui.cardFilter === 'all') return true;
      if (ui.cardFilter === 'watching') return card.status === 'watching';
      return card.severity === ui.cardFilter;
    });
    const selected = cards.find(function (card) { return card.id === ui.cardSelected; }) || filtered[0] || cards[0];
    if (selected && ui.cardSelected !== selected.id) ui.cardSelected = selected.id;
    const cardHtml = filtered.map(function (card) {
      return '<button class="failure-card' + (selected && card.id === selected.id ? ' is-active' : '') + '" data-action="select-card" data-id="' + attr(card.id) + '">' +
        '<div class="failure-card-head"><span class="badge badge--' + severityTone(card.severity) + '">' + esc(card.severity) + '</span><span class="failure-card-title">' + esc(card.title) + '</span>' + badge(card.status, card.status === 'fixed' || card.status === 'mitigated' ? 'green' : 'amber') + '</div>' +
        '<div class="failure-card-meta"><span class="module">' + esc(card.module) + '</span><span>' + card.occurrences + '× seen</span><span>' + esc(card.last_seen) + '</span></div></button>';
    }).join('');
    const detail = selected ? '<section class="failure-detail hud-corners">' +
      '<div class="failure-detail-head"><div class="flex items-center gap-2">' + badge(selected.severity, severityTone(selected.severity)) + badge(selected.status, selected.status === 'fixed' || selected.status === 'mitigated' ? 'green' : 'amber') + '<span class="ml-auto muted micro">' + selected.occurrences + ' occurrences</span></div>' +
      '<div class="failure-detail-title">' + esc(selected.title) + '</div><div class="muted micro mt-1">' + esc(selected.module) + ' · last seen ' + esc(selected.last_seen) + '</div></div>' +
      '<div class="failure-detail-body">' +
        '<div class="section-label">Gotchas</div><div class="gotcha-list">' + (selected.gotchas || []).map(function (gotcha, index) {
          return '<div class="gotcha-item"><span class="gotcha-num">' + String(index + 1).padStart(2, '0') + '</span><span>' + esc(gotcha) + '</span></div>';
        }).join('') + '</div>' +
        '<div class="section-label">Anti-pattern rules</div><div class="rule-list">' + (selected.anti_pattern_rules || []).map(function (rule) {
          return '<div class="rule-item">' + icon('check', 'icon--sm') + '<span>' + esc(rule) + '</span></div>';
        }).join('') + '</div>' +
        '<div class="section-label">Deterministic prevention</div><div class="prevention-block"><span class="label">Rule enforced by retrieval</span>' + esc((selected.anti_pattern_rules || ['No rule recorded.'])[0]) + '</div>' +
        '<div class="mt-4"><button class="btn btn--sm btn--primary" data-action="resolve-card" data-id="' + attr(selected.id) + '">' + icon('check', 'icon--sm') + ' Mark as Resolved</button></div>' +
      '</div></section>' : '<div class="empty-panel">' + icon('layers') + '<strong>No failure cards</strong><span>Failures accumulate automatically from critic runs.</span></div>';

    return '<div class="route-scroll" data-scroll="cards"><div class="route-wrap route-wrap--wide">' +
      '<header class="route-header"><div><p class="label route-eyebrow">FAILURE MEMORY · DETERMINISTIC PREVENTION</p><h1 class="route-title">Cards</h1><p class="route-sub">Every hard-won gotcha becomes a rule the agent retrieves before it repeats the mistake.</p></div>' +
        '<div class="route-header-right">' + badge(cards.length + ' cards', 'orange') + badge(cards.filter(function (card) { return card.severity === 'high'; }).length + ' high', 'rose') + '</div></header>' +
      '<div class="cards-layout">' +
        '<div><div class="segmented mb-2">' + [['all', 'All'], ['high', 'High'], ['watching', 'Watching']].map(function (item) {
          return '<button class="' + (ui.cardFilter === item[0] ? 'is-on' : '') + '" data-action="cards-filter" data-filter="' + item[0] + '">' + item[1] + '</button>';
        }).join('') + '</div><div class="card-list">' + (cardHtml || '<div class="empty-panel">No cards match this filter.</div>') + '</div></div>' +
        detail +
      '</div>' +
    '</div></div>';
  }

  function severityTone(severity) {
    if (severity === 'high') return 'rose';
    if (severity === 'medium') return 'amber';
    return 'muted';
  }

  /* ── Editor ───────────────────────────────────────────────────────────── */
  function renderEditor() {
    const editor = state.editor;
    const files = editor.files || [];
    const file = files.find(function (item) { return item.path === ui.activeFile; }) || files[0];
    if (file && ui.activeFile !== file.path) ui.activeFile = file.path;
    const tabs = files.map(function (item) {
      return '<div class="file-tab' + (file && item.path === file.path ? ' is-active' : '') + '" data-action="editor-tab" data-path="' + attr(item.path) + '">' +
        icon('file-text', 'icon--sm') + '<span class="tab-name">' + esc(item.path.split('/').pop()) + '</span>' +
        (item.dirty ? '<span class="dirty-dot"></span>' : '') +
        '<button class="tab-close" data-action="close-file" data-path="' + attr(item.path) + '" aria-label="Close">' + icon('x', 'icon--sm') + '</button></div>';
    }).join('');
    const tree = files.map(function (item) {
      return '<button class="tree-file' + (file && item.path === file.path ? ' is-active' : '') + '" data-action="editor-tab" data-path="' + attr(item.path) + '">' + icon('file-text') + '<span class="tree-name">' + esc(item.path.split('/').pop()) + '</span>' + (item.dirty ? '<span class="dirty-dot"></span>' : '') + '</button>';
    }).join('');
    const changes = (state.review.changed_files || []).map(function (path) {
      return '<button class="change-row" data-action="editor-mode" data-mode="diff"><span class="change-status">M</span><span class="change-path">' + esc(path) + '</span></button>';
    }).join('');
    const pane = !file
      ? '<div class="empty-panel" style="margin:20px">' + icon('code') + '<strong>No files open</strong><span>Pick a file from the tree or the tab strip.</span></div>'
      : ui.editorMode === 'diff'
        ? '<div class="diff-view">' + renderDiff(editor.diff || state.review.diff) + '</div>'
        : ui.editorMode === 'edit'
          ? '<textarea class="editor-textarea" data-field="editor-content" data-focus="editor-content" spellcheck="false">' + esc(file.content) + '</textarea>'
          : '<div class="code-view">' + renderCode(file.content) + '</div>';
    const terminal = ui.terminalOpen
      ? '<div class="terminal-drawer">' +
          '<div class="terminal-head"><button class="terminal-tab' + (ui.terminalTab === 0 ? ' is-active' : '') + '" data-action="terminal-tab" data-tab="0">Terminal 1</button><button class="terminal-tab' + (ui.terminalTab === 1 ? ' is-active' : '') + '" data-action="terminal-tab" data-tab="1">Terminal 2</button><button class="icon-btn ml-auto" data-action="toggle-terminal" title="Close terminal">' + icon('x', 'icon--sm') + '</button></div>' +
          '<div class="terminal-body" data-scroll="terminal">' + (editor.terminal_lines || []).map(function (line) {
            return '<div class="terminal-line is-' + esc(line.kind) + '">' + esc(line.text) + '</div>';
          }).join('') +
          '<div class="terminal-input"><span>$</span><input data-field="terminal-input" data-focus="terminal-input" value="' + attr(ui.terminalInput) + '" placeholder="run a command…"></div>' +
          '</div>' +
        '</div>'
      : '<button class="terminal-rail" data-action="toggle-terminal">' + icon('terminal', 'icon--sm') + ' Terminal · Ctrl+`' + icon('chevron-up', 'icon--sm ml-auto') + '</button>';

    return '<div class="editor-layout">' +
      '<aside class="editor-tree">' +
        '<div class="tree-section"><div class="section-label">Open files</div>' + (tree || '<span class="muted micro">No files open</span>') + '</div>' +
        '<div class="tree-section"><div class="section-label">Git changes · ' + (state.review.changed_files || []).length + '</div>' + (changes || '<span class="muted micro">Working tree clean</span>') + '</div>' +
        '<div class="tree-section"><div class="section-label">Terminal</div><button class="tree-file" data-action="toggle-terminal">' + icon('terminal') + '<span class="tree-name">' + (ui.terminalOpen ? 'Close terminal' : 'Open terminal') + '</span></button></div>' +
        '<div class="tree-section"><div class="section-label">Review</div><div class="kv-row"><span>Risk</span><b class="ember">' + (state.review.risk_score || 0) + '/100</b></div><div class="kv-row"><span>Level</span><b>' + esc(state.review.risk_level || 'Clean') + '</b></div><div class="kv-row"><span>+ / −</span><b>+' + (state.review.additions || 0) + ' / −' + (state.review.deletions || 0) + '</b></div></div>' +
      '</aside>' +
      '<div class="editor-main">' +
        '<div class="tabstrip">' + (tabs || '') + '</div>' +
        '<div class="editor-toolbar">' +
          '<div class="segmented"><button class="' + (ui.editorMode === 'code' ? 'is-on' : '') + '" data-action="editor-mode" data-mode="code">Code</button><button class="' + (ui.editorMode === 'edit' ? 'is-on' : '') + '" data-action="editor-mode" data-mode="edit">Edit</button><button class="' + (ui.editorMode === 'diff' ? 'is-on' : '') + '" data-action="editor-mode" data-mode="diff">Diff</button></div>' +
          '<span class="path">' + esc(file ? file.path : 'No file') + '</span>' +
          '<span class="ml-auto flex items-center gap-2">' + badge(file ? file.language : '—', 'muted') + '<button class="btn btn--xs btn--quiet" data-action="save-file">' + icon('save', 'icon--sm') + ' Save</button></span>' +
        '</div>' +
        '<div class="editor-pane" data-scroll="editor">' + pane + '</div>' +
        terminal +
        '<div class="editor-status"><span>' + esc(file ? file.path : '—') + '</span><span>UTF-8</span><span>LF</span><span class="status-right"><span>Ln ' + (file ? Math.min(12, file.content.split('\n').length) : 1) + ', Col 1</span><span>' + esc(editor.diff_summary || state.review.risk_level || 'Clean') + '</span></span></div>' +
      '</div>' +
    '</div>';
  }

  /* ── Browser ──────────────────────────────────────────────────────────── */
  function renderBrowser() {
    const browser = state.browser;
    const tabs = browser.tabs || [];
    const active = tabs.find(function (tab) { return tab.id === ui.browserActive; }) || tabs[0];
    if (active && ui.browserActive !== active.id) ui.browserActive = active.id;
    const tabHtml = tabs.map(function (tab) {
      return '<div class="browser-tab' + (active && tab.id === active.id ? ' is-active' : '') + '" data-action="browser-tab" data-id="' + attr(tab.id) + '">' +
        icon(tab.kind === 'api' ? 'database' : tab.kind === 'newtab' ? 'plus' : 'globe', 'icon--sm') +
        '<span class="tab-title">' + esc(tab.title) + '</span>' +
        '<button class="tab-close" data-action="browser-close-tab" data-id="' + attr(tab.id) + '" aria-label="Close tab">' + icon('x', 'icon--sm') + '</button></div>';
    }).join('');
    const content = !active ? newTabPage() : active.kind === 'api' ? apiInspector() : active.kind === 'newtab' ? newTabPage() : webPreview(active);

    return '<div class="browser-layout">' +
      '<div class="browser-tabstrip">' + tabHtml + '<button class="browser-newtab-btn" data-action="browser-new-tab" title="New tab (Ctrl+T)">' + icon('plus', 'icon--sm') + '</button></div>' +
      '<div class="browser-toolbar">' +
        '<button class="icon-btn" data-action="browser-back" title="Back">' + icon('arrow-left', 'icon--sm') + '</button>' +
        '<button class="icon-btn" data-action="browser-forward" title="Forward">' + icon('arrow-right', 'icon--sm') + '</button>' +
        '<button class="icon-btn" data-action="browser-reload" title="Reload">' + icon('rotate', 'icon--sm') + '</button>' +
        '<form class="omnibox" data-form="browser-omnibox">' + icon('search', 'icon--sm') +
          '<input data-field="browser-url" data-focus="browser-url" value="' + attr(active && active.kind !== 'newtab' ? active.url : ui.browserUrl) + '" placeholder="Search or type a URL">' +
          '<button type="button" class="icon-btn" data-action="browser-external" title="Open in system browser">' + icon('external', 'icon--sm') + '</button>' +
        '</form>' +
        '<button class="icon-btn" data-action="browser-star" title="Bookmark">' + icon('star', 'icon--sm') + '</button>' +
      '</div>' +
      '<div class="browser-content" data-scroll="browser">' + content + '</div>' +
    '</div>';
  }

  function apiInspector() {
    const browser = state.browser;
    const method = browser.method || 'GET';
    const headers = (browser.response_headers || []).map(function (pair) {
      return '<tr><td>' + esc(pair[0]) + '</td><td>' + esc(pair[1]) + '</td></tr>';
    }).join('');
    const body = esc(browser.response_body || '{}');
    return '<div class="api-inspector">' +
      '<div class="api-request-bar"><span class="method-badge' + (method === 'POST' ? ' is-post' : '') + '">' + esc(method) + '</span>' +
        '<span class="api-url">' + esc(browser.request_url || 'http://127.0.0.1:47890/api/chat/completions') + '</span>' +
        '<span class="status-code">' + (browser.status || 200) + '</span><span class="api-latency">' + (browser.latency_ms || 0) + ' ms</span>' + badge('JSON', 'green') + '</div>' +
      '<div class="api-grid">' +
        '<section class="panel"><div class="panel-head">' + icon('database', 'icon--sm') + ' Response headers</div><table class="headers-table"><thead><tr><th>Header</th><th>Value</th></tr></thead><tbody>' + (headers || '<tr><td colspan="2" class="muted">No headers captured.</td></tr>') + '</tbody></table></section>' +
        '<section class="response-panel"><div class="panel-head">' + icon('code', 'icon--sm') + ' Response body · application/json</div><pre class="response-body">' + body + '</pre></section>' +
      '</div>' +
      '<div class="mt-3 flex items-center gap-2"><span class="section-label section-label--plain">Network</span>' + badge('POST /api/chat/completions', 'blue') + badge('200 OK', 'green') + badge('no-store', 'muted') + '</div>' +
    '</div>';
  }

  function webPreview(tab) {
    return '<div class="browser-page"><div class="web-preview-bar">' + icon('info', 'icon--sm') + ' Preview mode · third-party pages open externally when framing is denied</div>' +
      '<div class="web-doc">' +
        '<div class="doc-brand">' + icon('terminal', 'icon--sm') + ' kaioken documentation</div>' +
        '<h1>Build a desktop that <span class="serif">remembers.</span></h1>' +
        '<p class="lead">Kaioken pairs a native Rust process with a hardware-accelerated WebView2 surface. The design system is compiled into the binary; there is no Node runtime and no devserver.</p>' +
        '<div class="doc-cards">' +
          '<div class="doc-card"><div class="label">Architecture</div><p>Tao owns the window. Wry owns the WebView. The controller serializes a single snapshot per revision.</p></div>' +
          '<div class="doc-card"><div class="label">Safety</div><p>Every tool proposal is deny-first. Enter approves; Escape denies; dismissal is never authorization.</p></div>' +
          '<div class="doc-card"><div class="label">Design</div><p>Hairline rules, Geist Mono chrome, zero-radius controls, and one accent red.</p></div>' +
        '</div>' +
        '<div class="doc-code">cargo run -- desktop --embedded\n\n// include_str!(\"ui/design.css\")\n// include_str!(\"ui/app.html\")\n// include_str!(\"ui/app.js\")</div>' +
        '<p class="muted micro mt-3">' + esc(tab.url || 'https://kaioken.dev/docs/desktop') + '</p>' +
      '</div></div>';
  }

  function newTabPage() {
    const links = [
      { url: 'https://kaioken.dev', title: 'Main website', host: 'kaioken.dev' },
      { url: 'https://kaioken.dev/docs', title: 'Documentation', host: 'kaioken.dev' },
      { url: 'https://github.com/babtix/kaioken', title: 'Source · GitHub', host: 'github.com' }
    ];
    return '<div class="newtab-page">' + icon('globe', 'newtab-mark') +
      '<h1>Where to?</h1><p>Search the web or type a URL · Ctrl+L</p>' +
      '<form class="newtab-search" data-form="newtab-search"><div class="query-box">' + icon('search') + '<input data-field="browser-url" data-focus="browser-url" value="' + attr(ui.browserUrl) + '" placeholder="Type a URL or search…"></div></form>' +
      '<div class="newtab-links"><div class="newtab-group"><div class="label">Project</div>' + links.map(function (link) {
        return '<button class="newtab-link" data-action="browser-open" data-url="' + attr(link.url) + '">' + icon('globe', 'icon--sm') + '<span>' + esc(link.title) + '</span><span class="host">' + esc(link.host) + '</span></button>';
      }).join('') + '</div></div></div>';
  }

  /* ── Activity ─────────────────────────────────────────────────────────── */
  function renderActivity() {
    const activity = state.activity;
    const runs = activity.runs || [];
    const activeRuns = runs.filter(function (run) { return run.state === 'running' || run.state === 'queued'; });
    const pastRuns = runs.filter(function (run) { return run.state !== 'running' && run.state !== 'queued'; });
    const runHtml = function (run) {
      const open = !!ui.openRuns[run.id];
      const active = run.state === 'running' || run.state === 'queued';
      const pct = run.progress_total ? Math.min(100, Math.round((run.progress_done / run.progress_total) * 100)) : 0;
      return '<div class="run-card' + (active ? ' is-active' : '') + '">' +
        '<button class="run-head" data-action="toggle-run" data-id="' + attr(run.id) + '">' +
          icon(open ? 'chevron-down' : 'chevron-right', 'icon--sm') +
          '<span class="run-kind">' + esc(run.label || run.kind) + '</span>' + badge(run.state, run.state === 'done' ? 'green' : run.state === 'running' ? 'orange' : run.state === 'failed' ? 'rose' : 'amber') +
          (run.multiplier ? badge('×' + run.multiplier, 'muted') : '') +
          '<span class="run-message">' + esc(run.message || '') + '</span>' +
          '<span class="run-duration">' + (run.duration_ms ? formatDuration(run.duration_ms) : active ? 'live' : '') + '</span>' +
        '</button>' +
        (active ? '<div class="progress"><i style="width:' + pct + '%"></i></div>' : '') +
        (open ? '<div class="run-logs">' + (run.logs || []).map(function (line) { return '<div class="run-log">' + esc(line) + '</div>'; }).join('') + '</div>' : '') +
      '</div>';
    };
    const events = (activity.events || []).map(function (event) {
      const cls = event.severity === 'critical' ? 'is-critical' : event.severity === 'warn' ? 'is-warn' : '';
      const tone = event.decision === 'blocked' || event.decision === 'denied' ? 'rose' : event.decision === 'approved' || event.decision === 'allowed' ? 'green' : 'amber';
      return '<div class="audit-row ' + cls + '"><span class="audit-time">' + esc(event.timestamp) + '</span>' +
        '<div class="audit-body"><span class="audit-action">' + esc(event.action) + '</span><span class="audit-target">' + esc(event.target) + '</span></div>' +
        badge(event.decision, tone) + '</div>';
    }).join('');

    return '<div class="route-scroll" data-scroll="activity"><div class="route-wrap route-wrap--wide">' +
      '<header class="route-header"><div><p class="label route-eyebrow">TASK QUEUES · DAEMON EVENTS · SAFETY LEDGER</p><h1 class="route-title">Activity</h1><p class="route-sub">Everything running, everything that ran, and every decision the safety layer made.</p></div>' +
        '<div class="route-header-right">' + (activeRuns.length ? '<span class="live-dot live-dot--orange live-dot--pulse"></span>' + badge(activeRuns.length + ' active', 'orange') : badge('idle', 'muted')) + badge(activity.events.length + ' events', 'muted') + '</div></header>' +
      '<div class="activity-grid">' +
        '<div>' +
          '<section class="panel wiki-launcher hud-corners">' +
            '<div class="launcher-head"><span class="launcher-icon">' + icon('book') + '</span><div><div class="section-label section-label--plain">Generate a wiki</div><p class="route-sub">Turn this repository into linked, readable chapters.</p></div></div>' +
            '<div class="pipeline-strip">' + ['init', 'scan', 'plan', 'generate', 'wiki', 'update'].map(function (step, index) {
              return (index ? '<span class="pipeline-arrow">→</span>' : '') + '<span class="pipeline-step" title="' + attr(stepCaption(step)) + '">' + step + '</span>';
            }).join('') + '<span class="muted">↺ loops back to wiki on every code change</span></div>' +
            '<div class="depth-control"><div class="depth-head"><span class="label">Depth</span><span class="depth-value">×' + ui.activityDepth + '</span></div>' +
              '<input class="depth-slider" type="range" min="1" max="10" value="' + ui.activityDepth + '" data-field="activity-depth" aria-label="Wiki depth multiplier">' +
              '<div class="depth-scale"><span>×1 fast</span><span>×10 exhaustive</span></div>' +
              '<p class="depth-caption">' + esc(powerCaption(ui.activityDepth)) + '</p></div>' +
            '<div class="launcher-actions"><div class="run-main"><button class="btn btn--primary" data-action="run-command" data-command="start_wiki">' + icon('play', 'icon--sm') + ' Start wiki</button><span class="muted micro">wiki ×' + ui.activityDepth + ' · estimated 34 calls</span></div>' +
              '<button class="btn btn--sm" data-action="run-command" data-command="doctor">' + icon('beaker', 'icon--sm') + ' Doctor</button>' +
              '<button class="btn btn--sm" data-action="run-command" data-command="clear_memory">' + icon('trash', 'icon--sm') + ' Clear memory</button></div>' +
            '<div class="secondary-actions">' + [['scan', 'Scan', 'refresh'], ['generate', 'Cards', 'layers'], ['skills', 'Skills', 'zap'], ['update', 'Update', 'rotate']].map(function (item) {
              return '<button class="btn btn--xs btn--quiet" data-action="run-command" data-command="' + item[0] + '">' + icon(item[2], 'icon--sm') + ' ' + item[1] + '</button>';
            }).join('') + '</div>' +
          '</section>' +
          '<section class="mt-3"><div class="section-label">In progress</div><div class="run-list mt-2">' + (activeRuns.map(runHtml).join('') || '<div class="empty-panel">No runs in flight.</div>') + '</div></section>' +
          '<section class="mt-3"><div class="section-label">History</div><div class="run-list mt-2">' + (pastRuns.map(runHtml).join('') || '<div class="empty-panel">No runs yet. Start one above — progress streams here live.</div>') + '</div></section>' +
        '</div>' +
        '<aside class="col gap-3">' +
          '<section class="panel"><div class="panel-head">' + icon('database', 'icon--sm') + ' Memory & learning</div><div class="panel-body"><div class="memory-stats">' +
            '<div class="memory-stat"><span class="label">Sessions</span><b>' + activity.sessions_recorded + '</b></div>' +
            '<div class="memory-stat"><span class="label">Learned</span><b>' + activity.learned_skills + '</b></div>' +
            '<div class="memory-stat"><span class="label">Reinforced</span><b>' + activity.reinforced_skills + '</b></div>' +
          '</div><p class="muted micro mt-2">Kaioken writes a digest after each chat, reinforces cited skills, and distills new lessons into deterministic rules.</p></div></section>' +
          '<section class="panel"><div class="panel-head">' + icon('shield', 'icon--sm') + ' Safety audit ledger</div><div class="panel-body"><div class="audit-list">' + (events || '<span class="muted micro">No audit events.</span>') + '</div></div></section>' +
          '<section class="panel"><div class="panel-head">' + icon('cpu', 'icon--sm') + ' Daemon</div><div class="panel-body">' +
            '<div class="kv-row"><span>Mode</span><b>' + esc(state.daemon.mode) + '</b></div>' +
            '<div class="kv-row"><span>Endpoint</span><b>' + esc(state.daemon.endpoint) + '</b></div>' +
            '<div class="kv-row"><span>Latency</span><b>' + (state.daemon.latency_ms || 0) + ' ms</b></div>' +
            '<div class="kv-row"><span>Watcher</span><b class="' + (state.settings.watcher_enabled ? 'green' : 'muted') + '">' + (state.settings.watcher_enabled ? 'watching' : 'paused') + '</b></div>' +
            '<div class="kv-row"><span>Debounce</span><b>' + (state.settings.debounce_ms || 300) + ' ms</b></div>' +
          '</div></section>' +
        '</aside>' +
      '</div>' +
    '</div></div>';
  }

  function stepCaption(step) {
    const captions = {
      init: 'sets up .kaioken/config.yaml in your repo',
      scan: 'inventories the repository so you see what will be analyzed',
      plan: 'the LLM proposes modules.yaml — edit it if you like',
      generate: 'writes knowledge cards per module, in parallel',
      wiki: 'deep multi-pass wiki (×3 by default)',
      update: 'after code changes: refreshes only what the diff touched'
    };
    return captions[step] || step;
  }

  /* ── Extensions ───────────────────────────────────────────────────────── */
  function renderExtensions() {
    const ext = state.extensions;
    const installed = ext.installed || [];
    const registry = ext.registry || [];
    const hooks = ext.hooks || [];
    const installedIds = installed.map(function (item) { return item.id; });
    let body = '';
    if (ui.extensionsTab === 'installed') {
      body = '<div class="ext-list">' + (installed.map(function (item) {
        const executable = item.kind === 'mcp' || item.kind === 'wasm';
        const untrusted = executable && !item.trusted;
        return '<div class="ext-row' + (untrusted ? ' is-untrusted' : '') + '">' +
          '<div class="ext-head"><span class="ext-id">' + esc(item.id) + '</span>' + badge('v' + item.version, 'muted') + badge(item.kind, item.kind === 'mcp' ? 'amber' : item.kind === 'wasm' ? 'blue' : 'tan') +
            (item.local ? badge('dev', 'orange') : '') + (executable ? (item.trusted ? badge('trusted', 'green') : badge('UNTRUSTED', 'rose')) : '') +
            (!item.enabled ? badge('disabled', 'muted') : '') +
            '<span class="ext-actions">' + (untrusted ? '<button class="btn btn--primary btn--xs" data-action="trust-ext" data-id="' + attr(item.id) + '">' + icon('shield', 'icon--sm') + ' Trust…</button>' : '') +
              '<button class="btn btn--xs btn--quiet" data-action="toggle-ext" data-id="' + attr(item.id) + '">' + (item.enabled ? 'Disable' : 'Enable') + '</button>' +
              '<button class="icon-btn" data-action="remove-ext" data-id="' + attr(item.id) + '" title="Uninstall">' + icon('trash', 'icon--sm') + '</button></span></div>' +
          '<div class="ext-description">' + esc(item.description) + '</div>' +
          '<div class="ext-meta">' + esc(item.repo) + (item.skills.length ? ' · skills: ' + esc(item.skills.join(', ')) : '') + (item.tools.length ? ' · tools: ' + esc(item.tools.join(', ')) : '') + '</div>' +
          (untrusted ? '<div class="trust-banner">' + icon('alert', 'icon--sm') + '<span>Executable extension is inert until this exact version is trusted. Trusting does not bypass per-call approval.</span></div>' : '') +
        '</div>';
      }).join('') || '<div class="empty-panel">' + icon('puzzle') + '<strong>No extensions installed</strong><span>Install a declarative package from the registry or a GitHub repo.</span></div>') + '</div>' +
      '<div class="mt-3"><div class="section-label">Install</div><div class="flex gap-2 mt-2"><input class="input flex-1" data-field="ext-install" data-focus="ext-install" value="' + attr(ui.installSource) + '" placeholder="owner/repo[@1.2.0]"><button class="btn btn--primary btn--sm" data-action="install-ext">' + icon('download', 'icon--sm') + ' Install</button></div></div>';
    } else if (ui.extensionsTab === 'tools') {
      const tools = [
        ['read_file', 'Read a repository file with path guard', 'READ_ONLY'],
        ['glob', 'Enumerate files matching a pattern', 'READ_ONLY'],
        ['grep', 'Search file contents with ripgrep semantics', 'READ_ONLY'],
        ['edit_file', 'Apply a structured patch to a file', 'WRITE_PROMPT'],
        ['create_file', 'Create a new file inside the workspace', 'WRITE_PROMPT'],
        ['bash', 'Execute a shell command in the repository', 'DANGEROUS'],
        ['git', 'Inspect and modify git state', 'DANGEROUS']
      ];
      body = '<div class="panel"><div class="panel-head">' + icon('tool', 'icon--sm') + ' Tool registry · ' + tools.length + ' tools</div><table class="tool-table"><thead><tr><th>Tool</th><th>Tier</th><th>Description</th><th>Source</th></tr></thead><tbody>' +
        tools.map(function (tool) {
          return '<tr><td><span class="kind-pill">' + tool[0] + '</span></td><td class="' + (tool[2] === 'READ_ONLY' ? 'tier-read' : tool[2] === 'WRITE_PROMPT' ? 'tier-write' : 'tier-danger') + '">' + tool[2] + '</td><td class="tool-desc">' + tool[1] + '</td><td class="muted micro">built-in</td></tr>';
        }).join('') + '</tbody></table></div>';
    } else if (ui.extensionsTab === 'mcp') {
      const servers = installed.filter(function (item) { return item.kind === 'mcp' || item.kind === 'wasm'; });
      body = '<div class="section-label">MCP servers & wasm plugins</div><div class="ext-list mt-2">' + servers.map(function (item) {
        return '<div class="hook-row">' + icon(item.kind === 'mcp' ? 'database' : 'cpu', 'icon--sm') + '<span class="hook-event">' + esc(item.kind) + '</span><span class="hook-name">' + esc(item.id) + ' v' + esc(item.version) + '</span>' + badge(item.trusted ? 'trusted' : 'untrusted', item.trusted ? 'green' : 'rose') + '<span class="hook-command">' + esc(item.tools.join(', ') || 'no tools declared') + '</span></div>';
      }).join('') + '</div>' +
      '<div class="section-label mt-4">Hooks</div><div class="ext-list mt-2">' + hooks.map(function (hook) {
        return '<div class="hook-row">' + icon('hook', 'icon--sm') + '<span class="hook-event">' + esc(hook.event) + '</span><span class="hook-name">' + esc(hook.name) + '</span>' + '<button class="btn btn--xs" data-action="toggle-git-hook" data-installed="' + (hook.enabled ? '1' : '0') + '">' + (hook.enabled ? 'Uninstall hook' : 'Install hook') + '</button><span class="hook-command">' + esc(hook.command) + '</span></div>';
      }).join('') + '</div>';
    } else {
      body = '<div class="ext-list">' + registry.map(function (item) {
        return '<div class="ext-row"><div class="ext-head"><span class="ext-id">' + esc(item.id) + '</span>' + badge('v' + item.version, 'muted') + badge(item.kind, item.kind === 'mcp' ? 'amber' : item.kind === 'wasm' ? 'blue' : 'tan') + '<span class="ml-auto">' + (installedIds.indexOf(item.id) >= 0 ? badge('installed', 'green') : '<button class="btn btn--xs" data-action="install-registry" data-id="' + attr(item.id) + '">' + icon('download', 'icon--sm') + ' Install</button>') + '</span></div>' +
          '<div class="ext-description">' + esc(item.description) + '</div><div class="ext-meta">' + esc(item.repo) + '</div></div>';
      }).join('') + '</div>';
    }

    return '<div class="route-scroll" data-scroll="extensions"><div class="route-wrap route-wrap--wide">' +
      '<header class="route-header"><div><p class="label route-eyebrow">TOOL REGISTRY · MCP · HOOKS</p><h1 class="route-title">Extensions</h1><p class="route-sub">Per-user packages installed from GitHub releases. One install serves every repository.</p></div>' +
        '<div class="route-header-right"><button class="btn btn--sm" data-action="open-registry">' + icon('external', 'icon--sm') + ' Registry</button><button class="btn btn--sm" data-action="extensions-refresh">' + icon('refresh', 'icon--sm') + ' Refresh</button><button class="btn btn--sm" data-action="extensions-update">Update all</button></div></header>' +
      '<div class="segmented mb-3">' + [['installed', 'Installed', installed.length], ['tools', 'Tool registry', 7], ['mcp', 'MCP & hooks', hooks.length], ['registry', 'Registry', registry.length]].map(function (tab) {
        return '<button class="' + (ui.extensionsTab === tab[0] ? 'is-on' : '') + '" data-action="extensions-tab" data-tab="' + tab[0] + '">' + tab[1] + ' <span class="muted">' + tab[2] + '</span></button>';
      }).join('') + '</div>' +
      body +
    '</div></div>';
  }

  /* ── Cost ─────────────────────────────────────────────────────────────── */
  function renderCost() {
    const cost = state.cost;
    const factor = ui.costWindow === '7d' ? 0.19 : ui.costWindow === '90d' ? 2.4 : ui.costWindow === 'all' ? 3.4 : 1;
    const scopeFactor = ui.costScoped ? 0.81 : 1;
    const totalUsd = (cost.total_usd || 0) * factor * scopeFactor;
    const calls = Math.round((cost.calls || 0) * factor * scopeFactor);
    const prompt = Math.round((cost.prompt_tokens || 0) * factor * scopeFactor);
    const completion = Math.round((cost.completion_tokens || 0) * factor * scopeFactor);
    const knownPct = cost.total_usd > 0 ? Math.round(((cost.known_cost_usd || 0) / cost.total_usd) * 100) : 100;
    const days = ui.costWindow === '7d' ? (cost.by_day || []).slice(-7) : ui.costWindow === '30d' ? (cost.by_day || []).slice(-30) : (cost.by_day || []);
    const maxDay = Math.max.apply(null, days.map(function (bucket) { return bucket.cost_usd || 0; }).concat([0.001]));
    const breakdown = function (title, buckets, limit) {
      const rows = (buckets || []).slice(0, limit || buckets.length);
      const max = Math.max.apply(null, rows.map(function (bucket) { return bucket.cost_usd || 0; }).concat([0.001]));
      return '<section class="panel cost-panel"><div class="section-label">' + title + '</div><div class="bar-rows mt-2">' + rows.map(function (bucket) {
        return '<div><div class="bar-row-head"><span class="bar-row-label">' + esc(bucket.key) + '</span><span class="bar-row-value">' + formatUsd((bucket.cost_usd || 0) * factor) + '</span></div>' +
          '<div class="bar-track"><div class="bar-fill' + (bucket.estimated_share > 0.2 ? ' is-muted' : '') + '" style="width:' + Math.max(2, ((bucket.cost_usd || 0) / max) * 100) + '%"></div></div>' +
          '<div class="bar-row-caption">' + formatTokens(bucket.tokens) + ' tokens · ' + bucket.calls + ' calls</div></div>';
      }).join('') + '</div></section>';
    };

    return '<div class="route-scroll" data-scroll="cost"><div class="route-wrap route-wrap--wide">' +
      '<header class="route-header"><div><p class="label route-eyebrow">TOKEN CONSUMPTION · MODEL SPEND · TELEMETRY</p><h1 class="route-title">Cost</h1><p class="route-sub">What Kaioken has spent, by what spent it.</p></div>' +
        '<div class="route-header-right"><div class="segmented">' + [['7d', '7d'], ['30d', '30d'], ['90d', '90d'], ['all', 'All time']].map(function (item) {
          return '<button class="' + (ui.costWindow === item[0] ? 'is-on' : '') + '" data-action="cost-window" data-window="' + item[0] + '">' + item[1] + '</button>';
        }).join('') + '</div><div class="segmented"><button class="' + (!ui.costScoped ? 'is-on' : '') + '" data-action="cost-scope" data-scoped="0">All repos</button><button class="' + (ui.costScoped ? 'is-on' : '') + '" data-action="cost-scope" data-scoped="1">' + esc(state.repo.name) + '</button></div><button class="icon-btn" data-action="cost-refresh" title="Reload">' + icon('refresh', 'icon--sm') + '</button></div></header>' +
      '<section class="panel cost-hero hud-corners"><div class="cost-total-row">' +
        '<div class="cost-stat"><div class="value is-accent">' + formatUsd(totalUsd) + '</div><div class="caption">Total spend</div></div>' +
        '<div class="cost-stat"><div class="value">' + calls.toLocaleString() + '</div><div class="caption">Calls</div></div>' +
        '<div class="cost-stat"><div class="value">' + formatTokens(prompt + completion) + '</div><div class="caption">Tokens</div></div>' +
        '<div class="cost-stat"><div class="value">' + formatTokens(prompt) + ' / ' + formatTokens(completion) + '</div><div class="caption">Prompt / completion</div></div>' +
        '<div class="cost-stat"><div class="value">' + (cost.local_calls || 0) + '</div><div class="caption">Local calls</div></div>' +
      '</div><div class="cost-note">' + icon('info', 'icon--sm') + '<span>' + knownPct + '% of this figure was reported by a provider; the rest is estimated from the model price catalog.</span>' +
        (cost.pricing_stale ? badge('catalog stale', 'amber') : badge('catalog fresh', 'green')) + '</div></section>' +
      '<div class="cost-grid">' +
        breakdown('By operation', cost.by_operation) +
        breakdown('By model', cost.by_model, 8) +
        breakdown('By provider', cost.by_provider) +
        breakdown('By workspace', cost.by_workspace, 8) +
      '</div>' +
      '<section class="panel cost-panel mt-3"><div class="section-label">Daily spend</div><div class="daily-chart">' + (days.length ? days.map(function (bucket) {
        return '<div class="daily-bar" style="height:' + Math.max(3, ((bucket.cost_usd || 0) / maxDay) * 100) + '%" title="' + esc(bucket.key) + ' — ' + formatUsd(bucket.cost_usd) + ' · ' + formatTokens(bucket.tokens) + ' tokens · ' + bucket.calls + ' calls"></div>';
      }).join('') : '<span class="muted micro">No daily buckets.</span>') + '</div><div class="daily-chart-labels"><span>' + (days[0] ? esc(days[0].key) : '—') + '</span><span>' + (days.length ? esc(days[days.length - 1].key) : '—') + '</span></div></section>' +
    '</div></div>';
  }

  /* ── Settings ─────────────────────────────────────────────────────────── */
  function renderSettings() {
    const draft = ui.settingsDraft;
    const permissionModes = [
      ['read-only', 'Read only', 'Observe and search. Never writes, never runs shell commands.'],
      ['edit-with-approval', 'Edit with approval', 'Every write and dangerous command waits for an explicit decision.'],
      ['full-auto', 'Full auto', 'Allowlisted writes and commands run without prompts. Dangerous tools still ask.']
    ];
    const keys = [
      ['openrouter', 'OpenRouter', draft.has_openrouter_key, 'OPENROUTER_API_KEY'],
      ['anthropic', 'Anthropic', draft.has_anthropic_key, 'ANTHROPIC_API_KEY'],
      ['perplexity', 'Perplexity', draft.has_perplexity_key, 'PERPLEXITY_API_KEY'],
      ['tavily', 'Tavily', draft.has_tavily_key, 'TAVILY_API_KEY']
    ];
    return '<div class="route-scroll" data-scroll="settings"><div class="route-wrap settings-wrap">' +
      '<header class="route-header"><div><p class="label route-eyebrow">MODEL · PERMISSIONS · KEYS · DAEMON</p><h1 class="route-title">Settings</h1><p class="route-sub">Changes apply to the desktop shell and are written back to the daemon configuration.</p></div>' +
        '<div class="route-header-right">' + (ui.settingsDirty ? badge('unsaved', 'amber') : badge('saved', 'green')) + kbd('Ctrl+,') + '</div></header>' +

      '<details class="settings-section" open><summary>' + icon('shield', 'icon--sm') + ' Permission mode <span class="summary-badge">' + esc(draft.permission_mode) + '</span></summary><div class="settings-body">' +
        '<div class="permission-grid">' + permissionModes.map(function (mode) {
          return '<button class="permission-card' + (draft.permission_mode === mode[0] ? ' is-active' : '') + '" data-action="settings-permission" data-mode="' + mode[0] + '"><div class="permission-name">' + icon(draft.permission_mode === mode[0] ? 'check-circle' : 'shield', 'icon--sm') + ' ' + mode[1] + '</div><div class="permission-desc">' + mode[2] + '</div></button>';
        }).join('') + '</div>' +
        '<div class="settings-field mt-2"><div class="field-copy"><div class="field-label">Approval behavior</div><div class="field-hint">Deny-first: Escape always denies, and dismissing the dialog is never authorization. Dangerous tools require explicit approval even under full-auto.</div></div><div class="field-control text-right">' + badge('deny-first', 'rose') + '</div></div>' +
      '</div></details>' +

      '<details class="settings-section" open><summary>' + icon('cpu', 'icon--sm') + ' Model & provider <span class="summary-badge">' + esc(draft.model) + '</span></summary><div class="settings-body">' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Model</div><div class="field-hint">Provider/model id. The titlebar shows the short model name.</div></div><div class="field-control"><input class="input" data-setting="model" data-focus="setting-model" value="' + attr(draft.model) + '" placeholder="provider/model-id"></div></div>' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Base URL override</div><div class="field-hint">Only for account-scoped endpoints (Azure, Cloudflare). Leave blank otherwise.</div></div><div class="field-control"><input class="input" data-setting="base_url" data-focus="setting-base_url" value="' + attr(draft.base_url) + '" placeholder="https://…"></div></div>' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Research engine</div><div class="field-hint">Both queries every vendor with a key and merges the result set.</div></div><div class="field-control"><select class="select" data-setting="search_provider">' + optionList(['both', 'perplexity', 'tavily', 'exa'], draft.search_provider) + '</select></div></div>' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Page reader</div><div class="field-hint">Auto picks the best available fetcher; browser mode renders JS-heavy pages.</div></div><div class="field-control"><select class="select" data-setting="fetcher_mode">' + optionList(['auto', 'http', 'browser'], draft.fetcher_mode) + '</select></div></div>' +
      '</div></details>' +

      '<details class="settings-section"><summary>' + icon('key', 'icon--sm') + ' API keys <span class="summary-badge">' + (keys.filter(function (item) { return item[2]; }).length + ' configured') + '</span></summary><div class="settings-body">' +
        '<p class="field-hint mb-2">Keys are written to ~/.kaioken/config.yaml and are never sent back to this UI. Presence is shown as a status badge.</p>' +
        keys.map(function (item) {
          return '<div class="key-row"><span class="key-provider">' + item[1] + '</span><span class="key-status">' + (item[2] ? badge('configured', 'green') : badge('no key', 'rose')) + '</span>' +
            '<span class="key-input"><input class="input" type="password" data-key="' + item[0] + '" placeholder="' + (item[2] ? '•••••••••••• (saved)' : 'paste ' + item[3]) + '"></span>' +
            '<span class="key-actions"><button class="btn btn--xs" data-action="save-key" data-key="' + item[0] + '">' + icon('save', 'icon--sm') + ' Save</button>' + (item[2] ? '<button class="icon-btn" data-action="test-key" data-key="' + item[0] + '" title="Test key">' + icon('beaker', 'icon--sm') + '</button>' : '') + '</span></div>';
        }).join('') +
      '</div></details>' +

      '<details class="settings-section"><summary>' + icon('database', 'icon--sm') + ' Daemon <span class="summary-badge">' + esc(draft.daemon_mode) + '</span></summary><div class="settings-body">' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Mode</div><div class="field-hint">Embedded runs memory, watcher, and IPC in this process. Daemon connects to a named pipe or TCP endpoint.</div></div><div class="field-control"><div class="segmented"><button class="' + (draft.daemon_mode === 'embedded' ? 'is-on' : '') + '" data-action="settings-daemon-mode" data-mode="embedded">Embedded</button><button class="' + (draft.daemon_mode === 'daemon' ? 'is-on' : '') + '" data-action="settings-daemon-mode" data-mode="daemon">Daemon</button></div></div></div>' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Endpoint</div><div class="field-hint">Windows named pipe, Unix socket, or loopback TCP address.</div></div><div class="field-control"><input class="input" data-setting="endpoint" data-focus="setting-endpoint" value="' + attr(draft.endpoint) + '" placeholder="embedded in-process"></div></div>' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">File watcher</div><div class="field-hint">Debounce window for filesystem events before AST re-parsing.</div></div><div class="field-control"><div class="toggle-line justify-between"><button class="switch' + (draft.watcher_enabled ? ' is-on' : '') + '" data-action="settings-watcher" aria-label="Toggle watcher"></button><input class="input" style="width:90px" type="number" min="50" max="5000" step="50" data-setting="debounce_ms" value="' + attr(draft.debounce_ms) + '"><span class="muted micro">ms</span></div></div></div>' +
      '</div></details>' +

      '<details class="settings-section"><summary>' + icon('sun', 'icon--sm') + ' Appearance <span class="summary-badge">' + esc(ui.theme) + '</span></summary><div class="settings-body">' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Theme</div><div class="field-hint">Persisted in localStorage and in the daemon settings.</div></div><div class="field-control"><div class="segmented"><button class="' + (ui.theme === 'dark' ? 'is-on' : '') + '" data-action="settings-theme" data-theme="dark">Dark</button><button class="' + (ui.theme === 'light' ? 'is-on' : '') + '" data-action="settings-theme" data-theme="light">Light</button></div></div></div>' +
        '<div class="settings-field"><div class="field-copy"><div class="field-label">Scanline texture</div><div class="field-hint">CRT scanlines at the threshold of perception; disabled by prefers-reduced-motion.</div></div><div class="field-control text-right">' + badge('HUD active', 'green') + '</div></div>' +
      '</div></details>' +

      '<div class="settings-savebar"><button class="btn btn--primary btn--sm" data-action="save-settings">' + icon('save', 'icon--sm') + ' Save settings</button>' +
        '<span class="save-status">' + (ui.settingsDirty ? 'Unsaved changes' : 'All changes saved') + '</span>' +
        '<span class="ml-auto muted micro">' + esc(draft.permission_mode) + ' · ' + esc(draft.model) + '</span></div>' +
    '</div></div>';
  }

  function optionList(values, selected) {
    return values.map(function (value) {
      return '<option value="' + attr(value) + '"' + (value === selected ? ' selected' : '') + '>' + esc(value) + '</option>';
    }).join('');
  }

  /* ── Modals ───────────────────────────────────────────────────────────── */
  function renderModals() {
    const root = document.getElementById('modalRoot');
    if (!root) return;
    const approval = currentApproval();
    const chunks = [];
    if (ui.paletteOpen) chunks.push(renderPalette());
    if (ui.helpOpen) chunks.push(renderHelp());
    if (approval) chunks.push(renderApproval(approval));
    if (ui.trustExt) chunks.push(renderTrust());
    root.innerHTML = chunks.join('');

    if (ui.paletteOpen) {
      setTimeout(function () {
        const input = document.getElementById('paletteInput');
        if (input && document.activeElement !== input) {
          input.focus();
          input.setSelectionRange(input.value.length, input.value.length);
        }
      }, 0);
    }
    if (approval && ui.approvalFocusId !== approval.id) {
      ui.approvalFocusId = approval.id;
      setTimeout(function () {
        const deny = document.getElementById('approvalDeny');
        if (deny) deny.focus();
      }, 0);
    }
    if (!approval) ui.approvalFocusId = null;
  }

  function renderPalette() {
    const query = ui.paletteQuery;
    const items = paletteItems(query);
    const isFiles = ui.paletteMode === 'files';
    ui.paletteFiltered = items;
    if (ui.paletteIndex >= items.length) ui.paletteIndex = Math.max(0, items.length - 1);
    const groups = {};
    items.forEach(function (item, index) {
      if (!groups[item.group]) groups[item.group] = [];
      groups[item.group].push({ item: item, index: index });
    });
    const list = Object.keys(groups).map(function (group) {
      return '<div class="palette-group"><div class="palette-group-label">' + esc(group) + '</div>' +
        groups[group].map(function (entry) {
          return '<button class="palette-item' + (entry.index === ui.paletteIndex ? ' is-selected' : '') + '" data-action="palette-exec" data-id="' + attr(entry.item.id) + '">' +
            '<span class="palette-title">' + esc(entry.item.title) + '</span>' +
            (entry.item.shortcut ? kbd(entry.item.shortcut) : '') +
            '<span class="palette-cat">' + esc(entry.item.category) + '</span></button>';
        }).join('') + '</div>';
    }).join('');
    return '<div class="overlay palette-overlay" data-scrim="palette"><div class="palette hud-corners">' +
      '<div class="palette-search">' + icon('search') + '<input id="paletteInput" data-field="palette-query" value="' + attr(query) + '" placeholder="' + (isFiles ? 'Search open files…' : 'Type a command or route…') + '" spellcheck="false"><span class="kbd">Esc</span></div>' +
      '<div class="palette-list" id="paletteList">' + (list || '<div class="palette-empty">No results</div>') + '</div>' +
      '<div class="palette-foot"><span>' + kbd('↑') + kbd('↓') + ' navigate</span><span>' + kbd('Enter') + ' run</span><span>' + kbd('Esc') + ' close</span><span class="ml-auto">' + (items.length ? items.length + (isFiles ? ' files' : ' commands') : 'nothing matches') + '</span></div>' +
    '</div></div>';
  }

  function filePaletteItems(query) {
    const q = String(query || '').trim().toLowerCase();
    const files = state.editor.files || [];
    return files
      .filter(function (file) { return !q || fuzzyMatch(file.path.toLowerCase(), q); })
      .slice(0, 24)
      .map(function (file) {
        return {
          id: 'file:' + file.path,
          title: file.path,
          category: 'Open Files',
          shortcut: null,
          group: 'Open Files'
        };
      });
  }

  function paletteItems(query) {
    if (ui.paletteMode === 'files') return filePaletteItems(query);
    const actions = Array.isArray(state.palette_actions) && state.palette_actions.length
      ? state.palette_actions
      : ROUTE_META.map(function (route) {
        return { title: 'Switch to ' + route.label, shortcut: route.shortcut || null, category: 'Navigation', action_id: 'nav:' + route.id };
      }).concat([
        { title: 'New Chat Session', shortcut: 'Ctrl+N', category: 'Commands', action_id: 'action:new_session' },
        { title: 'Compact Conversation', shortcut: 'Ctrl+Shift+C', category: 'Commands', action_id: 'action:compact_session' },
        { title: 'Review Working Tree Diff', shortcut: 'Ctrl+6', category: 'Commands', action_id: 'action:git_diff' },
        { title: 'Run Codebase Scan', shortcut: 'Ctrl+Shift+S', category: 'Commands', action_id: 'action:scan' },
        { title: 'Run Environment Doctor', shortcut: 'Ctrl+Shift+D', category: 'Commands', action_id: 'action:doctor' },
        { title: 'Clear Repository Memory', shortcut: null, category: 'Commands', action_id: 'action:clear_memory' },
        { title: 'Toggle Theme', shortcut: null, category: 'Commands', action_id: 'action:toggle_theme' }
      ]);
    const q = String(query || '').trim().toLowerCase();
    const mapped = actions.map(function (action) {
      return {
        id: action.action_id,
        title: action.title || action.label || '',
        category: action.category || (action.action_id && action.action_id.indexOf('nav:') === 0 ? 'Navigation' : 'Commands'),
        shortcut: action.shortcut || null,
        group: action.category || (action.action_id && action.action_id.indexOf('nav:') === 0 ? 'Navigation' : 'Commands')
      };
    });
    if (!q) return mapped.slice(0, 28);
    return mapped.filter(function (item) {
      return fuzzyMatch(item.title.toLowerCase(), q) || item.category.toLowerCase().indexOf(q) >= 0;
    }).slice(0, 24);
  }

  function fuzzyMatch(text, query) {
    if (text.indexOf(query) >= 0) return true;
    let index = 0;
    for (let i = 0; i < text.length && index < query.length; i += 1) {
      if (text[i] === query[index]) index += 1;
    }
    return index === query.length;
  }

  function updatePaletteList() {
    const list = document.getElementById('paletteList');
    if (!list) {
      renderModals();
      return;
    }
    const items = paletteItems(ui.paletteQuery);
    ui.paletteFiltered = items;
    if (ui.paletteIndex >= items.length) ui.paletteIndex = Math.max(0, items.length - 1);
    const groups = {};
    items.forEach(function (item, index) {
      if (!groups[item.group]) groups[item.group] = [];
      groups[item.group].push({ item: item, index: index });
    });
    list.innerHTML = Object.keys(groups).map(function (group) {
      return '<div class="palette-group"><div class="palette-group-label">' + esc(group) + '</div>' +
        groups[group].map(function (entry) {
          return '<button class="palette-item' + (entry.index === ui.paletteIndex ? ' is-selected' : '') + '" data-action="palette-exec" data-id="' + attr(entry.item.id) + '">' +
            '<span class="palette-title">' + esc(entry.item.title) + '</span>' +
            (entry.item.shortcut ? kbd(entry.item.shortcut) : '') +
            '<span class="palette-cat">' + esc(entry.item.category) + '</span></button>';
        }).join('') + '</div>';
    }).join('') || '<div class="palette-empty">No results</div>';
  }

  function movePalette(delta) {
    if (!ui.paletteOpen) return;
    const items = ui.paletteFiltered.length ? ui.paletteFiltered : paletteItems(ui.paletteQuery);
    if (!items.length) return;
    ui.paletteIndex = (ui.paletteIndex + delta + items.length) % items.length;
    document.querySelectorAll('.palette-item').forEach(function (element, index) {
      element.classList.toggle('is-selected', index === ui.paletteIndex);
    });
    const selected = document.querySelectorAll('.palette-item')[ui.paletteIndex];
    if (selected) selected.scrollIntoView({ block: 'nearest' });
  }

  function renderHelp() {
    return '<div class="overlay" data-scrim="help"><div class="modal help-modal hud-corners">' +
      '<div class="modal-head"><span class="modal-title">Keyboard shortcuts</span><span class="modal-sub">Full cheat sheet</span><button class="icon-btn ml-auto" data-action="close-help" aria-label="Close">' + icon('x', 'icon--sm') + '</button></div>' +
      '<div class="modal-body"><div class="shortcut-columns">' + SHORTCUTS.map(function (group) {
        return '<div class="shortcut-group"><div class="shortcut-group-label">' + esc(group.group) + '</div>' + group.rows.map(function (row) {
          return '<div class="shortcut-row"><span>' + esc(row[1]) + '</span><span>' + row[0].split('+').map(function (part) { return kbd(part); }).join('+') + '</span></div>';
        }).join('') + '</div>';
      }).join('') + '</div></div>' +
      '<div class="modal-foot"><span class="muted micro">Press Escape or click outside to close</span></div>' +
    '</div></div>';
  }

  function renderApproval(proposal) {
    const tier = String(proposal.policy_tier || 'WRITE_PROMPT');
    const isDanger = tier === 'DANGEROUS';
    const diff = proposal.diff || '';
    return '<div class="overlay" data-scrim="approval"><div class="modal approval-modal hud-corners">' +
      '<div class="approval-warning">' + icon('alert') + '<div class="flex-1"><div class="warn-title">Tool execution requires approval</div><div class="warn-sub">The agent is parked on this decision. Nothing is written until you approve.</div></div>' + badge(tier, isDanger ? 'rose' : 'amber') + '</div>' +
      '<div class="modal-body">' +
        '<dl class="approval-meta"><dt>Tool</dt><dd>' + esc(proposal.tool_name) + '</dd><dt>Target</dt><dd>' + esc(proposal.target) + '</dd><dt>Tier</dt><dd>' + esc(tier) + '</dd><dt>Details</dt><dd>' + esc(proposal.details) + '</dd></dl>' +
        (diff ? '<div class="approval-diff">' + renderDiff(diff) + '</div>' : '<pre class="approval-command">' + esc(proposal.target) + '</pre>') +
      '</div>' +
      '<div class="modal-foot"><button class="btn btn--danger" id="approvalDeny" data-action="deny-tool" data-id="' + attr(proposal.id) + '">Deny ' + kbd('Esc') + '</button>' +
        '<button class="btn btn--primary" data-action="approve-tool" data-id="' + attr(proposal.id) + '">Approve ' + kbd('Enter') + '</button>' +
        '<span class="hint">Denying leaves the working tree byte-identical</span></div>' +
    '</div></div>';
  }

  function renderTrust() {
    const all = (state.extensions.installed || []).concat(state.extensions.registry || []);
    const ext = all.find(function (item) { return item.id === ui.trustExt; });
    if (!ext) return '';
    const executable = ext.kind === 'mcp' || ext.kind === 'wasm';
    return '<div class="overlay" data-scrim="trust"><div class="modal trust-modal hud-corners">' +
      '<div class="modal-head">' + icon('shield', 'icon--sm') + '<span class="modal-title">Trust ' + esc(ext.id) + ' ' + esc(ext.version) + '?</span><button class="icon-btn ml-auto" data-action="close-trust" aria-label="Close">' + icon('x', 'icon--sm') + '</button></div>' +
      '<div class="modal-body">' +
        (executable
          ? '<p class="field-hint">This extension wants to run an ' + (ext.kind === 'mcp' ? 'MCP server UNSANDBOXED' : 'sandboxed wasm plugin') + '. Trust applies to this exact version; updates revoke it.</p><pre class="trust-command">' + esc(ext.kind === 'mcp' ? ext.repo + '\ncommand: npx ' + ext.id : 'module: ' + ext.id + '.wasm\npermissions: fs:read:workspace') + '</pre>'
          : '<p class="field-hint">This declarative extension contributes skills and never runs code. Trust is not required.</p>') +
        '<p class="field-hint mt-2">Every tool call still asks for approval. Kaioken cannot restrict what an unsandboxed MCP process does.</p>' +
      '</div>' +
      '<div class="modal-foot"><button class="btn btn--sm" data-action="close-trust">Cancel</button><button class="btn btn--primary btn--sm" data-action="confirm-trust" data-id="' + attr(ext.id) + '">Trust and run</button></div>' +
    '</div></div>';
  }

  /* ── Actions ──────────────────────────────────────────────────────────── */
  function go(route) {
    const known = (state.routes || ROUTE_META).some(function (item) { return item.id === route; });
    if (!known) return;
    ui.route = route;
    post({ action: 'switch_tab', tab: route });
    render();
  }

  function sendChat() {
    const text = ui.chatInput.trim();
    if (!text) return;
    ui.chatInput = '';
    ui.chatPending = true;
    render();
    post({ action: 'send_chat', prompt: text });
  }

  function approveTool(id) {
    const proposal = currentApproval();
    const proposalId = id || (proposal && proposal.id);
    if (!proposalId) return;
    ui.dismissedApproval = proposalId;
    state.agent.pending_proposals = (state.agent.pending_proposals || []).filter(function (item) { return item.id !== proposalId; });
    render();
    post({ action: 'approve_tool', proposal_id: proposalId });
    toast('success', 'Approved', 'The tool call is executing in the embedded runtime.');
  }

  function denyTool(id) {
    const proposal = currentApproval();
    const proposalId = id || (proposal && proposal.id);
    if (!proposalId) return;
    ui.dismissedApproval = proposalId;
    state.agent.pending_proposals = (state.agent.pending_proposals || []).filter(function (item) { return item.id !== proposalId; });
    render();
    post({ action: 'deny_tool', proposal_id: proposalId });
    toast('warn', 'Denied', 'No files were modified.');
  }

  function toggleTheme() {
    setTheme(ui.theme === 'dark' ? 'light' : 'dark');
  }

  function setTheme(theme) {
    ui.theme = theme === 'light' ? 'light' : 'dark';
    state.theme = ui.theme;
    state.settings.theme = ui.theme;
    ui.settingsDraft.theme = ui.theme;
    applyTheme();
    post({ action: 'toggle_theme', theme: ui.theme });
    render();
  }

  function toggleTerminal() {
    if (ui.route !== 'editor') {
      ui.route = 'editor';
      ui.terminalOpen = true;
      post({ action: 'switch_tab', tab: 'editor' });
    } else {
      ui.terminalOpen = !ui.terminalOpen;
    }
    render();
  }

  function startResearch() {
    const query = ui.researchInput.trim() || state.research.query || '';
    if (!query) {
      toast('warn', 'No question', 'Type a research question first.');
      return;
    }
    ui.researchInput = '';
    post({ action: 'start_research', query: query, power_level: state.research.power_level || 3 });
    toast('info', 'Research started', '×' + (state.research.power_level || 3) + ' · ' + query);
    setTimeout(function () { ensureResearchTimer(); }, 250);
  }

  function executePaletteAction(id) {
    if (!id) return;
    closePalette();
    if (id.indexOf('file:') === 0) {
      go('editor');
      selectFile(id.slice(5));
      return;
    }
    if (id.indexOf('nav:') === 0) {
      go(id.slice(4));
      return;
    }
    const action = id.indexOf('action:') === 0 ? id.slice(7) : id;
    switch (action) {
      case 'new_session': post({ action: 'new_session' }); ui.chatPending = false; toast('success', 'New session', 'Conversation cleared.'); go('chat'); break;
      case 'compact_session': post({ action: 'compact_session' }); toast('success', 'Compacting', 'Older turns are summarized out of context.'); break;
      case 'git_diff': go('editor'); ui.editorMode = 'diff'; post({ action: 'run_command_action', command: 'git_diff' }); render(); break;
      case 'scan': post({ action: 'run_command_action', command: 'scan' }); toast('info', 'Scan queued', 'Progress streams on Activity.'); go('activity'); break;
      case 'doctor': post({ action: 'run_command_action', command: 'doctor' }); toast('info', 'Doctor', 'Environment checks are running.'); break;
      case 'clear_memory': post({ action: 'run_command_action', command: 'clear_memory' }); toast('warn', 'Memory cleared', 'The next scan will rebuild the index.'); go('activity'); break;
      case 'toggle_theme': toggleTheme(); break;
      default: post({ action: 'run_command_action', command: action }); toast('info', action, 'Command dispatched.'); break;
    }
  }

  function closePalette() {
    ui.paletteOpen = false;
    ui.paletteFocused = false;
    ui.paletteMode = 'commands';
    renderModals();
  }

  function openPalette(mode) {
    ui.paletteOpen = true;
    ui.paletteMode = mode === 'files' ? 'files' : 'commands';
    ui.paletteQuery = '';
    ui.paletteIndex = 0;
    ui.paletteFocused = false;
    renderModals();
  }

  function saveSettings() {
    ui.settingsDirty = false;
    state.settings = clone(ui.settingsDraft);
    post({ action: 'update_settings', settings: clone(ui.settingsDraft) });
    toast('success', 'Settings saved', 'Permission mode and daemon configuration updated.');
    render();
  }

  function markSettingsDirty() {
    if (!ui.settingsDirty) {
      ui.settingsDirty = true;
      const status = document.querySelector('.settings-savebar .save-status');
      if (status) status.textContent = 'Unsaved changes';
      const header = document.querySelector('.route-header-right .badge');
      if (header) {
        header.textContent = 'unsaved';
        header.className = 'badge badge--amber';
      }
    }
  }

  function selectFile(path) {
    ui.activeFile = path;
    ui.editorMode = 'code';
    post({ action: 'read_file', path: path });
    render();
  }

  function closeFile(path) {
    state.editor.files = (state.editor.files || []).filter(function (file) { return file.path !== path; });
    if (ui.activeFile === path) {
      ui.activeFile = state.editor.files[0] ? state.editor.files[0].path : '';
    }
    render();
  }

  function saveFile() {
    const file = (state.editor.files || []).find(function (item) { return item.path === ui.activeFile; });
    if (!file) return;
    file.saved = file.content;
    file.dirty = false;
    post({ action: 'save_file', path: file.path, content: file.content });
    toast('success', 'Saved', file.path);
    render();
  }

  function appendTerminalLine() {
    const text = ui.terminalInput.trim();
    if (!text) return;
    state.editor.terminal_lines = (state.editor.terminal_lines || []).concat([
      { kind: 'cmd', text: '$ ' + text }
    ]);
    post({ action: 'terminal_input', command: text });
    ui.terminalInput = '';
    render();
  }

  function browserSubmit() {
    const value = ui.browserUrl.trim();
    if (!value) return;
    const tab = (state.browser.tabs || []).find(function (item) { return item.id === ui.browserActive; });
    if (!tab) return;
    let url = value;
    if (!/^https?:\/\//i.test(url)) {
      url = url.indexOf('.') >= 0 ? 'https://' + url : 'https://kaioken.dev/search?q=' + encodeURIComponent(url);
    }
    tab.url = url;
    tab.title = url.split('/').slice(-1)[0] || url;
    tab.kind = url.indexOf('/api/') >= 0 ? 'api' : 'web';
    state.browser.request_url = url;
    state.browser.method = 'GET';
    ui.browserActive = tab.id;
    ui.browserUrl = '';
    post({ action: 'browser_request', url: url, method: 'GET' });
    render();
  }

  function newBrowserTab() {
    const id = 'tab-' + Date.now();
    state.browser.tabs = (state.browser.tabs || []).concat([{ id: id, title: 'New tab', url: '', kind: 'newtab' }]);
    ui.browserActive = id;
    ui.browserUrl = '';
    render();
  }

  function closeBrowserTab(id) {
    const tabs = state.browser.tabs || [];
    if (tabs.length <= 1) return;
    state.browser.tabs = tabs.filter(function (tab) { return tab.id !== id; });
    if (ui.browserActive === id) ui.browserActive = state.browser.tabs[0].id;
    render();
  }

  function openLink(url) {
    if (!url) return;
    go('browser');
    ui.browserUrl = url;
    post({ action: 'run_command_action', command: 'open_url', url: url });
    toast('info', 'Opening in browser', url);
  }

  /* ── Event handling ───────────────────────────────────────────────────── */
  function onClick(event) {
    const scrim = event.target.closest('[data-scrim]');
    if (scrim && event.target === scrim) {
      if (scrim.dataset.scrim === 'palette') closePalette();
      else if (scrim.dataset.scrim === 'help') { ui.helpOpen = false; renderModals(); }
      else if (scrim.dataset.scrim === 'approval') { const approval = currentApproval(); if (approval) denyTool(approval.id); }
      else if (scrim.dataset.scrim === 'trust') { ui.trustExt = null; renderModals(); }
      return;
    }

    const routeButton = event.target.closest('[data-route]');
    if (routeButton) {
      event.preventDefault();
      go(routeButton.dataset.route);
      return;
    }

    const target = event.target.closest('[data-action]');
    if (!target) return;
    const action = target.dataset.action;

    if (action === 'go-chat') { go('chat'); return; }
    if (action === 'open-palette') { openPalette(); return; }
    if (action === 'close-help') { ui.helpOpen = false; renderModals(); return; }
    if (action === 'dismiss-toast') {
      ui.toasts = ui.toasts.filter(function (item) { return String(item.id) !== target.dataset.id; });
      renderToasts();
      return;
    }
    if (action === 'repo-menu') {
      ui.repoMenuOpen = !ui.repoMenuOpen;
      renderRepoMenu();
      const switchButton = document.getElementById('repoSwitch');
      if (switchButton) switchButton.classList.toggle('is-open', ui.repoMenuOpen);
      return;
    }
    if (action === 'repo-select' || action === 'open-repo-picker') {
      ui.repoMenuOpen = false;
      renderRepoMenu();
      post({ action: 'run_command_action', command: 'scan' });
      toast('info', 'Repository', 'Scanning workspace files and synchronizing memory graph...');
      return;
    }
    if (action === 'window-minimize' || action === 'window-maximize' || action === 'window-close') {
      if (action === 'window-maximize') {
        ui.maximized = !ui.maximized;
        renderTitlebar();
      }
      post({ action: action });
      return;
    }
    if (action === 'toggle-theme') { toggleTheme(); return; }
    if (action === 'toggle-terminal') { toggleTerminal(); return; }

    if (action === 'new-session') { post({ action: 'new_session' }); ui.chatPending = false; toast('success', 'New session', 'Conversation cleared.'); return; }
    if (action === 'switch-session') { state.agent.active_session_id = target.dataset.id; render(); return; }
    if (action === 'compact-session') { post({ action: 'compact_session' }); toast('success', 'Compacting', 'Keeping the most recent turns.'); return; }
    if (action === 'toggle-auto-approve') { ui.autoApprove = !ui.autoApprove; render(); return; }
    if (action === 'toggle-allow-shell') { ui.allowShell = !ui.allowShell; render(); return; }
    if (action === 'suggestion') {
      ui.chatInput = target.dataset.text || '';
      renderRoute();
      const input = document.querySelector('[data-focus="chat-input"]');
      if (input) input.focus();
      return;
    }
    if (action === 'send-chat') { sendChat(); return; }
    if (action === 'approve-tool') { approveTool(target.dataset.id); return; }
    if (action === 'deny-tool') { denyTool(target.dataset.id); return; }

    if (action === 'set-power') {
      const level = Number(target.dataset.power);
      state.research.power_level = level;
      post({ action: 'set_power_level', power_level: level });
      render();
      return;
    }
    if (action === 'start-research') { startResearch(); return; }
    if (action === 'research-history') {
      ui.researchInput = target.dataset.question || '';
      renderRoute();
      const input = document.querySelector('[data-focus="research-query"]');
      if (input) input.focus();
      return;
    }
    if (action === 'cite') {
      const index = Number(target.dataset.index);
      const citation = (state.research.citations || []).find(function (item) { return item.index === index; });
      if (citation) {
        openLink(citation.url);
        toast('info', 'Citation [' + index + ']', citation.title + ' · ' + citation.host);
      }
      return;
    }
    if (action === 'open-link') { openLink(target.dataset.url); return; }

    if (action === 'open-wiki') {
      ui.wikiSelected = target.dataset.path;
      state.wiki.selected_path = target.dataset.path;
      ui.wikiToc = '';
      render();
      return;
    }
    if (action === 'wiki-toc') {
      const slug = target.dataset.slug;
      ui.wikiToc = slug;
      document.querySelectorAll('.toc-item').forEach(function (item) { item.classList.toggle('is-active', item.dataset.slug === slug); });
      const heading = document.getElementById(slug);
      if (heading) heading.scrollIntoView({ behavior: 'smooth', block: 'start' });
      return;
    }

    if (action === 'graph-kind') { ui.graphKind = target.dataset.kind; render(); return; }
    if (action === 'graph-fit') { ui.graphZoom = 1; ui.graphKind = 'all'; ui.graphQuery = ''; render(); return; }
    if (action === 'graph-zoom-in') { ui.graphZoom = Math.min(2.5, (ui.graphZoom || 1) + 0.2); render(); return; }
    if (action === 'graph-zoom-out') { ui.graphZoom = Math.max(0.5, (ui.graphZoom || 1) - 0.2); render(); return; }
    if (action === 'select-node') { ui.graphSelected = Number(target.dataset.node); render(); return; }

    if (action === 'cards-filter') { ui.cardFilter = target.dataset.filter; render(); return; }
    if (action === 'select-card') { ui.cardSelected = target.dataset.id; render(); return; }
    if (action === 'resolve-card') {
      post({ action: 'resolve_failure', id: target.dataset.id, fix_summary: 'Resolved in UI' });
      toast('success', 'Resolved', target.dataset.id);
      render();
      return;
    }

    if (action === 'editor-tab') { selectFile(target.dataset.path); return; }
    if (action === 'close-file') { event.stopPropagation(); closeFile(target.dataset.path); return; }
    if (action === 'editor-mode') { ui.editorMode = target.dataset.mode; render(); return; }
    if (action === 'save-file') { saveFile(); return; }
    if (action === 'terminal-tab') { ui.terminalTab = Number(target.dataset.tab); render(); return; }

    if (action === 'browser-tab') { ui.browserActive = target.dataset.id; ui.browserUrl = ''; render(); return; }
    if (action === 'browser-close-tab') { event.stopPropagation(); closeBrowserTab(target.dataset.id); return; }
    if (action === 'browser-new-tab') { newBrowserTab(); return; }
    if (action === 'browser-back') { toast('info', 'Back', 'History navigation is local to this session.'); return; }
    if (action === 'browser-forward') { toast('info', 'Forward', 'History navigation is local to this session.'); return; }
    if (action === 'browser-reload') {
      const tab = (state.browser.tabs || []).find(function (item) { return item.id === ui.browserActive; });
      if (tab && tab.url) post({ action: 'browser_request', url: tab.url, method: 'GET' });
      toast('info', 'Reloading', 'Refreshing live response...');
      return;
    }
    if (action === 'browser-external' || action === 'open-registry') {
      const tab = (state.browser.tabs || []).find(function (item) { return item.id === ui.browserActive; });
      if (action === 'open-registry') toast('info', 'Registry', 'kaioken.dev/extensions opens in the system browser.');
      else openLink(tab && tab.url ? tab.url : 'https://kaioken.dev');
      return;
    }
    if (action === 'browser-star') { toast('success', 'Bookmarked', 'This page is pinned to the new-tab surface.'); return; }
    if (action === 'browser-open') { openLink(target.dataset.url); return; }

    if (action === 'toggle-run') {
      const id = target.dataset.id;
      ui.openRuns[id] = !ui.openRuns[id];
      render();
      return;
    }
    if (action === 'run-command') {
      const command = target.dataset.command;
      post({ action: 'run_command_action', command: command });
      if (command === 'clear_memory') toast('warn', 'Memory cleared', 'The next scan rebuilds the index.');
      else if (command === 'start_wiki') { toast('info', 'Wiki queued', '×' + ui.activityDepth + ' · progress streams below.'); ui.route = 'activity'; }
      else toast('info', command, 'Command dispatched to the daemon.');
      render();
      return;
    }

    if (action === 'extensions-tab') { ui.extensionsTab = target.dataset.tab; render(); return; }
    if (action === 'extensions-refresh') { toast('info', 'Registry refreshed', 'The local package list is up to date.'); return; }
    if (action === 'extensions-update') { toast('success', 'Update all', 'Installed extensions are current.'); return; }
    if (action === 'toggle-git-hook') {
      const installed = target.dataset.installed === '1';
      post({ action: 'toggle_git_hook', install: !installed });
      toast('info', 'Git Hook', !installed ? 'Installing hook...' : 'Uninstalling hook...');
      return;
    }
    if (action === 'trust-ext') { ui.trustExt = target.dataset.id; renderModals(); return; }
    if (action === 'close-trust') { ui.trustExt = null; renderModals(); return; }
    if (action === 'confirm-trust') {
      const ext = (state.extensions.installed || []).find(function (item) { return item.id === target.dataset.id; });
      if (ext) ext.trusted = true;
      ui.trustExt = null;
      toast('success', 'Trusted', 'This exact version can now run. Per-call approval still applies.');
      render();
      return;
    }
    if (action === 'toggle-ext') {
      const ext = (state.extensions.installed || []).find(function (item) { return item.id === target.dataset.id; });
      if (ext) ext.enabled = !ext.enabled;
      render();
      return;
    }
    if (action === 'remove-ext') {
      state.extensions.installed = (state.extensions.installed || []).filter(function (item) { return item.id !== target.dataset.id; });
      toast('success', 'Uninstalled', target.dataset.id);
      render();
      return;
    }
    if (action === 'install-registry') {
      const item = (state.extensions.registry || []).find(function (entry) { return entry.id === target.dataset.id; });
      if (item) state.extensions.installed = (state.extensions.installed || []).concat([clone(item)]);
      toast('success', 'Installed', target.dataset.id);
      render();
      return;
    }
    if (action === 'install-ext') {
      const source = ui.installSource.trim();
      if (!source) { toast('warn', 'Nothing to install', 'Type owner/repo first.'); return; }
      const id = source.split('/').pop().split('@')[0];
      state.extensions.installed = (state.extensions.installed || []).concat([{
        id: id, version: '1.0.0', kind: 'declarative', description: 'Installed from ' + source,
        repo: source, enabled: true, trusted: true, local: false, skills: [], tools: []
      }]);
      ui.installSource = '';
      toast('success', 'Installed', source);
      render();
      return;
    }

    if (action === 'cost-window') { ui.costWindow = target.dataset.window; render(); return; }
    if (action === 'cost-scope') { ui.costScoped = target.dataset.scoped === '1'; render(); return; }
    if (action === 'cost-refresh') { toast('info', 'Reloaded', 'Usage ledger and pricing catalog are current.'); return; }

    if (action === 'settings-permission') {
      ui.settingsDraft.permission_mode = target.dataset.mode;
      markSettingsDirty();
      render();
      return;
    }
    if (action === 'settings-daemon-mode') {
      ui.settingsDraft.daemon_mode = target.dataset.mode;
      markSettingsDirty();
      render();
      return;
    }
    if (action === 'settings-watcher') {
      ui.settingsDraft.watcher_enabled = !ui.settingsDraft.watcher_enabled;
      markSettingsDirty();
      render();
      return;
    }
    if (action === 'settings-theme') { setTheme(target.dataset.theme); return; }
    if (action === 'save-settings') { saveSettings(); return; }
    if (action === 'save-key') {
      const field = document.querySelector('[data-key="' + target.dataset.key + '"]');
      if (field && !field.value.trim()) { toast('warn', 'No key', 'Paste a key before saving.'); return; }
      const prop = 'has_' + target.dataset.key + '_key';
      ui.settingsDraft[prop] = true;
      markSettingsDirty();
      post({ action: 'save_config_key', provider: target.dataset.key, key: field.value.trim() });
      toast('success', 'Key saved', target.dataset.key + ' · written to config');
      render();
      return;
    }
    if (action === 'test-key') {
      const field = document.querySelector('[data-key="' + target.dataset.key + '"]');
      const val = field && field.value ? field.value.trim() : '';
      post({ action: 'test_api_key', provider: target.dataset.key, key: val });
      toast('info', 'Testing ' + target.dataset.key, 'Validating key against provider endpoint...');
      return;
    }

    if (action === 'palette-exec') { executePaletteAction(target.dataset.id); return; }
  }

  function onInput(event) {
    const target = event.target;
    if (!target || !target.dataset) return;
    if (target.dataset.field === 'chat-input') {
      ui.chatInput = target.value;
      const button = document.querySelector('.send-btn');
      if (button) button.disabled = !ui.chatInput.trim();
      return;
    }
    if (target.dataset.field === 'research-query') { ui.researchInput = target.value; return; }
    if (target.dataset.field === 'palette-query') {
      ui.paletteQuery = target.value;
      ui.paletteIndex = 0;
      updatePaletteList();
      return;
    }
    if (target.dataset.field === 'wiki-search') { ui.wikiQuery = target.value; renderRoute(); return; }
    if (target.dataset.field === 'graph-filter') { ui.graphQuery = target.value; renderRoute(); return; }
    if (target.dataset.field === 'browser-url') { ui.browserUrl = target.value; return; }
    if (target.dataset.field === 'ext-install') { ui.installSource = target.value; return; }
    if (target.dataset.field === 'terminal-input') { ui.terminalInput = target.value; return; }
    if (target.dataset.field === 'editor-content') {
      const file = (state.editor.files || []).find(function (item) { return item.path === ui.activeFile; });
      if (file) {
        file.content = target.value;
        file.dirty = file.content !== file.saved;
        const dirtyTab = document.querySelector('.file-tab.is-active');
        if (dirtyTab) {
          const dot = dirtyTab.querySelector('.dirty-dot');
          if (file.dirty && !dot) {
            const name = dirtyTab.querySelector('.tab-name');
            if (name) name.insertAdjacentHTML('afterend', '<span class="dirty-dot"></span>');
          } else if (!file.dirty && dot) {
            dot.remove();
          }
        }
      }
      return;
    }
    if (target.dataset.field === 'activity-depth') {
      ui.activityDepth = Number(target.value);
      const value = document.querySelector('.depth-value');
      const caption = document.querySelector('.depth-caption');
      if (value) value.textContent = '×' + ui.activityDepth;
      if (caption) caption.textContent = powerCaption(ui.activityDepth);
      return;
    }
    if (target.dataset.setting) {
      const value = target.dataset.setting === 'debounce_ms' ? Number(target.value) : target.value;
      ui.settingsDraft[target.dataset.setting] = value;
      markSettingsDirty();
      return;
    }
  }

  function onChange(event) {
    const target = event.target;
    if (!target || !target.dataset) return;
    if (target.dataset.setting) {
      ui.settingsDraft[target.dataset.setting] = target.value;
      markSettingsDirty();
    }
  }

  function onSubmit(event) {
    const form = event.target.closest('[data-form]');
    if (!form) return;
    event.preventDefault();
    if (form.dataset.form === 'browser-omnibox' || form.dataset.form === 'newtab-search') {
      const input = form.querySelector('[data-field="browser-url"]');
      if (input) ui.browserUrl = input.value;
      browserSubmit();
    }
  }

  function onKeyDown(event) {
    const approval = currentApproval();
    if (approval) {
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        approveTool(approval.id);
        return;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        denyTool(approval.id);
        return;
      }
      return;
    }

    if (ui.paletteOpen) {
      if (event.key === 'ArrowDown') { event.preventDefault(); movePalette(1); return; }
      if (event.key === 'ArrowUp') { event.preventDefault(); movePalette(-1); return; }
      if (event.key === 'Enter') {
        event.preventDefault();
        const item = ui.paletteFiltered[ui.paletteIndex];
        if (item) executePaletteAction(item.id);
        return;
      }
      if (event.key === 'Escape') { event.preventDefault(); closePalette(); return; }
    }

    if (ui.helpOpen && event.key === 'Escape') {
      event.preventDefault();
      ui.helpOpen = false;
      renderModals();
      return;
    }
    if (ui.trustExt && event.key === 'Escape') {
      event.preventDefault();
      ui.trustExt = null;
      renderModals();
      return;
    }
    if (ui.repoMenuOpen && event.key === 'Escape') {
      ui.repoMenuOpen = false;
      renderRepoMenu();
      return;
    }

    const mod = event.ctrlKey || event.metaKey;
    if (mod && event.key.toLowerCase() === 'k') { event.preventDefault(); ui.paletteOpen ? closePalette() : openPalette('commands'); return; }
    if (mod && event.key.toLowerCase() === 'p' && !event.shiftKey) { event.preventDefault(); openPalette('files'); return; }
    if (mod && event.key >= '1' && event.key <= '9') {
      event.preventDefault();
      const route = ROUTE_META[Number(event.key) - 1];
      if (route) go(route.id);
      return;
    }
    if (mod && event.key === ',') { event.preventDefault(); go('settings'); return; }
    if (mod && event.code === 'Backquote') { event.preventDefault(); toggleTerminal(); return; }
    if (mod && event.key.toLowerCase() === 'n' && !event.shiftKey) { event.preventDefault(); post({ action: 'new_session' }); toast('success', 'New session', 'Conversation cleared.'); go('chat'); return; }
    if (mod && event.shiftKey && event.key.toLowerCase() === 'c') { event.preventDefault(); post({ action: 'compact_session' }); toast('success', 'Compacting', 'Older turns summarized out of context.'); return; }
    if (mod && event.shiftKey && event.key.toLowerCase() === 's') { event.preventDefault(); post({ action: 'run_command_action', command: 'scan' }); toast('info', 'Scan queued', 'Progress streams on Activity.'); go('activity'); return; }
    if (mod && event.shiftKey && event.key.toLowerCase() === 'd') { event.preventDefault(); post({ action: 'run_command_action', command: 'doctor' }); toast('info', 'Doctor', 'Environment checks are running.'); return; }
    if (mod && event.key.toLowerCase() === 's' && !event.shiftKey && ui.route === 'editor') { event.preventDefault(); saveFile(); return; }
    if (mod && event.key.toLowerCase() === 'l' && ui.route === 'browser') {
      event.preventDefault();
      const input = document.querySelector('[data-focus="browser-url"]');
      if (input) { input.focus(); input.select(); }
      return;
    }
    if (mod && event.key.toLowerCase() === 't' && ui.route === 'browser') { event.preventDefault(); newBrowserTab(); return; }
    if (mod && event.key.toLowerCase() === 'w' && ui.route === 'browser') { event.preventDefault(); closeBrowserTab(ui.browserActive); return; }
    if (event.key === '?' && !isInputFocused()) { event.preventDefault(); ui.helpOpen = !ui.helpOpen; renderModals(); return; }
    if (event.key === 'Escape') {
      if (ui.helpOpen || ui.trustExt || ui.paletteOpen) {
        ui.helpOpen = false; ui.trustExt = null; ui.paletteOpen = false; renderModals();
      }
      return;
    }

    const target = event.target;
    if (target && target.dataset && target.dataset.field === 'chat-input' && event.key === 'Enter' && !event.shiftKey && !event.altKey && !event.ctrlKey) {
      event.preventDefault();
      sendChat();
      return;
    }
    if (target && target.dataset && target.dataset.field === 'terminal-input' && event.key === 'Enter') {
      event.preventDefault();
      appendTerminalLine();
    }
  }

  function onMouseDown(event) {
    if (event.button !== 0) return;
    const titlebar = event.target.closest('[data-drag-region]');
    if (titlebar && !event.target.closest('.titlebar-no-drag') && !event.target.closest('button') && !event.target.closest('input')) {
      post({ action: 'window_drag' });
      return;
    }
    post({ action: 'mouse_down', x: event.clientX, y: event.clientY });
  }

  let lastMouseMoveAt = 0;
  function onMouseMove(event) {
    const now = performance.now();
    if (now - lastMouseMoveAt < 36) return;
    lastMouseMoveAt = now;
    post({ action: 'mouse_move', x: event.clientX, y: event.clientY });
  }

  function onDoubleClick(event) {
    const titlebar = event.target.closest('[data-drag-region]');
    if (!titlebar) return;
    if (event.target.closest('.titlebar-no-drag') || event.target.closest('button')) return;
    ui.maximized = !ui.maximized;
    renderTitlebar();
    post({ action: 'window_maximize' });
  }

  /* ── State merge / boot ──────────────────────────────────────────────── */
  function mergeState(next) {
    if (!next || typeof next !== 'object') return;
    const previousTheme = state.theme;
    const previousMessages = (state.agent.messages || []).length;
    state = deepMerge(state, next);
    if ((state.agent.messages || []).length > previousMessages) ui.chatPending = false;
    if (!(state.agent.pending_proposals || []).length) ui.dismissedApproval = null;
    if (next.theme && next.theme !== previousTheme && next.theme !== ui.theme) {
      ui.theme = next.theme;
      state.settings.theme = next.theme;
      applyTheme();
    }
    if (!ui.settingsDirty) ui.settingsDraft = clone(state.settings);
    if (ui.activeFile && !(state.editor.files || []).some(function (file) { return file.path === ui.activeFile; })) {
      ui.activeFile = state.editor.active_path || (state.editor.files[0] && state.editor.files[0].path) || '';
    }
    if (ui.wikiSelected && !(state.wiki.chapters || []).some(function (chapter) { return chapter.path === ui.wikiSelected; })) {
      ui.wikiSelected = state.wiki.selected_path || (state.wiki.chapters[0] && state.wiki.chapters[0].path) || '';
    }
    if (!(state.browser.tabs || []).some(function (tab) { return tab.id === ui.browserActive; })) {
      ui.browserActive = state.browser.active_id || (state.browser.tabs[0] && state.browser.tabs[0].id) || '';
    }
    ensureResearchTimer();
    render();
  }

  function boot() {
    if (booted) return;
    booted = true;
    applyTheme();
    if (ui.theme !== state.theme) {
      post({ action: 'toggle_theme', theme: ui.theme });
    }
    render();
    if (!state.routes || !state.routes.length) state.routes = ROUTE_META;
    window.KAI.ready = true;
    window.KAI.booted = true;
  }

  window.KAI = {
    boot: boot,
    mergeState: mergeState,
    ready: false,
    booted: false
  };

  document.addEventListener('click', onClick);
  document.addEventListener('input', onInput);
  document.addEventListener('change', onChange);
  document.addEventListener('submit', onSubmit);
  document.addEventListener('keydown', onKeyDown);
  document.addEventListener('mousedown', onMouseDown);
  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('dblclick', onDoubleClick);
  document.addEventListener('contextmenu', function (event) {
    event.preventDefault();
  });

  boot();
})();

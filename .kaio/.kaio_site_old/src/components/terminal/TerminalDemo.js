import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import TerminalWindow from "./TerminalWindow";
import { useReducedMotion } from "@/lib/motion";
import { cn } from "@/lib/utils";
const SCENARIOS = [
    {
        id: "verify",
        label: "Verification Gate",
        title: "kaioken — autonomous verification gate",
        status: "gate passed",
        lines: [
            { kind: "user", text: "› refactor TokenValidator to accept custom expiry", hold: 300 },
            { kind: "tool", text: "● reading symbol index: auth/token.go" },
            { kind: "toolres", text: "  ↳ AST parsed · TokenValidator signature found" },
            { kind: "del", text: "- func TokenValidator(token string) bool {" },
            { kind: "add", text: "+ func TokenValidator(token string, opts ...Option) (bool, error) {" },
            { kind: "tool", text: "● running autonomous verification gate: `go test ./...`" },
            { kind: "warn", text: "  ↳ pass 1: compilation error in session_test.go:42 (missing opts)" },
            { kind: "tool", text: "● diagnostic loop: updating caller in session_test.go..." },
            { kind: "add", text: "+   valid, err := TokenValidator(testToken, WithDefaultExpiry())" },
            { kind: "tool", text: "● re-running gate: `go test ./...`" },
            { kind: "ok", text: "✓ all 42 tests passed · 0 compiler warnings · Go gate OK", hold: 800 },
            { kind: "approve", text: "apply verified patch?  [y] yes   [n] no" },
            { kind: "ok", text: "✓ applied clean commit to main tree" },
        ],
    },
    {
        id: "worktree",
        label: "Worktree Sub-Agents",
        title: "kaioken — git worktree sandbox",
        status: "sub-agent isolated",
        lines: [
            { kind: "user", text: "› delegate migration to redis v9 in background worktree", hold: 300 },
            { kind: "tool", text: "● git worktree add .kaioken/worktrees/agent-redis-migration" },
            { kind: "toolres", text: "  ↳ isolated worktree created at HEAD (7fa90c)" },
            { kind: "tool", text: "● sub-agent running in parallel workspace..." },
            { kind: "del", text: "-   legacyClient.Get(ctx, key)" },
            { kind: "add", text: "+   redisClient.Get(ctx, key).Result()" },
            { kind: "tool", text: "● running worktree test suite: `go test ./internal/cache/...`" },
            { kind: "ok", text: "✓ 12/12 cache tests passed in isolated worktree" },
            { kind: "approve", text: "merge worktree patch back into main tree?  [y]es  [n]o" },
            { kind: "ok", text: "✓ merged 2 files · 0 conflicts · worktree pruned" },
        ],
    },
    {
        id: "wiki",
        label: "Deep Architectural Wiki",
        title: "kaioken — deep wiki engine",
        status: "knowledge verified",
        lines: [
            { kind: "user", text: "› /wiki x3 force", hold: 300 },
            { kind: "tool", text: "● scanning repository AST: 214 source files · 1,480 symbols" },
            { kind: "tool", text: "● proposing modules.yaml boundaries with architectural planner..." },
            { kind: "toolres", text: "  ↳ module [core/engine]: 5 cards generated" },
            { kind: "toolres", text: "  ↳ module [internal/api]: 5 cards generated" },
            { kind: "toolres", text: "  ↳ module [store/postgres]: 5 cards generated" },
            { kind: "tool", text: "● verifying cross-module links against symbol table..." },
            { kind: "ok", text: "✓ ×3 exhaustive coverage verified · 0 grounding failures" },
            { kind: "ok", text: "✓ written to .kaioken/wiki/ · ready for team onboarding" },
        ],
    },
    {
        id: "impact",
        label: "Blast Radius (/impact)",
        title: "kaioken — symbol blast radius",
        status: "analysis complete",
        lines: [
            { kind: "user", text: "› /impact internal/store/user.go:UpdateUser", hold: 300 },
            { kind: "tool", text: "● analyzing AST call graph across entire codebase..." },
            { kind: "toolres", text: "  ↳ direct callers (4): auth/session.go, api/v1/user.go" },
            { kind: "toolres", text: "  ↳ indirect callers (9): cli/commands.go, worker/sync.go" },
            { kind: "toolres", text: "  ↳ test suites to re-run: store_test.go, user_e2e_test.go" },
            { kind: "warn", text: "  ↳ stale skills flagged: .agent/skills/user-provisioning.md" },
            { kind: "ok", text: "✓ blast radius: 4 files, 2 test suites · 0 ungrounded claims" },
        ],
    },
];
const LINE_STYLES = {
    user: "text-kai-blue font-semibold",
    assistant: "text-foreground",
    tool: "text-kai-tan",
    toolres: "text-kai-sage",
    add: "text-kai-green",
    del: "text-kai-rose",
    approve: "text-kai-amber font-semibold",
    ok: "text-kai-green",
    warn: "text-kai-amber",
    dim: "text-kai-dim",
    blank: "",
};
export default function TerminalDemo() {
    const [activeScenario, setActiveScenario] = React.useState(SCENARIOS[0]);
    const [shownCount, setShownCount] = React.useState(0);
    const reduced = useReducedMotion();
    React.useEffect(() => {
        if (reduced) {
            setShownCount(activeScenario.lines.length);
            return;
        }
        setShownCount(0);
        let cancelled = false;
        let timer;
        const step = (idx) => {
            if (cancelled)
                return;
            if (idx >= activeScenario.lines.length)
                return;
            setShownCount(idx + 1);
            const currentLine = activeScenario.lines[idx];
            const pause = 220 + (currentLine?.hold ?? 0);
            timer = setTimeout(() => step(idx + 1), pause);
        };
        timer = setTimeout(() => step(0), 300);
        return () => {
            cancelled = true;
            clearTimeout(timer);
        };
    }, [activeScenario, reduced]);
    return (_jsxs("div", { className: "flex w-full flex-col gap-2.5", children: [_jsx("div", { className: "flex flex-wrap items-center justify-center gap-1 sm:justify-start", children: SCENARIOS.map((scenario) => (_jsx("button", { onClick: () => setActiveScenario(scenario), className: cn("rounded-full px-3 py-1 font-mono text-[11px] font-medium transition-all cursor-pointer", activeScenario.id === scenario.id
                        ? "border border-kai-orange/40 bg-kai-orange/15 text-kai-orange shadow-sm"
                        : "border border-white/5 bg-[#0a0b10] text-kai-muted hover:border-white/15 hover:text-white"), children: scenario.label }, scenario.id))) }), _jsx(TerminalWindow, { title: activeScenario.title, status: activeScenario.status, className: "min-h-[380px]", children: _jsxs("div", { className: "space-y-1.5 overflow-x-auto", children: [activeScenario.lines.slice(0, shownCount).map((line, idx) => (_jsx("div", { className: cn("leading-relaxed", LINE_STYLES[line.kind]), children: line.text }, idx))), shownCount < activeScenario.lines.length && (_jsx("div", { className: "inline-block size-2 bg-kai-orange animate-pulse" }))] }) })] }));
}

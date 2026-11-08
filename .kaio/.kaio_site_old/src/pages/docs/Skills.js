import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import CodeBlock from "@/components/CodeBlock";
import { C, Callout, DocPage, H2, LI, P, UL } from "@/components/docs/parts";
const SKILL_MD = `---
name: add-a-tui-command
description: How to add a slash command to the Kaioken TUI. Use when adding,
  renaming or removing TUI commands.
sources:
  - internal/tui/tui.go
generated_at: 2026-07-24T19:02:36Z
---

# Add a TUI command

## Steps
1. Add a case to \`dispatch\` …`;
export default function Skills() {
    return (_jsxs(DocPage, { title: "Skills", lead: "The wiki explains what a codebase contains. A skill explains how to do a task in it \u2014 which files to touch, in what order, following which local conventions.", children: [_jsx(H2, { id: "why", children: "Why skills" }), _jsx(P, { children: "A description of the architecture is not what an agent needs at the moment it starts working. It needs the procedure: the files, the order, the local rules \u2014 exactly what a general model cannot know about your project." }), _jsxs(P, { children: ["Kaioken plans the recurring tasks in your repo \u2014 ", _jsx(C, { children: "add-an-api-endpoint" }), ",", " ", _jsx(C, { children: "add-a-cli-command" }), ", ", _jsx(C, { children: "write-a-test" }), ", ", _jsx(C, { children: "run-a-migration" }), ", whatever fits your stack \u2014 then writes one grounded guide per task."] }), _jsx(H2, { id: "layout", children: "Layout" }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: ".kaioken/skills/", code: `.kaioken/skills/
├─ README.md                     the catalog an agent reads first
└─ add-a-tui-command/
   └─ SKILL.md` }) }), _jsx(H2, { id: "format", children: "The Agent Skills format" }), _jsxs(P, { children: ["Each ", _jsx(C, { children: "SKILL.md" }), " carries frontmatter so runtimes that understand the format can load skills on demand by matching the ", _jsx(C, { children: "description" }), "."] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "SKILL.md", code: SKILL_MD }) }), _jsx(P, { children: "The body is procedural, not descriptive: prerequisites, numbered steps naming real files, the conventions that are not obvious from the code, how to verify the change, and the mistakes people actually make in this codebase." }), _jsx(H2, { id: "current", children: "They stay current" }), _jsxs(P, { children: ["The ", _jsx(C, { children: "sources" }), " list is what makes skills incremental. ", _jsx(C, { children: "kaioken update" }), " diffs the repo and refreshes only the skills whose sources the change touched \u2014 the same mechanism that keeps the wiki current."] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "powershell", prompt: true, code: `kaioken skills            # plan and build the set
kaioken skills list       # see what exists
kaioken skills -force     # rewrite them all
kaioken update            # refreshes wiki AND the affected skills` }) }), _jsxs(Callout, { kind: "tip", title: "When to run it", children: ["The TUI suggests ", _jsx(C, { children: "/skills" }), " once a wiki or card run finishes, since that is when there is something to build on."] }), _jsx(H2, { id: "agents", children: "How agents pick them up" }), _jsxs(UL, { children: [_jsxs(LI, { children: ["Kaioken's own chat agent lists skills first in its knowledge catalog, and is told to open a matching one ", _jsx("em", { children: "before" }), " starting a task."] }), _jsxs(LI, { children: ["For other workflows, point at the catalog from ", _jsx(C, { children: "AGENTS.md" }), "."] })] }), _jsx("div", { className: "pt-4", children: _jsx(CodeBlock, { title: "AGENTS.md", code: `## Project knowledge
Before starting a task, check .kaioken/skills/README.md — if a skill matches
what you are doing, follow it. For unfamiliar areas, read
.kaioken/KNOWLEDGE.md and the cards for the modules you touch, or the
relevant chapter in .kaioken/wiki/.` }) })] }));
}

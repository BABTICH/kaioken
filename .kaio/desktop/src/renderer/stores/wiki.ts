// src/renderer/stores/wiki.ts
import { atom } from "nanostores";

export interface WikiPage {
  id: string;
  title: string;
  path: string;
  content: string;
  updatedAt: number;
}

export interface WikiOutlineItem {
  id: string;
  level: number;
  text: string;
}

export interface WikiState {
  pages: WikiPage[];
  activePageId: string | null;
  outline: WikiOutlineItem[];
}

const initialState: WikiState = {
  pages: [],
  activePageId: null,
  outline: []
};

export const wikiStore = atom<WikiState>(initialState);

function set(patch: Partial<WikiState>): void {
  wikiStore.set({
    ...wikiStore.get(),
    ...patch
  });
}

function createMockPages(): WikiPage[] {
  const now = Date.now();

  const overview: WikiPage = {
    id: "wiki-overview",
    title: "Overview",
    path: "wiki/overview.md",
    updatedAt: now,
    content: `# Kaioken Desktop Wiki

## Purpose
Kaioken Desktop Studio is a secure desktop control plane for grounded agent work.

## Core Guarantees
- Process isolation for the Kaiopi engine.
- Sandboxed workspace access.
- Destructive actions require approval.
- Persistent knowledge artifacts.

## Surface Map
- Chat
- Research
- Wiki
- Codemap
- Cards
- Cost
- Settings

## Runtime Flow
\`\`\`mermaid
flowchart LR
  Renderer --> Main
  Main --> Utility
  Utility --> Kaiopi
  Kaiopi --> Main
  Main --> Renderer
\`\`\`
`
  };

  const architecture: WikiPage = {
    id: "wiki-architecture",
    title: "Architecture",
    path: "wiki/architecture.md",
    updatedAt: now,
    content: `# Architecture

## Process Model
- Main process: security, IPC, PTY, approval gateway.
- Utility process: Kaiopi engine and session orchestration.
- Renderer process: React surfaces and local stores.

## IPC Planes
- Control plane: validated invoke calls.
- Data plane: batched envelopes and ACKs.

## Security Boundaries
- Renderer never receives provider secrets.
- Filesystem access is constrained to the trusted workspace root.
- Destructive tools pause until desktop approval is resolved.

\`\`\`mermaid
graph TD
  Main -->|control| Utility
  Utility -->|events| Main
  Main -->|stream| Renderer
  Renderer -->|ack| Main
\`\`\`
`
  };

  const agentTools: WikiPage = {
    id: "wiki-agent-tools",
    title: "Agent Tools",
    path: "wiki/agent-tools.md",
    updatedAt: now,
    content: `# Agent Tools

## Read Tools
- kaio_symbol_lookup
- kaio_read_file
- kaio_wiki_search

## Mutating Tools
- write_to_file
- apply_patch
- run_command

## Approval States
- proposed
- awaiting_approval
- approved
- denied
- timeout

## Tool Card Semantics
- Ember: running or success.
- Red: failure, denied, timeout, or destructive waiting state.
`
  };

  const phaseStatus: WikiPage = {
    id: "wiki-phase-status",
    title: "Phase Status",
    path: "wiki/phase-status.md",
    updatedAt: now,
    content: `# Phase Status

## Completed
- Secure shell
- Engine supervisor
- Session streaming
- Workspace and terminal
- Approval gateway
- Diff inspector skeleton
- Codemap skeleton

## Active
- Wiki surface

## Next
- Cards
- Cost ledger
- Hardening
`
  };

  return [overview, architecture, agentTools, phaseStatus];
}

export function loadMockWiki(): void {
  const pages = createMockPages();

  set({
    pages,
    activePageId: pages[0]?.id ?? null,
    outline: []
  });
}

export function setActiveWikiPage(pageId: string): void {
  set({
    activePageId: pageId,
    outline: []
  });
}

export function setWikiOutline(items: WikiOutlineItem[]): void {
  set({
    outline: items
  });
}

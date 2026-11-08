// src/renderer/components/wiki/WikiSurface.tsx
import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import {
  loadMockWiki,
  setActiveWikiPage,
  setWikiOutline,
  wikiStore
} from "@renderer/stores/wiki";
import { WikiMarkdown } from "./WikiMarkdown";
import { RotateCw, FileText } from "lucide-react";

function scrollToHeading(id: string): void {
  const el = document.getElementById(id);

  if (!el) {
    return;
  }

  el.scrollIntoView({
    behavior: "smooth",
    block: "start"
  });
}

export function WikiSurface() {
  const wiki = useStore(wikiStore);

  const pageCount = wiki.pages.length;

  useEffect(() => {
    if (pageCount === 0) {
      loadMockWiki();
    }
  }, [pageCount]);

  const activePage =
    wiki.pages.find((page) => page.id === wiki.activePageId) ??
    wiki.pages[0];

  if (!activePage) {
    return (
      <div className="wiki-empty">
        <p className="lead">Loading wiki...</p>
      </div>
    );
  }

  return (
    <div className="wiki-surface">
      <div className="wiki-toolbar">
        <span className="label label--accent">Wiki</span>

        <span className="label">pages {wiki.pages.length}</span>

        <span className="label">
          outline {wiki.outline.length}
        </span>

        <div style={{ flex: 1 }} />

        <button
          type="button"
          className="btn btn--sm inline-flex items-center gap-1"
          onClick={() => loadMockWiki()}
        >
          <RotateCw size={12} />
          <span>Reload mock</span>
        </button>
      </div>

      <div className="wiki-body">
        <aside className="wiki-pages hud-scrollbar">
          {wiki.pages.map((page) => {
            const isActive = page.id === wiki.activePageId;

            return (
              <button
                key={page.id}
                type="button"
                className={[
                  "wiki-page-item",
                  isActive ? "is-active" : ""
                ].join(" ")}
                onClick={() => setActiveWikiPage(page.id)}
              >
                <span>{page.title}</span>

                <span className="wiki-page-meta">
                  {page.path}
                </span>
              </button>
            );
          })}
        </aside>

        <main
          key={activePage.id}
          className="wiki-main hud-scrollbar"
        >
          <article className="wiki-article panel-glow">
            <WikiMarkdown
              content={activePage.content}
              onOutline={setWikiOutline}
            />
          </article>
        </main>

        <aside className="wiki-outline hud-scrollbar">
          <div className="label label--accent wiki-outline-title">
            Outline
          </div>

          {wiki.outline.length === 0 && (
            <div
              className="mono"
              style={{
                fontSize: "var(--fs-xs)",
                color: "var(--fg-mute)"
              }}
            >
              No headings found.
            </div>
          )}

          {wiki.outline.map((item) => (
            <button
              key={item.id}
              type="button"
              className={[
                "wiki-outline-item",
                `wiki-outline-level-${item.level}`
              ].join(" ")}
              onClick={() => scrollToHeading(item.id)}
            >
              {item.text}
            </button>
          ))}
        </aside>
      </div>
    </div>
  );
}

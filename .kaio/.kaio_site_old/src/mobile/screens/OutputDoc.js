import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import * as React from "react";
import { Link, useParams } from "react-router-dom";
import { ArrowLeft, ArrowRight, FileCode2 } from "lucide-react";
import Markdown from "@/components/Markdown";
import { useScreenTitle } from "@/mobile/lib/chrome";
import { RUN_COST, WIKI_FLAT, docUrl, findDoc } from "@/data/wiki";
/** Kaioken's provenance footer is metadata, not prose — strip it from the body. */
const FOOTER = /\n*<!--\s*kaioken:files[\s\S]*?-->\s*$/;
export default function OutputDoc() {
    const { section: sectionSlug, doc: docSlug } = useParams();
    // findDoc builds a new object each call; without memoising it the fetch
    // effect re-runs on every render and never settles.
    const hit = React.useMemo(() => findDoc(sectionSlug, docSlug), [sectionSlug, docSlug]);
    const [body, setBody] = React.useState(null);
    const [error, setError] = React.useState(null);
    useScreenTitle(hit?.doc.title ?? null);
    React.useEffect(() => {
        if (!hit)
            return;
        let alive = true;
        setBody(null);
        setError(null);
        fetch(docUrl(hit.section.dir, hit.doc.file))
            .then((r) => {
            if (!r.ok)
                throw new Error(`${r.status}`);
            return r.text();
        })
            .then((text) => {
            if (alive)
                setBody(text.replace(FOOTER, ""));
        })
            .catch((e) => {
            if (alive)
                setError(String(e.message ?? e));
        });
        return () => {
            alive = false;
        };
    }, [hit]);
    if (!hit) {
        return (_jsxs("div", { className: "px-4 pt-8", children: [_jsx("p", { className: "font-mono text-[13px] text-kai-rose", children: "document not found" }), _jsx(Link, { to: "/preview", className: "mt-3 inline-block font-mono text-[13px] text-kai-orange", children: "\u2190 back to the output tree" })] }));
    }
    const { section, doc } = hit;
    const index = WIKI_FLAT.findIndex((e) => e.section.slug === section.slug && e.doc.slug === doc.slug);
    const prev = index > 0 ? WIKI_FLAT[index - 1] : undefined;
    const next = index >= 0 && index < WIKI_FLAT.length - 1 ? WIKI_FLAT[index + 1] : undefined;
    return (_jsx(_Fragment, { children: _jsxs("article", { className: "px-4 pt-5 pb-4", children: [_jsxs("header", { className: "border-b border-border pb-4", children: [_jsxs("p", { className: "font-mono text-[10.5px] leading-relaxed text-kai-dim", children: [_jsx(Link, { to: "/preview", children: ".kaioken/wiki" }), _jsx("span", { "aria-hidden": true, children: " / " }), _jsx("span", { className: "text-kai-orange", children: section.title }), _jsx("span", { "aria-hidden": true, children: " / " }), _jsx("span", { className: "break-all", children: doc.file })] }), _jsxs("p", { className: "mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10.5px] text-kai-dim", children: [_jsxs("span", { children: [_jsx("span", { className: "text-kai-amber", children: RUN_COST.level }), " generated"] }), _jsxs("span", { children: [doc.words.toLocaleString(), " words"] }), doc.hasMermaid ? _jsx("span", { className: "text-kai-blue", children: "diagrams" }) : null, _jsx("a", { href: docUrl(section.dir, doc.file), target: "_blank", rel: "noopener noreferrer", className: "text-kai-orange", children: "raw \u2197" })] })] }), doc.sources.length > 0 ? (_jsxs("div", { className: "mt-4 rounded-md border border-border bg-card p-3.5", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx(FileCode2, { className: "size-3.5 text-kai-orange", "aria-hidden": true }), _jsx("span", { className: "font-mono text-[10px] tracking-[0.15em] text-kai-dim uppercase", children: "written from" })] }), _jsx("ul", { className: "m-no-scrollbar mt-2.5 flex gap-1.5 overflow-x-auto", children: doc.sources.map((src) => (_jsx("li", { className: "shrink-0 rounded-sm bg-kai-panel px-2 py-1 font-mono text-[11px] whitespace-nowrap text-kai-tan", children: src }, src))) })] })) : null, _jsx("div", { className: "pt-2 [&_h1]:text-[22px] [&_h2]:text-[18px] [&_h3]:text-[15px] [&_li]:[overflow-wrap:anywhere] [&_p]:[overflow-wrap:anywhere]", children: error ? (_jsxs("p", { className: "pt-6 font-mono text-[12.5px] text-kai-rose", children: ["could not load this document (", error, ")"] })) : body === null ? (_jsxs("p", { className: "pt-6 font-mono text-[12.5px] text-kai-dim", children: [_jsx("span", { className: "animate-caret text-kai-orange", children: "\u258E" }), " loading\u2026"] })) : (_jsx(Markdown, { sectionDir: section.dir, children: body })) }), _jsxs("nav", { className: "mt-10 space-y-2.5 border-t border-border pt-6", children: [prev ? (_jsxs(Link, { to: `/preview/${prev.section.slug}/${prev.doc.slug}`, className: "m-press flex min-h-[60px] items-center gap-3 rounded-md border border-border bg-card px-4 py-3", children: [_jsx(ArrowLeft, { className: "size-4 shrink-0 text-kai-dim", "aria-hidden": true }), _jsxs("span", { className: "min-w-0", children: [_jsx("span", { className: "block truncate font-mono text-[10px] tracking-[0.12em] text-kai-dim uppercase", children: prev.section.title }), _jsx("span", { className: "mt-0.5 block font-mono text-[13px] font-bold text-foreground", children: prev.doc.title })] })] })) : null, next ? (_jsxs(Link, { to: `/preview/${next.section.slug}/${next.doc.slug}`, className: "m-press flex min-h-[60px] items-center gap-3 rounded-md border border-kai-orange/35 bg-kai-orange/[0.07] px-4 py-3", children: [_jsxs("span", { className: "min-w-0 flex-1", children: [_jsx("span", { className: "block truncate font-mono text-[10px] tracking-[0.12em] text-kai-dim uppercase", children: next.section.title }), _jsx("span", { className: "mt-0.5 block font-mono text-[13px] font-bold text-kai-orange", children: next.doc.title })] }), _jsx(ArrowRight, { className: "size-4 shrink-0 text-kai-orange", "aria-hidden": true })] })) : null, _jsx(Link, { to: "/preview", className: "m-press flex min-h-[48px] items-center justify-center rounded-md border border-border px-4 font-mono text-[12.5px] text-muted-foreground", children: "back to the output tree" })] })] }) }));
}

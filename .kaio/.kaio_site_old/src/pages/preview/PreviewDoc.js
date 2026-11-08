import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import * as React from "react";
import { Link, useParams } from "react-router-dom";
import { ArrowLeft, ArrowRight, FileCode2 } from "lucide-react";
import Markdown from "@/components/Markdown";
import { RUN_COST, WIKI_FLAT, docUrl, findDoc } from "@/data/wiki";
/** Kaioken's provenance footer is metadata, not prose — strip it from the body. */
const FOOTER = /\n*<!--\s*kaioken:files[\s\S]*?-->\s*$/;
export default function PreviewDoc() {
    const { section: sectionSlug, doc: docSlug } = useParams();
    // findDoc builds a new object each call; without memoising it the fetch
    // effect re-runs on every render and never settles.
    const hit = React.useMemo(() => findDoc(sectionSlug, docSlug), [sectionSlug, docSlug]);
    const [body, setBody] = React.useState(null);
    const [error, setError] = React.useState(null);
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
        return (_jsxs("article", { className: "min-w-0 pb-20", children: [_jsx("p", { className: "font-mono text-[13px] text-kai-rose", children: "document not found" }), _jsx(Link, { to: "/preview", className: "mt-3 inline-block font-mono text-[13px] text-kai-orange", children: "\u2190 back to the output tree" })] }));
    }
    const { section, doc } = hit;
    const index = WIKI_FLAT.findIndex((e) => e.section.slug === section.slug && e.doc.slug === doc.slug);
    const prev = index > 0 ? WIKI_FLAT[index - 1] : undefined;
    const next = index >= 0 && index < WIKI_FLAT.length - 1 ? WIKI_FLAT[index + 1] : undefined;
    return (_jsxs("article", { className: "min-w-0 pb-20", children: [_jsxs("header", { className: "border-b border-border pb-5", children: [_jsxs("p", { className: "flex flex-wrap items-center gap-1.5 font-mono text-[11px] text-kai-dim", children: [_jsx(Link, { to: "/preview", className: "transition-colors hover:text-kai-orange", children: ".kaioken/wiki" }), _jsx("span", { "aria-hidden": true, children: "/" }), _jsx("span", { className: "text-kai-orange", children: section.title }), _jsx("span", { "aria-hidden": true, children: "/" }), _jsx("span", { className: "text-muted-foreground", children: doc.file })] }), _jsxs("p", { className: "mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[11px] text-kai-dim", children: [_jsxs("span", { children: [_jsx("span", { className: "text-kai-amber", children: RUN_COST.level }), " generated"] }), _jsxs("span", { children: [doc.words.toLocaleString(), " words"] }), doc.hasMermaid ? _jsx("span", { className: "text-kai-blue", children: "contains diagrams" }) : null, _jsx("a", { href: docUrl(section.dir, doc.file), target: "_blank", rel: "noopener noreferrer", className: "transition-colors hover:text-kai-orange", children: "view raw markdown \u2197" })] })] }), doc.sources.length > 0 ? (_jsxs("div", { className: "mt-5 rounded-sm border border-border bg-card p-4", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx(FileCode2, { className: "size-3.5 text-kai-orange" }), _jsx("span", { className: "font-mono text-[11px] tracking-wider text-kai-dim uppercase", children: "written from" })] }), _jsx("ul", { className: "mt-2 flex flex-wrap gap-1.5", children: doc.sources.map((src) => (_jsx("li", { className: "rounded-sm bg-kai-panel px-2 py-0.5 font-mono text-[11.5px] text-kai-tan", children: src }, src))) })] })) : null, _jsx("div", { className: "pt-6", children: error ? (_jsxs("p", { className: "font-mono text-[13px] text-kai-rose", children: ["could not load this document (", error, ")"] })) : body === null ? (_jsx("p", { className: "font-mono text-[13px] text-kai-dim", children: "loading\u2026" })) : (_jsx(Markdown, { sectionDir: section.dir, children: body })) }), _jsxs("nav", { className: "mt-14 grid gap-3 border-t border-border pt-6 sm:grid-cols-2", children: [prev ? (_jsxs(Link, { to: `/preview/${prev.section.slug}/${prev.doc.slug}`, className: "group rounded-sm border border-border bg-card p-4 transition-colors hover:border-kai-orange/45", children: [_jsxs("span", { className: "flex items-center gap-1.5 font-mono text-[11px] text-kai-dim", children: [_jsx(ArrowLeft, { className: "size-3" }), prev.section.title] }), _jsx("span", { className: "mt-1 block font-mono text-[13.5px] font-bold text-foreground transition-colors group-hover:text-kai-orange", children: prev.doc.title })] })) : (_jsx("span", {})), next ? (_jsxs(Link, { to: `/preview/${next.section.slug}/${next.doc.slug}`, className: "group rounded-sm border border-border bg-card p-4 text-right transition-colors hover:border-kai-orange/45 sm:col-start-2", children: [_jsxs("span", { className: "flex items-center justify-end gap-1.5 font-mono text-[11px] text-kai-dim", children: [next.section.title, _jsx(ArrowRight, { className: "size-3" })] }), _jsx("span", { className: "mt-1 block font-mono text-[13.5px] font-bold text-foreground transition-colors group-hover:text-kai-orange", children: next.doc.title })] })) : null] })] }));
}

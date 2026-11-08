import manifest from "./wiki-manifest.json";
export const WIKI_SECTIONS = manifest.sections;
export const WIKI_README = manifest.readme;
export const WIKI_STATS = manifest.stats;
/**
 * What this run actually cost, and what the same repo would cost at ×10.
 * The shipped output is a default ×3 run.
 */
export const RUN_COST = {
    level: "×3",
    levelNote: "default depth",
    tokens: "~1.3M",
    tokensLong: "≈ 1,300,000 tokens",
    deepLevel: "×10",
    deepTokens: "~2.3M",
    deepTokensLong: "≈ 2,300,000 tokens",
};
/** Public URL of a document's raw markdown. */
export function docUrl(sectionDir, file) {
    return `/kaioken/wiki/${encodeURIComponent(sectionDir)}/${encodeURIComponent(file)}`;
}
export function findSection(slug) {
    return WIKI_SECTIONS.find((s) => s.slug === slug);
}
export function findDoc(sectionSlug, docSlug) {
    const section = findSection(sectionSlug);
    if (!section)
        return undefined;
    const doc = section.docs.find((d) => d.slug === docSlug);
    return doc ? { section, doc } : undefined;
}
/** Flat reading order, for prev/next inside the preview. */
export const WIKI_FLAT = WIKI_SECTIONS.flatMap((s) => s.docs.map((d) => ({ section: s, doc: d })));

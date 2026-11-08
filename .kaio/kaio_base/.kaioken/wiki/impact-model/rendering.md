# Rendering the Report

The impact module's public output is plain text — an array of strings, one per rendered section — produced by `renderImpact` in `kaioken/impact/src/render.ts`. It takes an `ImpactReport` and returns something a human can read directly, without any HTML or terminal styling.

```ts
kaioken/impact/src/render.ts:9-55
function renderImpact(report: ImpactReport): string[]
```

The function is deliberately shaped around the shape of the report itself. Because an `ImpactReport` is a composite structure — containing a summary plus multiple typed sections — `renderImpact` walks the report and emits a section per entry. Each section is formatted as a titled block, separated by blank lines, so the returned array aligns one-to-one with the report's logical sections.

The rendering is **lossy by design**: it does not attempt to re-serialize the entire `ImpactReport` object for round-tripping. Instead it extracts the human-meaningful parts — the headline, the affected components, the reasoning — and discards internal metadata that only matters to serialization or provenance tracking. This keeps the output stable and readable even if the underlying report gains new internal fields.

Notably, `renderImpact` returns `string[]`, not a single string. That choice matters: it lets callers (such as the serve module, or CLI tools) decide whether to join with newlines, print section by section, or render each section as a separate block in a UI. The function itself makes no assumptions about the final presentation medium.

The implementation has no dependencies on other modules — it is a pure transformation from the report object to text. That isolation is intentional: the rendering logic is the one place where the impact data model meets human eyes, so it stays free of graph lookups or provenance checks. If the report format changes, only this function (and its tests) need to change.

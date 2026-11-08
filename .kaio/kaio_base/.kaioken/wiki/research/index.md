# Research Infrastructure

The research module in `kaioken/research` implements a defense-in-depth pipeline: it queries web sources through a port abstraction, fetches pages, sanitizes them against prompt injection and private-network attacks, and finally composes a model prompt with only safe, fenced excerpts. The module is deliberately split so that `ports.ts` defines the contracts and `run.ts` orchestrates the workflow; `sanitize.ts` is the security-critical piece.

## Ports and Data Shapes (`src/ports.ts`)

The module's boundaries are defined by two ports. `WebSearchPort` matches a search provider that accepts a `query` and a numeric `limit`, returning `Promise<WebHit[]>`; each `WebHit` is an opaque interface (lines 17–22) with enough fields for a URL and title but no assumed schema beyond that. `WebFetchPort` (lines 42–44) is even narrower—given a `url`, it returns a `Promise<WebFetchResult>` containing the fetched content and metadata. These ports are why `run.ts` never imports a concrete HTTP client; the tests supply fakes, and production binds real implementations.

The module exposes two pure helpers on these shapes. `dedupeHits` (lines 54–64) removes duplicate `WebHit` entries by a rule that is unspecified in the declaration, so callers must rely on the implementation—likely comparing both URL and title. `numberSources` (lines 239–241) takes `readonly ResearchSource[]` and returns a new array where each source is assigned an ordinal; this is used downstream by `buildPrompt` to present sources as `[1]`, `[2]`, etc.

## Sanitization and Security (`src/sanitize.ts`)

The most critical function here is `htmlToText` (lines 42–143). It strips markup and converts HTML entities to plain text; the long line count hints at handling many edge cases (e.g., `<script>` removal, whitespace normalization, decoding of numeric character references). Any vulnerability here—like a crafted page that survives as raw HTML in the prompt—would be a prompt-injection vector, so this function is the first line of defense.

Two smaller helpers support the writing phase. `excerptOf` (lines 189–197) clips a string to `maxChars`, presumably on a token or character boundary, returning a `SourceExcerpt`. `fenceSource` (lines 177–180) wraps an excerpt in code fences—likely with a language hint like `text`—so the excerpt appears as verbatim content in the prompt, not as instructions the model should execute.

`injectionPatterns` (lines 206–221) is the anomaly detector. Given `text`, it returns `string[]` of patterns that look like prompt-injection attempts—things like "ignore previous instructions", "system:", or similar jailbreak idioms. This is used by `run.ts` to reject or quarantine suspicious sources before they reach the prompt.

## Fetchable-URL Validation (`src/ports.ts` lines 89–176)

Before any fetch happens, `isFetchableUrl` (lines 89–111) does a synchronous conservative check on the URL—likely rejecting `data:`, `file:`, and other non-`http(s)` schemes. But DNS rebinding attacks can bypass that, so `isFetchableUrlResolved` (lines 126–167) is an async counterpart that takes an optional `DnsLookupFn` (a type alias, lines 113–115, that returns either an IP array or a hostname string). It resolves the hostname and checks each IP with `isPrivateIp` (lines 169–176), which rejects RFC1918 and loopback addresses (10.x, 172.16–31.x, 192.168.x, 127.x). The presence of this function signals the module is running in an untrusted network context where a compromised DNS could redirect the server to an internal resource.

## Orchestration (`src/run.ts`)

The public entry points are `gatherSources` and `generateResearch`. `gatherSources` (lines 101–212) takes a `GatherInput` (lines 48–59) that includes the question (or search query), a limit, and the two ports; it returns a `GatherResult` (lines 61–68) that carries both the raw `WebHit[]` and the matched `ResearchSource[]` with excerpts. The likely flow: call `search`, dedupe hits, filter out unfetchable URLs, fetch candidates, run `htmlToText`, produce `excerptOf` for each, and reject any that trip `injectionPatterns`.

`generateResearch` (lines 237–292) takes a `GenerateInput` (lines 218–223)—which must contain the already-gathered sources and a `question`—and returns a `GenerateResult` (lines 225–228) with the final text. It calls `buildPrompt` (lines 297–310), which concatenates the question, the numbered excerpts (via `numberSources`), and instructions to answer only from the fenced content. `pathFor` (lines 343–350) derives a filesystem path from the question string; the implementation probably hashes or slugifies it to produce a stable artifact location.

## Assembly Notes

- `index.ts` is empty of indexed declarations; it likely re-exports from the other files but the evidence doesn't show that, so treat each file's exports as the public API.
- The `tsconfig.json` and `package.json` are unremarkable—strict TypeScript, Node ESM, no bundled dependencies visible in the evidence.
- The orchestration order is strictly: `gatherSources` (validate & sanitize) → `generateResearch` (compose & prompt) → write to `pathFor`. Nothing bypasses `htmlToText` or `fenceSource`, which is the security invariant of the module.

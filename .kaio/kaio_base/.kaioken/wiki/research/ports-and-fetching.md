# Research Infrastructure: Ports, Fetching, and URL Safety

The research module discovers sources, fetches them, and produces a sanitized set of documents. At its boundary sit two ports that decouple the research pipeline from any concrete search engine or HTTP client.

## Ports: searching and fetching without coupling

`WebSearchPort` is the search abstraction. Its only method, `WebSearchPort.search`, takes a `query` string and a `limit`, and returns `Promise<WebHit[]>`. The contract is deliberately strict: a failed search is a throw, not an empty result — callers must treat an empty hit list as "no results found", distinct from "search infrastructure is down".

`WebHit` is the shape of one search result, carrying the fields you'd expect: `title`, `url`, `snippet`, and a `score` for ranking.

`WebFetchPort` is the fetching side. Its single method, `WebFetchPort.fetch(url: string): Promise<WebFetchResult>`, returns the fetched page's content and metadata. Again, failure is exceptional: the implementation may throw on non-2xx status, timeouts, or invalid responses; the caller decides how to surface that.

Neither port knows about URLs beyond the `string` type. All URL-level decisions — what's safe to fetch, how to normalize, what to dedupe — live in the free functions alongside them.

## Normalization and deduplication

`normaliseUrl` is a small helper that treats URLs differing only by trailing slash or fragment as the same page. Why bother? `dedupeHits` relies on it: `dedupeHits` collapses `WebHit[]` across multiple queries, using the normalized URL as the identity key. Two search queries may return the same page with slightly different spellings; this prevents the downstream pipeline from ingesting the same document twice under two URLs.

Note the British spelling — `normaliseUrl` — and the lowercase `e` in `dedupeHits`. These names are baked into the module's public surface and are used by other chapters (the Facts module, for instance, consumes `dedupeHits` on a `Map<string, WebHit>` keyed by the normalized URL).

## URL safety: who may become a source

Not every URL a search returns is fetchable. `isFetchableUrl` performs the synchronous, syntax-level check: it parses the URL and rejects anything whose host matches `BLOCKED_HOST_SUFFIXES` — `"localhost"`, `"0.0.0.0"`, `".local"`, `".internal"` — by suffix. A search result may *never* point at your own machine or an internal network:

```ts
const BLOCKED_HOST_SUFFIXES = ["localhost", "0.0.0.0", ".local", ".internal"];
```

Suffix matching means `myhost.local` is blocked, but so is a bare `localhost` — and a domain like `notlocal` is *not* blocked because `.local` must appear at the end of the host.

But suffix blocking alone is not enough. An attacker who controls a search result can point a public domain at a private IP (or flip its resolution after the check — the classic DNS rebinding attack). That's why `isFetchableUrlResolved` exists.

## Resolution-time validation: `isFetchableUrlResolved` and `isPrivateIp`

`isFetchableUrlResolved` is the async counterpart to `isFetchableUrl`; it re-validates at the moment of resolution. It takes an optional `DnsLookupFn` — a function from `hostname` to a `Promise` of either `Array<{ address: string; family: number }>` or a `string` — and resolves the hostname *then* checks `isPrivateIp` on every returned address.

The injection point matters: by default it uses the platform's resolver, but tests can pass a stub `DnsLookupFn` that simulates an attacker's DNS server. The contract is loose on purpose — the resolver may return either a structured array (Node's `dns.lookup` shape) or a raw string (e.g. `"127.0.0.1"`), and the function normalizes both.

`isPrivateIp` is the classifier. For IPv6 addresses, it delegates to `isPrivateIpv6`; otherwise it treats the input as IPv4 (via `isPrivateIpv4`). If the address cannot be parsed as either, it's rejected — an unparseable address is not a public one.

`isPrivateIpv4` and `isPrivateIpv6` implement the RFC-defined private ranges by literal inspection:

- **IPv4**: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, plus loopback (`127.0.0.0/8`) and link-local (`169.254.0.0/16`). Checking octets literally avoids a dependency on a networking library.
- **IPv6**: the same idea expressed in 128-bit space — loopback (`::1`), link-local (`fe80::/10`), and the unique-local range (`fc00::/7`), which includes both `fc00::/8` and `fd00::/8`.

IPv6 makes literal checking tricky because the same address has many spellings — `::ffff:192.168.1.1` and `0:0:0:0:0:ffff:c0a8:101` are the same address. That's what `embeddedIpv4` handles: it extracts the IPv4 address inside an IPv4-mapped IPv6 address, in either spelling, so `isPrivateIpv6` can hand the embedded part back to `isPrivateIpv4`. The deep dive into literal octets and double-colon expansions exists precisely so that no private range slips through disguised as public.

The flow in production is: `isFetchableUrl` first (cheap, no I/O), then `isFetchableUrlResolved` (DNS lookup plus `isPrivateIp` per address). Both must pass before a fetch is attempted.

## Assigning citation numbers

When sources survive validation, `numberSources` stamps each with a citation number. It takes `readonly ResearchSource[]` and returns the same array with each element's `number` field set to `[1]`..`[N]`, in the order sources will be presented. The numbering is stable — it depends only on the input order, which the caller controls (e.g. ranking by the search's `score` before calling). This keeps citation numbers meaningful in the final output rather than reflecting insertion order.

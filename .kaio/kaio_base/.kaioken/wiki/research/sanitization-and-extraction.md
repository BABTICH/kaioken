# HTML Sanitization and Excerpt Extraction

## Deterministic HTML-to-Text Conversion

`htmlToText` is the workhorse of research content cleaning. It must convert arbitrary fetched HTML into plain text, *deterministically* — the same input always produces the same output, which is essential for reproducible research artifacts and for comparing documents across scans.

The function operates as a character state machine rather than a DOM parser. This is a deliberate design choice: it keeps the function dependency-free, avoids differences in parser implementations, and makes it trivially testable against a known corpus. The state machine tracks whether we are inside a tag, an attribute value (single-quoted or double-quoted), or regular text, and handles comment blocks. While a tree-walking DOM parser would also work, this approach is lighter and more predictable for the controlled set of markup a research fetcher typically receives.

Two tag sets drive the transformation:

- `SKIP_TAGS` — a `Set` containing `script`, `style`, `noscript`, `template`, and `svg`. When the state machine encounters a closing tag in this set, it discards everything inside the element. This removes executable JavaScript, presentation CSS, and non-text content like embedded vector graphics that would otherwise pollute the excerpt with gibberish.
- `BLOCK_TAGS` — a `Set` covering structural elements: `p`, `div`, `li`, `tr`, headings `h1`–`h6`, `section`, `article`, `blockquote`, `header`, `footer`, `nav`, `main`, and `aside`. When the state machine crosses a boundary of one of these tags, it inserts a newline in the output. This preserves paragraph structure and visual readability in excerpt form, while collapsing inline markup (like `<strong>` or `<a>`) to nothing.

The emphasis on deterministic behavior also means the function is safe to re-run on cached HTML: re-sanitizing a page produces byte-identical text, which lets downstream caching and diffing logic compare excerpts without ambiguity.

## Entity Decoding

`decodeEntities` is the second pipeline stage. HTML from the wild uses numeric and named character references — `&#8212;` for an em dash, `&amp;` for an ampersand, and so on. The sanitizer pipeline runs `htmlToText` first, then `decodeEntities`, so that the newline-insertion logic in the state machine never trips on semicolons or ampersands that are actually entity syntax.

The function decodes both numeric references (decimal and hexadecimal) and named references. It is deliberately forgiving: malformed or unknown entities are left as-is rather than throwing, because research fetching must be resilient to HTML from real-world sites. The decode step is applied once, after tag stripping, so entities inside skipped elements are never decoded (their text was discarded).

## Excerpt Structuring

`excerptOf` converts the full plain-text page into a bounded, structured excerpt for downstream use. It takes the sanitized `text` and a `maxChars` budget, and returns a `SourceExcerpt`. The budget is a soft limit on the *character* count of the excerpt body — the function will honor it by hard-cutting the text at the nearest boundary, avoiding the expense of running full prose segmentation for every fetched page.

Because research documents are often long and the model context window is finite, the excerpt is the unit that gets embedded into the knowledge graph and later presented to models. Its shape (a bounded text body) is what keeps downstream prompts predictable in size.

## Fencing for Source Presentation

`fenceSource` wraps a `SourceExcerpt` in a fenced block that renders the content as numbered source lines. The fence is more than cosmetic: it is a security boundary. By enclosing sanitized content between explicit markers and assigning line numbers, the function prevents prompt-injection when the excerpt is later shown to a model or user as part of a research document.

The use of numbered lines is deliberate — it gives downstream consumers a stable reference anchor for quoting specific parts of the excerpt, and it makes it visually obvious that the content inside the fence is *data* (source text), not instructions. The fenced block is the final shape in which sanitized content reaches the model.

## Injection Pattern Detection

`injectionPatterns` complements the fencing strategy. It scans sanitized text for known patterns that indicate a prompt-injection attempt — phrases or structures that try to override instructions embedded in the document content. The function returns an array of matched patterns, so callers can log, flag, or quarantine documents that exhibit suspicious content.

This is a best-effort heuristic, not a security guarantee. But combined with `fenceSource`, it gives the research pipeline two independent lines of defense: content is structurally isolated by the fence, and semantically suspicious content is surfaced for human review.

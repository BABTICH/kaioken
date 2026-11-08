# kaioken-pi

A deterministic truth layer for [Pi](https://pi.dev). It answers questions about
a repository from an index of the repository rather than from a model's memory,
and it refuses to call work finished until the tests say so.

## Install

```bash
pi install npm:kaioken-pi
```

## What you get

A header, seven tools, sixteen commands, two themes, and a hook that blocks
destructive shell commands.

### The header

The KAIOKEN wordmark under its amber→red gradient, beside a panel that shows
what you otherwise cannot see together: repo, branch, model, provider, whether
a key is set, and what `.kaioken/` currently holds — including how much of it is
still true.

It animates twice and then stops. A boot curtain on startup (the wordmark rises
a row at a time, an aura opens under it, three lines type themselves out) and a
CRT power-off on quit. Nothing else moves: DESIGN.md's axiom is *if everything
glows, nothing communicates*, so the only other motion is a rule that sweeps
while a run is actually in flight.

Set `NO_MOTION=1` or `NO_COLOR=1` to switch all of it off.

### The tools

| Tool | What it answers |
| --- | --- |
| `kaio_symbol_lookup` | Does this symbol exist, and where? Or a negative guarantee. |
| `kaio_read_file` | Exact line ranges with verified anchors, for quoting code. |
| `kaio_wiki_search` | Search the generated wiki, cards and skills. |
| `kaio_impact` | What breaks if this symbol changes. |
| `kaio_skill_load` | Load a task procedure distilled from this repository. |
| `kaio_status` | What has drifted since the documents were written. |
| `kaio_verify` | Run the repository's own tests, build and lint. |

### The commands

Sixteen `/kaio-*` commands for the pipeline: `scan`, `symbols`,
`search`, `status`, `verify`, `graph`, `serve`, `export`, `plan`,
`cards`, `wiki`, `update`, `research`, `skills`, `delegate`,
`merge`.

## What it costs

Nothing until you ask for it. The scanning, indexing, search, graph, impact and
verification paths are entirely offline — no network, no API key, no model. They
run on a fresh clone with no configuration.

The generative stages (`plan`, `cards`, `wiki`, `update`, `research`,
`skills`) need a model bound to the session. Every one of them estimates its
cost from the active model's published rates and asks before spending, and the
multiplier dial (`×1`–`×10`) sets the budget up front.

## What it writes

Everything goes under `.kaioken/` in the repository being worked on:

| Path | Contents |
| --- | --- |
| `.kaioken/scan.json` | Every file, with hashes. |
| `.kaioken/index.json` | Every declaration, by AST. |
| `.kaioken/search-index/` | The BM25 corpus. |
| `.kaioken/wiki/` | Generated chapters. |
| `.kaioken/cards/` | Per-module knowledge cards. |
| `.kaioken/skills/` | Task procedures. |
| `.kaioken/provenance.json` | What each document was written from. |
| `.kaioken/verification.json` | Which claims held up, and which did not. |

Nothing outside `.kaioken/` is written, and nothing is written outside the
repository. Delete `.kaioken/` to remove every trace.

## Honest limits

- The wiki, cards and skills are model output. Every claim in them is checked
  against the index at generation time and the failures are recorded in
  `.kaioken/verification.json`, but a claim that passed is still a claim.
  Where a document and the code disagree, the code is right.
- Grounding is only as good as the index. A language without a parser is
  indexed by file, not by declaration, and the oracle will say so rather than
  guess.
- The verifier runs your repository's own tests. It has no opinion about
  whether those tests are any good.

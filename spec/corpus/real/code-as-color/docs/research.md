# Research and citation

One Markdown record per source in `research/sources/<key>.md`. Bibliographic
data in frontmatter; in the body, the distilled claims the book is permitted to
rely on.

```markdown
---
key: cie-15-colorimetry
type: standard
title: "Colorimetry, 4th Edition"
year: 2018
publisher: International Commission on Illumination (CIE)
number: CIE 015:2018
doi: 10.25039/TR.015.2018
tier: primary
author:
  - "International Commission on Illumination"
---

## Why it matters

One paragraph on why this source is in the bibliography at all.

## Claims

- [high | §7.1] {#ybar-is-v-lambda} ȳ(λ) is identical to the photopic luminous
  efficiency function V(λ), by construction of the XYZ basis. #observer
```

## The rules

- **Metadata is looked up, never typed.** `cite add` takes a DOI, an arXiv id or
  a URL and fetches from Crossref, arXiv or the page's own metadata, with
  retries. What it writes is for a human to check — the engine fills the record
  in, it does not vouch for it.
- **Every claim needs a locator.** `[confidence | §3.2]`. A claim with no
  locator is an assertion, and the parser refuses it.
- **Confidence is one of** `high`, `medium`, `low`, `disputed`. A `disputed`
  claim left in the store is surfaced by `check` every time, on purpose.
- **`web` sources must record `accessed:`**, and go stale after a year.

## What `cite check` audits

| Finding | Meaning |
|---|---|
| `unknown-citation` | `@key` in the prose with no record |
| `unknown-figure-source` | a figure cites a source that does not exist |
| `unsourced-claim` | a figure asserts a `claim` with no `sources` |
| `no-claims` | a source has been collected, not read |
| `stale-access` | a web source last checked over a year ago |
| `unidentifiable` | no DOI, ISBN, URL or standard number |
| `unused-source` | carried but never cited |
| `disputed-claim` | still in the store, still disputed |

## Evidence sheets

```sh
node engine/cli.ts research evidence   # per-chapter sources and claims
node engine/cli.ts research claims     # the full claim index, greppable
```

The evidence sheet is what you read *before* writing a chapter and check
*after*. The Typst bibliography (`build/research/bibliography.yml`, Hayagriva)
is generated from the store and never edited.

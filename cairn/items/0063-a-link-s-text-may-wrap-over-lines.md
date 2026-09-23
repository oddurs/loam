---
id: 63
title: A link's text may wrap over lines
type: spec
status: backlog
milestone: v0.1
created: 2026-09-22
updated: 2026-09-22
priority: p2
---

## Current text

Spec §6.1, version 1 as closed in v0.0: "The link must lie on one line."

## Proposed text

The text may run over several lines of one paragraph, as wrapped prose makes it
do; the destination, and the title and `)` after it, are on one line. A link's
line is the line its destination is on. Code spans are found in a paragraph as
a whole, so one may run over a line break too.

## What a conforming reader has to do differently

Search a paragraph's lines joined, not each line alone. Found by `loam mv` on a
copy of poptop: its README links `[… what it will\nnot](docs/design/…)` across a
line break, and nothing rewrote it, because nothing had found it. Neither Python
reader nor the independent second reader found wrapped links; poptop's guide
pages have four the corpus had never recorded.

This changes the reading of pages that already exist, which spec §9 says costs
a version number. It is made at version 1 anyway, and recorded here, because
version 1 has had no release and no reader but the two in this repository; the
alternative is a format whose first release knowingly misses links in every
wrapped paragraph.

## Acceptance criteria

- [ ] spec/README.md updated
- [ ] A corpus case exercises it

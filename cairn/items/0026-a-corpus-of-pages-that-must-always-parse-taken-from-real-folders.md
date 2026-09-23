---
id: 26
title: A corpus of pages that must always parse, taken from real folders
type: chore
status: backlog
milestone: v0.0
depends_on:
- 23
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- format
effort: s
---

## What

`spec/corpus/`: every page from code-as-color, poptop and measure-of-the-world's
docs folders, copied as they are, with the expected reading of each beside it.
Plus the hostile cases cairn's corpus taught:

- no frontmatter; frontmatter with no closing delimiter
- a BOM; CRLF line endings; mixed
- no H1; two H1s; an H1 that is not the first line
- YAML 1.1 booleans unquoted (`status: no`) — cairn `0118`
- a Unicode title whose slug would put a combining mark in a filename — cairn `0119`

And `spec/conformance.py`, a reader independent of loam's code, as cairn has.

A page that has ever parsed stays in the corpus for ever (cairn `0087`).

## Acceptance criteria

- [ ] All three folders present, each page with its expected reading
- [ ] Every hostile case above present
- [ ] conformance.py passes against the corpus

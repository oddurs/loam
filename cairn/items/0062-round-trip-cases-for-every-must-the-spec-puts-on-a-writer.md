---
id: 62
title: Round-trip cases for every must the spec puts on a writer
type: chore
status: backlog
milestone: v0.1
depends_on:
- 32
created: 2026-09-22
updated: 2026-09-22
priority: p2
pillar:
- format
effort: s
---

## What

The corpus in `spec/corpus` is a reading corpus: it gives a reader a repository
and checks what the reader reports. Four of the spec's musts are on a
*writer*, and a reading corpus cannot exercise them:

- §3.1: reproduce the line endings the file used, CRLF included;
- §3.2: quote any value that would change meaning when read back (`no`, `12:30`);
- §4.4: keep every frontmatter key it did not change, in the order read;
- §7.1: never reorder the kinds when rewriting `loam.toml`.

v0.0 has no writer. The first commands that write — `new` (0032), `mv` (0034),
supersession (0035) — need round-trip cases: a page in, an edit, and the exact
bytes that must come out. cairn's golden corpus does the same for items.

## Acceptance criteria

- [ ] A round-trip case for each of the four writer musts, run in CI
- [ ] Each case also passes the reading corpus afterwards: a written page reads as intended

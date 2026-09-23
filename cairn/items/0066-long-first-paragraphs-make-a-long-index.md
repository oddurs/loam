---
id: 66
title: Long first paragraphs make a long index
type: feature
status: backlog
milestone: later
created: 2026-09-22
updated: 2026-09-22
priority: p2
effort: s
---

## Problem

Adopting measure-of-the-world (0036): a summary is the whole first paragraph
(spec §5.3), and some first paragraphs run to eighty words. In the index they
make rows that are mostly one cell.

## Proposal

Decide whether the index shows the summary's first sentence, a length it cuts
at, or all of it and leaves the fix to the page (a `summary:` line, spec §11).
The first two are display rules only, and change no reading.

## Acceptance criteria

- [ ] Decided, with a rendered index from each adopted folder beside the rule

---
id: 37
title: A README that says what loam is and is not
type: docs
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 14
- 19
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p2
effort: s
---

## What is missing

A README in cairn's manner: what a page is, why it is not a cairn item, the
boundary, install, a quickstart on a real folder. It should answer "how does this
fit with cairn and harrow" in the first screen, because that is the first thing
anybody who knows cairn will ask.

## Acceptance criteria

- [x] States the tense rule and the boundary
- [x] The quickstart is run for real, not written from memory

## 2026-09-22

README.md states the tense rule and the boundary, and fits loam beside cairn and harrow in its first screen. Its quickstart, and the longer one in docs/guide/adopting-a-docs-folder.md, are output from a real run on a fresh copy of poptop; the run found the summary-link bug fixed under 0030.

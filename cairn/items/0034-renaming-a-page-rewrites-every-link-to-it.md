---
id: 34
title: Renaming a page rewrites every link to it
type: feature
status: backlog
milestone: v0.1
depends_on:
- 16
- 31
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- cli
- links
effort: m
---

## Problem

Pages are addressed by path, so a rename breaks every link to the page — which is
why docs folders keep bad names for ever.

## Proposal

`loam mv OLD NEW` moves the file and rewrites every relative link to it, under
one lock, atomically, and says which files it touched. It is to pages what `cairn
set 1 key=v2.0` is to milestone keys. Moving a directory moves every page in it.

## Costs

Rewriting other files is the riskiest write loam makes. It gets cairn's
treatment: atomic writes, line endings preserved, a dry run.

## Acceptance criteria

- [ ] `--dry-run` lists every change without making one
- [ ] Links with anchors and `../` paths rewritten correctly
- [ ] An interrupted move leaves every file either old or new, never half

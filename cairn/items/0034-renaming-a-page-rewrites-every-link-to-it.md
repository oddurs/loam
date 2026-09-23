---
id: 34
title: Renaming a page rewrites every link to it
type: feature
status: done
milestone: v0.1
depends_on:
- 16
- 31
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] `--dry-run` lists every change without making one
- [x] Links with anchors and `../` paths rewritten correctly
- [x] An interrupted move leaves every file either old or new, never half

## 2026-09-22

mv rewrites links to what moved and links out of it, in every Markdown file git knows about — the repository README and CHANGELOG link into docs more than pages do. Found on a copy of poptop: a link whose text wrapped over two lines was not found, which was a gap in the spec, fixed as 0063. The interrupted-move test makes one file unwritable mid-move and checks every file is either as it was or as it will be. A link that still resolves after the move is left as written, so moving a page and back can leave a path spelled differently from before; the test records it.

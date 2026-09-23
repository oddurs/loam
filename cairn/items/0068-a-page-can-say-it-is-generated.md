---
id: 68
title: A page can say it is generated
type: spec
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
effort: s
---

## Current text

None. A generated page reads like any other.

## Proposed text

Answering 0060 found two generated pages among the three adopted folders:
code-as-color's `outline.md` (from `outline/*.ts`) and measure-of-the-world's
`status.md` (from `make status`). Both are read, so both belong in the index;
neither can carry frontmatter written by hand, because the next build removes
it. And neither should ever be called stale (v0.2): its generator keeps it
true, not a review.

Proposed: an optional key, `generated`, whose value says what generates the
page (`make status`). A generator that writes frontmatter can set it; staleness
leaves such a page alone.

## What a conforming reader has to do differently

Nothing until staleness exists. A new optional key (spec §9).

## Acceptance criteria

- [x] spec/README.md updated
- [x] A corpus case exercises it

## 2026-09-23

Folded into the same spec change as covers and reviewed: generated is a string saying what writes the page, and anything reporting staleness should leave such a page out. hostile/covers-and-reviewed has a case of each shape.

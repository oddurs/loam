---
id: 68
title: A page can say it is generated
type: spec
status: backlog
milestone: v0.2
created: 2026-09-22
updated: 2026-09-22
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

- [ ] spec/README.md updated
- [ ] A corpus case exercises it

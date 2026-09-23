---
id: 40
title: Say which pages the code has moved out from under
type: feature
status: backlog
milestone: v0.2
depends_on:
- 28
- 39
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- freshness
effort: l
---

## Problem

A page that describes code which has since changed reads exactly like one that
does not, and an agent reads it at the start of every session with the same
trust.

## Proposal

`loam stale` asks git, for each page with `covers`, what changed in those paths
since `reviewed.commit`, and lists pages with the commits and the size of the
change:

```
docs/architecture.md   reviewed 41 days ago
  src/engine/**        9 commits, +312 −140   "split the scheduler out"
```

Sorted by how much changed, not by how long ago. `--json` for harrow and CI.
Read history through the git command line, as `cairn log` does, rather than
linking a git library.

## Costs

It is only as good as `covers`. A page with none is reported as "unknown", not
as fresh — silence would be a claim.

## Acceptance criteria

- [ ] The replay from the covers question reproduces through this command
- [ ] Pages without `covers` listed as unknown, separately
- [ ] Fast enough to run in a pre-commit hook on the three repositories

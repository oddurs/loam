---
id: 40
title: Say which pages the code has moved out from under
type: feature
status: done
milestone: v0.2
depends_on:
- 28
- 39
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] The replay from the covers question reproduces through this command
- [x] Pages without `covers` listed as unknown, separately
- [x] Fast enough to run in a pre-commit hook on the three repositories

## 2026-09-23

The replay in 0038 ran through this command: every first-parent commit of poptop and measure-of-the-world, as loam stale --json --at COMMIT. Pages without covers are 'unknown', counted apart and listed with --all. Release build, warm cache: 433 ms on poptop (seven pages with covers, 196 commits), 174 ms on measure-of-the-world, 37 ms on code-as-color — well inside a pre-commit hook. The first version took 2.4 s, all of it one git log diffing every file of every commit; limiting that log to each page's patterns as git pathspecs fixed it. A page never reviewed is compared from the last commit that changed its body, not its frontmatter — found when my own covers commit on measure-of-the-world reset every baseline, because adding frontmatter to a page that had none left a blank line where the page began.

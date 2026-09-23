---
id: 56
title: harrow shows stale pages under Needs you
type: feature
status: backlog
milestone: v0.4
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-22
priority: p2
pillar:
- links
- freshness
effort: s
---

## Problem

harrow's Needs you lens is "everything addressed to a person and nothing else". A
page the code has moved out from under is exactly that.

## Proposal

One row kind in harrow, fed by `loam stale --json` when loam is installed:
"`docs/architecture.md` — covered code changed in 9 commits". Its key opens the
page in `$EDITOR`; another runs `loam review`.

This belongs in harrow's backlog; it is filed here so the `--json` shape is
designed with a consumer in mind. harrow does **not** get a docs browser — pages
are read, not triaged.

## Acceptance criteria

- [ ] An item filed in harrow's backlog, linking here
- [ ] `stale --json` carries everything the row needs

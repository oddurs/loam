---
id: 35
title: A page that supersedes another says so, and so does the other
type: feature
status: done
milestone: v0.1
depends_on:
- 18
- 28
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p2
pillar:
- cli
- index
effort: s
---

## Problem

Agents in particular write `architecture-v2.md` next to `architecture.md`, and
from then on a reader cannot tell which is true.

## Proposal

`loam supersede OLD NEW` sets `supersedes` on the new page and `status:
superseded` with `superseded_by` on the old, which gains a one-line notice at the
top pointing to its replacement. The index moves the old page to its trailing
section. `check` warns about links that still point at the old page.

## Acceptance criteria

- [x] Both pages updated in one write
- [x] The notice is added once, not again on a second run

## 2026-09-22

Both pages are written under one lock, each atomically; a filesystem offers no atomic write of two files, so 'one write' means one command under one lock. The notice goes above the title, not below it, so the old page's title and summary still read as they did.

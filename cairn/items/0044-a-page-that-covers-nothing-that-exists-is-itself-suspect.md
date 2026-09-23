---
id: 44
title: A page that covers nothing that exists is itself suspect
type: feature
status: backlog
milestone: v0.2
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-22
priority: p2
pillar:
- freshness
- links
effort: s
---

## Problem

When the code a page describes is deleted or moved, its globs match nothing, the
diff is empty, and the page reads as fresh — the most wrong it can possibly be.

## Proposal

`check` warns about a `covers` glob that matches no file, naming the glob.

## Acceptance criteria

- [ ] Deleting a covered directory in a test repository produces the warning

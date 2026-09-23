---
id: 44
title: A page that covers nothing that exists is itself suspect
type: feature
status: done
milestone: v0.2
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Deleting a covered directory in a test repository produces the warning

## 2026-09-23

check reports covers-nothing for every including pattern that matches no file git knows about (or, without git, no file on disk), on the covers line. Deleting covered code first makes the page stale, correctly; the warning matters after the next review or edit, when the page reads as fresh while describing nothing, and the test goes through exactly that.

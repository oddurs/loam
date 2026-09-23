---
id: 18
title: A page is draft, current or superseded, and never done
type: decision
status: done
milestone: v0.0
depends_on:
- 13
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
pillar:
- format
effort: s
---

## Context

cairn's status categories are open, active, done and dropped. None of them fits
a page. A page is not finished; it is either trustworthy or not.

## Options

1. **No status.** Every page is current. Simple, and an agent's half-finished
   research looks exactly like a reviewed design.
2. **Three states**: `draft` (written, not yet trusted), `current` (the default,
   and what an absent status means), `superseded` (kept, but replaced).
3. **Configurable statuses** like cairn's, with categories.

## Recommendation

**Option 2, fixed, with `current` as the default.** A page with no frontmatter
is current, so adopting a folder asks nothing of it. The states are few enough
to be a format guarantee rather than configuration, which is what lets a site
built from the manifest grey out a draft without reading anybody's schema.

A superseded page is kept, not deleted. Superseded research is still evidence,
and a link to it should land somewhere that says what replaced it.

## Consequences

- `supersedes` / `superseded_by` are format keys, not configuration.
- Staleness (v0.2) is not a status. A current page can be stale; that is a
  derived fact about it, not something written into it.

## What it rules out

- Project-declared page statuses, and status categories for pages.
- `done`, `archived`, `deprecated`, `stale` as statuses. `unknown-status` is
  reported for any of them (spec §4.1).
- Staleness as something written into a page.
- Deleting a superseded page as the way to supersede it.

## Acceptance criteria

- [x] The decision is stated in one sentence under Recommendation
- [x] What it rules out is written down

## 2026-09-22

Decided as recommended, with one addition in spec §5.5: when status is absent and superseded_by names a page, the status is superseded rather than current, so recording the successor is enough.

---
id: 41
title: review records that a page was read against the code as it is now
type: feature
status: backlog
milestone: v0.2
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- freshness
- cli
effort: s
---

## Problem

Clearing a stale flag by hand means writing a commit hash into YAML. Nobody will.

## Proposal

`loam review PAGE` sets `reviewed` to HEAD and today. It warns, and does not
refuse, when covered paths have uncommitted changes — the review would be
against code that does not exist in any commit yet. `--note TEXT` appends a line
to a trailing `## Review log` section, for pages where why it was still true is
worth keeping.

## Acceptance criteria

- [ ] One command clears a page from `stale`
- [ ] The warning about uncommitted covered changes is tested

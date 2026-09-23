---
id: 47
title: Pages an agent writes start as drafts
type: feature
status: backlog
milestone: v0.3
depends_on:
- 18
- 46
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- agents
effort: s
---

## Problem

Research an agent wrote an hour ago and a design a person reviewed last month
look identical in a docs folder, and are read with identical trust.

## Proposal

`agent_status = "draft"` in the config makes `loam new` from an agent — detected
the way cairn detects one, or declared by a flag — write `status: draft`.
Making it current is a person's edit to the frontmatter. The contract
says so.

Say plainly that this is a guard rail and not a boundary: an agent with a shell
can edit the frontmatter. cairn's manual makes the same admission and is better
for it.

## Acceptance criteria

- [ ] Agent-created pages are drafts; person-created are not
- [ ] The index marks drafts
- [ ] The limitation is documented, in those words

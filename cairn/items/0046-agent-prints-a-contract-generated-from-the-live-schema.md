---
id: 46
title: agent prints a contract generated from the live schema
type: feature
status: backlog
milestone: v0.3
depends_on:
- 32
- 33
- 35
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- agents
effort: m
---

## Problem

Told to "research this in docs", an agent invents a filename and a format. The
next agent invents another. cairn fixed this for items with `cairn agent`, which
cannot describe a workflow the project does not have because it is generated
from the schema.

## Proposal

`loam agent` prints, and `--write AGENTS.md` inserts or updates, a block that
says:

- what kinds exist, what question each answers, and where each lives
- **search before writing**: `loam search`, then update an existing page rather
  than start another
- use `loam new KIND "Title"`, never a hand-made path
- replacing a page is `loam supersede`, never `-v2.md`
- after changing covered code, run `loam stale` and update or `review` what it
  names
- how pages and cairn items divide: the tense rule, in two sentences

## Acceptance criteria

- [ ] Generated from the config; changing a kind changes the block
- [ ] `--write` is idempotent and leaves the rest of AGENTS.md alone
- [ ] A real agent session, given only this block, files research correctly

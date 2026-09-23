---
id: 46
title: agent prints a contract generated from the live schema
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 32
- 33
- 35
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Generated from the config; changing a kind changes the block
- [x] `--write` is idempotent and leaves the rest of AGENTS.md alone
- [x] A real agent session, given only this block, files research correctly

## 2026-09-23

Generated from loam.toml: kinds, their directories and descriptions, age limits, whether agents write drafts, and the cairn paragraph only when links.cairn is set; tests change a kind and see the block change. --write replaces the block in place between <!-- loam:begin --> and <!-- loam:end -->, appends it otherwise, and reports 'already current' when nothing changed. Before any agent saw it, reading poptop's block found two faults, fixed: a multi-line description broke its list item, and 'sources' had no stated shape. The real sessions, 2026-09-23, in a scratch clone of poptop with a research kind added and the block in AGENTS.md, each a fresh general-purpose agent told only to read AGENTS.md and do a research task. Session 1 (per-process CPU% against top and htop): loam new research, so the page landed in docs/research/ as a draft, with seven sources in the stated shape each dated, covers set with loam set on the four files it describes, a summary, the index re-rendered by the hook; and, unasked, the two suspected bugs it found filed as cairn items rather than written into the page — the tense rule, followed from the block's two sentences. Session 2, a fresh agent with an overlapping report (400% in poptop, 50% in top on 8 cores): loam search found session 1's page, it checked the page against the code, and added a 31-line section to it. No second page. loam check passed after each.

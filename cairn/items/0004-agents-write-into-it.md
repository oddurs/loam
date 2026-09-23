---
id: 4
key: v0.3
title: Agents write into it
type: milestone
status: backlog
depends_on:
- 3
created: 2026-09-22
updated: 2026-09-22
priority: p1
---

An agent told to research something finds what is already written before writing it again, and writes what it finds where it belongs.

## The problem this answers

Asked to research, an agent writes `docs/research-notes.md`. Asked again next
week, it writes `docs/research/notes-2.md`, having never read the first. Asked
to plan, it writes `PLAN.md` beside the three plans already there. cairn solved
the same thing for work items by handing agents a contract generated from the
schema; this milestone does it for pages, and adds the thing agents most need
that cairn never did: being told which pages matter to the file they are about
to change.

## Acceptance criteria

- [ ] An agent given only `loam agent` output files research in the right place, with the right frontmatter, in a real session
- [ ] `loam context PATH` returns the relevant pages within a stated budget

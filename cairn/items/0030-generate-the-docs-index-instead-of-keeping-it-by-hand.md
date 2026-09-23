---
id: 30
title: Generate the docs index instead of keeping it by hand
type: feature
status: backlog
milestone: v0.1
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- index
effort: m
---

## Problem

poptop's `docs/README.md` and code-as-color's both keep a table of every page,
by hand. Each is correct today and each will drift — the same rot `cairn render`
removed from `ROADMAP.md`.

## Proposal

`loam render` writes the index: pages grouped by kind in declared order, each
with its title and summary, drafts marked, superseded pages in a trailing
section. poptop's index has prose above each table saying what the kind is for;
that comes from the kind's description, and anything else hand-written is
spliced in from a header and footer, as cairn does.

`render --check` exits non-zero when the committed file differs, for CI.
A hook in the config keeps it current, as cairn's does.

## Costs

The hand-written tables carry a short note per page that a summary field must
now hold. Adopting means moving those notes into the pages — the right place for
them, but work.

## Acceptance criteria

- [ ] poptop's rendered index carries everything its hand-kept one does
- [ ] `render --check` fails on drift and passes after rendering
- [ ] Rendering twice changes nothing

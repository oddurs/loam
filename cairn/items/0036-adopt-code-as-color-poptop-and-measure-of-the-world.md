---
id: 36
title: Adopt code-as-color, poptop and measure-of-the-world
type: chore
status: backlog
milestone: v0.1
depends_on:
- 29
- 30
- 31
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- cli
- index
effort: m
---

## What

The three folders are the requirements. Adopt each on a branch, render its
index, fix what `check` finds, and file every friction here as its own item —
the command that was missing, the inference that was wrong, the message that did
not say what to do.

Do this before anything in v0.2. A staleness check built on a format nobody has
used for a week is built on sand.

## Acceptance criteria

- [ ] Each of the three checks clean on a branch
- [ ] Each rendered index replaces the hand-kept one
- [ ] Every friction filed as an item

---
id: 36
title: Adopt code-as-color, poptop and measure-of-the-world
type: chore
status: done
milestone: v0.1
depends_on:
- 29
- 30
- 31
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] Each of the three checks clean on a branch
- [x] Each rendered index replaces the hand-kept one
- [x] Every friction filed as an item

## 2026-09-22

All three adopted on a branch named loam in each repository, as local worktrees, not pushed: poptop f30431e, code-as-color df0dde4, measure-of-the-world bf19ac9. Each passes loam check --strict --render. poptop's and code-as-color's hand-kept indexes are replaced by rendered ones; measure-of-the-world had none and now has one. Frictions: fixed in v0.1 — link text that wraps (0063), a curated order (0064), inferred kind order and the root kind's heading (0029), links broken only by case (0031), a summary's links in the index (0030), a hook depending on PATH (hooks now run the running loam); filed — 0065 labels, 0066 long summaries, 0067 a set command, 0068 generated pages (0060's answer).

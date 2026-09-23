---
id: 2
key: v0.1
title: Useful to me
type: milestone
status: backlog
depends_on:
- 1
created: 2026-09-22
updated: 2026-09-22
priority: p0
---

loam replaces the hand-kept table in a `docs/README.md` and catches the link that broke last week — in three real repositories, not a demo.

## Why this is the first thing worth shipping

code-as-color and poptop both keep a table of their pages by hand, and both
tables will drift, exactly as a hand-edited `ROADMAP.md` did before `cairn
render`. That is the smallest honest claim loam can make, and it is enough to
use every day.

## What it claims

- `init` adopts an existing folder without rewriting a single page
- `render` generates the index and `render --check` fails CI when it drifts
- `check` reports broken links and malformed pages with a file and line

## Acceptance criteria

- [ ] code-as-color, poptop and measure-of-the-world each adopted and checking clean
- [ ] Every friction met while adopting them is filed as an item here

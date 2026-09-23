---
id: 29
title: init adopts a docs folder that already exists
type: feature
status: backlog
milestone: v0.1
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- cli
effort: m
---

## Problem

Every repository this is for already has a docs folder. A tool that starts from
an empty one is a tool for new projects, and there are fewer of those.

## Proposal

`loam init` finds `docs/` or `doc/`, proposes kinds from its subdirectories (or
a preset when it is flat), writes the config, and **touches no page**. It then
prints what it inferred for each page and what it could not:

```
docs/guide/first-run.md     guide     "First run"
docs/concept.md             page      "Concept"   (no directory → default kind)
docs/decisions/0002-...     ?         no kind claims docs/decisions/
```

`--preset` picks one of the presets from the kinds decision.

## Costs

Guessing kinds from directory names will be wrong sometimes. It prints its
guesses rather than hiding them, and the config is one file to correct.

## Acceptance criteria

- [ ] Run on the three real folders: no page changed, config plausible, every guess shown
- [ ] Refuses to overwrite an existing config without `--force`

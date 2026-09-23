---
id: 45
title: Staleness in CI without failing every build
type: feature
status: backlog
milestone: v0.2
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- freshness
effort: s
---

## Problem

A stale page is not a broken build. A CI step that fails on it gets turned off
within the week, and then it is worth nothing (the argument cairn makes for
`require_criteria` being off by default).

## Proposal

`check --stale` reports stale pages as warnings in `file:line` form, so CI and
editors annotate them, and exits zero. `--strict` exists for a project that
wants the gate. A stale page touched in the same diff as its covered code is not
reported — the author is plausibly updating it.

## Acceptance criteria

- [ ] Warnings annotate in GitHub Actions
- [ ] A diff touching both code and page does not warn about that page

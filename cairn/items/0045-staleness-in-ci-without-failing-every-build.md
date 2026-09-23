---
id: 45
title: Staleness in CI without failing every build
type: feature
status: done
milestone: v0.2
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Warnings annotate in GitHub Actions
- [x] A diff touching both code and page does not warn about that page

## 2026-09-23

check --stale reports stale pages as warnings, exit 0, and --strict is the gate. In GitHub Actions every finding is also printed as a workflow command, ::warning file=…,line=…,title=loam CODE::message, with GitHub's escaping; the test checks the exact line. A page is left out when it changed after the first change to its code, or is changed in the working tree: its author is plausibly updating it. loam's own make check now runs check --stale.

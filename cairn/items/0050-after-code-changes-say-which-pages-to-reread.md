---
id: 50
title: After code changes, say which pages to reread
type: feature
status: backlog
milestone: v0.3
depends_on:
- 40
- 49
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- agents
- freshness
effort: s
---

## Problem

`stale` answers about history. An agent that has just edited three files wants
to know, before it stops, which pages its own edit has just made wrong.

## Proposal

`loam stale --working-tree` (uncommitted changes) and `--since REV`. Suitable for
an agent's stop hook: "you changed `src/engine/**`; `docs/design/scheduling.md`
covers it — update it or run `loam review`."

## Acceptance criteria

- [ ] Both flags tested against a scratch repository
- [ ] A cookbook recipe for a stop hook

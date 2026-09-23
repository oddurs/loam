---
id: 50
title: After code changes, say which pages to reread
type: feature
status: done
milestone: v0.3
depends_on:
- 40
- 49
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Both flags tested against a scratch repository
- [x] A cookbook recipe for a stop hook

## 2026-09-23

stale --working-tree: pages covering uncommitted changes; --since REV: covering what changed since REV; together, both. Pages changed alongside are 'updating', not to reread; exit 1 when there is something to reread. The stop-hook recipe, docs/cookbook/loam-stop.sh, blocks once with the list and lets Claude stop when stop_hook_active; a page updated or reviewed counts as changed alongside, so the next stop goes through. Tested with the documented Stop input: block, nothing when already active, nothing when clean. Used on loam itself during this milestone: it named four of loam's own pages, three updated and one reviewed.

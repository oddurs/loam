---
id: 42
title: A squash or rebase must not make every page stale
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
effort: m
---

## Problem

A reviewed commit on a branch that is later squash-merged no longer exists on
`main`. Compare from a commit that is not there and either nothing works or
everything is stale.

## Proposal

When `reviewed.commit` is not an ancestor of HEAD, fall back to `reviewed.date`:
compare from the last commit on the current branch at or before that date, and
say once that it did. `review` on the next run writes a commit that exists.

## Acceptance criteria

- [ ] A test repository squash-merges a branch whose pages were reviewed on it, and `stale` reports only real changes
- [ ] The fallback is announced, once per run

---
id: 3
key: v0.2
title: Knows when it is wrong
type: milestone
status: backlog
depends_on:
- 2
created: 2026-09-22
updated: 2026-09-22
priority: p0
---

A page can say what code it describes, and loam can say when that code has moved on without it.

## Why this is the milestone that justifies the project

An index is convenient. A stale `architecture.md` is worse than none, because it
is read with the authority of something written down — by a person once, and by
an agent every session. Nothing in ordinary docs tooling knows that a page and
the code under it have diverged. git does. This milestone asks it.

## What it claims

`loam stale` names the pages whose covered code changed since they were last
reviewed, with the commits that changed it, and `loam review` records that a
page has been read against the code as it is now.

## Acceptance criteria

- [ ] Replaying three months of a real repository's history flags pages a person agrees were stale, and few that were not
- [ ] A squash-merged branch does not make every page stale

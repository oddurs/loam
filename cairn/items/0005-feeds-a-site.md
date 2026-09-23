---
id: 5
key: v0.4
title: Feeds a site
type: milestone
status: backlog
depends_on:
- 4
created: 2026-09-22
updated: 2026-09-22
priority: p2
---

The docs folder is an indexable source: a site, a search box or another program can be built from it without reading loam's code or parsing its pages twice.

## What it claims

`loam index --json` emits the whole tree — kinds, titles, summaries, status,
links in both directions, freshness — under a documented, versioned schema. A
recipe builds a site from it. References to cairn items resolve, by asking
cairn.

## What it does not claim

loam does not build the site. See the boundary decision.

## Acceptance criteria

- [ ] A site is built from the manifest by the cookbook recipe, with no page parsed outside loam
- [ ] cairn references resolve and are checked

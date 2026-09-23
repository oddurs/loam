---
id: 55
title: 'A recipe: a site built from the manifest'
type: docs
status: backlog
milestone: v0.4
depends_on:
- 52
created: 2026-09-22
updated: 2026-09-22
priority: p2
pillar:
- index
effort: m
---

## What is missing

Proof the manifest is enough. cairn's `www/sync.mjs` already builds an Astro
site by copying the spec in; this recipe goes further — navigation, kinds as
sections, drafts marked, stale pages bannered, backlinks — using only the
manifest and the page bodies.

It is a recipe in the cookbook, not a feature. See the boundary.

## Acceptance criteria

- [ ] Built from one of the adopted repositories
- [ ] Reads nothing but the manifest and page bodies

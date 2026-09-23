---
id: 52
title: 'index --json: the whole docs tree as one document'
type: feature
status: backlog
milestone: v0.4
depends_on:
- 30
- 51
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- index
effort: m
---

## Problem

"Indexable source you could build a docs site from" needs one thing to build it
from. Today a site would have to walk the folder and parse every page's
frontmatter itself, re-implementing inference and getting it subtly different.

## Proposal

`loam index --json` emits the manifest. `render` and `index` share one code path,
so the Markdown index and the JSON can never disagree.

## Acceptance criteria

- [ ] Output validates against the published schema
- [ ] The Markdown index is provably derived from the same data

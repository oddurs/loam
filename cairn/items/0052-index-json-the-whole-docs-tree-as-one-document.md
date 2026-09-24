---
id: 52
title: 'index --json: the whole docs tree as one document'
type: feature
status: done
milestone: v0.4
depends_on:
- 30
- 51
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Output validates against the published schema
- [x] The Markdown index is provably derived from the same data

## 2026-09-23

loam index prints the Markdown block render writes; loam index --json prints the manifest (--no-history leaves out freshness). Both come from index::sections(), so they cannot disagree; manifest_check.py reads the Markdown back and compares it with sections on every folder. Found on the way: a title holding a link, or badges, nested a link inside the index row's link, which Markdown reads as the inner one, so the row did not reach its page (105 rows in the stress set); titles in rows now lose their links and images (scan::unlinked).

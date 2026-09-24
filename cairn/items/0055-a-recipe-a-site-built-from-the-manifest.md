---
id: 55
title: 'A recipe: a site built from the manifest'
type: docs
status: done
milestone: v0.4
assignee: Oddur Sigurdsson
depends_on:
- 52
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Built from one of the adopted repositories
- [x] Reads nothing but the manifest and page bodies

## 2026-09-23

docs/cookbook/site.py (~160 lines, Python Markdown) builds a static site from the manifest and page bodies only: nav from sections, draft marks, stale banners, TOC from headings with the manifest's slugs as ids, backlinks footer, page links rewritten to HTML. Built from poptop: 36 pages, 1,559 internal links and anchors, none broken. manifest_check.py builds the site for every folder it checks, so the recipe cannot rot. Guide: docs/guide/building-a-site-from-the-manifest.md (a draft, as an agent wrote it).

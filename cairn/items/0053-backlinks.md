---
id: 53
title: Backlinks
type: feature
status: done
milestone: v0.4
assignee: Oddur Sigurdsson
depends_on:
- 31
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
pillar:
- links
- index
effort: s
---

## Problem

"What links here?" is the question that tells you whether a page is safe to
delete or change, and a folder of files cannot answer it without a scan.

## Proposal

Computed on read, like cairn's `inverse` fields. In `show`, in the manifest, and
optionally rendered as a "Linked from" footer on each page by `render --pages`.

## Acceptance criteria

- [x] `show` lists backlinks
- [x] Present in the manifest

## 2026-09-23

Backlinks are computed on read (Tree::backlinks_all), from page links only: the index is left out, since it links every page, and so is a page's link to itself. show lists them ('linked from'), show --json has backlinks, and the manifest has them per page. render --pages (a 'Linked from' footer written into each page) was not built: it would write into every page, which loam avoids; the cookbook site shows the footer from the manifest instead.

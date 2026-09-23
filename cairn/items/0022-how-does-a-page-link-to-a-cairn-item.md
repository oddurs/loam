---
id: 22
title: How does a page link to a cairn item?
type: question
status: backlog
milestone: v0.0
depends_on:
- 14
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- links
effort: s
---

## Question

What does a reference from a page to a cairn item look like in the file, so that
it renders on GitHub, survives the item being retitled, and can be checked?

## Why it has to be answered first

cairn renames an item's file when its title changes: `0072-relationships-...md`
becomes something else. A plain relative link rots the first time a title is
improved, which on this author's backlogs is often.

## Options

1. **A relative link to the file.** Renders everywhere. Rots on retitle.
2. **A scheme**, `[0072](cairn:72)`. Survives retitling. GitHub renders it as a
   dead link.
3. **A relative link whose target loam repairs**: the filename's numeric prefix
   is the identity, so `check --fix` rewrites
   `../cairn/items/0072-old-title.md` to the current filename. Renders
   everywhere, survives retitling for anyone who runs check.
4. **Ask cairn to keep the link stable** — a redirect file, or no slug in the
   filename. A change to cairn for loam's benefit; a poor trade.

## What would settle it

Write a page with each form, push it, and look at it on GitHub and in an Astro
build. Then retitle the item and see what each does.

## Answer

<!-- Filled in when the spike closes. -->

## Acceptance criteria

- [ ] Answer written, with the evidence that settled it

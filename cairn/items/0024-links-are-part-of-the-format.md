---
id: 24
title: Links are part of the format
type: spec
status: backlog
milestone: v0.0
depends_on:
- 22
- 23
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- format
- links
effort: s
---

## Current text

The draft spec says nothing about links.

## Proposed text

A section saying which links a reader resolves and what counts as broken:

- relative links to other pages, with or without `#anchor`
- anchors, resolved against headings the way GitHub slugs them — the one
  slugging rule everybody already sees
- references to cairn items, in whatever form the cairn-link question settles
- absolute URLs: never resolved, never checked (no network, per the boundary)

## What a conforming reader has to do differently

Know the GitHub heading-slug rule, and treat a link to a missing page or anchor
as a finding rather than an error that stops the read.

## Acceptance criteria

- [ ] spec/README.md updated
- [ ] A corpus case exercises it: broken page, broken anchor, anchor with Unicode, cairn reference

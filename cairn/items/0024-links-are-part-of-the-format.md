---
id: 24
title: Links are part of the format
type: spec
status: done
milestone: v0.0
depends_on:
- 22
- 23
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] spec/README.md updated
- [x] A corpus case exercises it: broken page, broken anchor, anchor with Unicode, cairn reference

## 2026-09-22

Written as spec §6: finding links (§6.1), resolving them (§6.2), anchors by GitHub's slug rule (§6.3), cairn references (§6.4), and the findings (§6.5). The slug rule was checked against github-slugger 2.0.0, the library GitHub's own rule is published as, on 21 headings: 20 agree, and the one that does not — an emoji shortcode — is written into §6.3 as a known limit. github-slugger keeps exactly Unicode TS #18's word characters, which is how §6.3 states it. Corpus: hostile/links (broken page, broken anchor, directories, case), hostile/anchors (Unicode, combining marks, duplicates), hostile/cairn.

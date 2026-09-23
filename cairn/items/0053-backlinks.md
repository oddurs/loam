---
id: 53
title: Backlinks
type: feature
status: backlog
milestone: v0.4
depends_on:
- 31
created: 2026-09-22
updated: 2026-09-22
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

- [ ] `show` lists backlinks
- [ ] Present in the manifest

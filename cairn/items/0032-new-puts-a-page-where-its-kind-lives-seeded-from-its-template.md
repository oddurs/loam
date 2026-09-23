---
id: 32
title: new puts a page where its kind lives, seeded from its template
type: feature
status: backlog
milestone: v0.1
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- cli
effort: s
---

## Problem

A person or an agent writing research has to know where research goes and what
headings it has. Today that is folklore, and agents do not have folklore.

## Proposal

`loam new research "Markdown rendering in a forty-column pane"` writes
`docs/research/markdown-rendering-in-a-forty-column-pane.md` with the kind's
template and frontmatter, and prints the path. Slugs follow cairn's filename
rules. It never overwrites.

## Acceptance criteria

- [ ] Path, template and frontmatter correct for every kind of a preset
- [ ] Refuses an existing path and says which page is there

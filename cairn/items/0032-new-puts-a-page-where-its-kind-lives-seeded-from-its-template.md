---
id: 32
title: new puts a page where its kind lives, seeded from its template
type: feature
status: done
milestone: v0.1
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] Path, template and frontmatter correct for every kind of a preset
- [x] Refuses an existing path and says which page is there

## 2026-09-22

tests/cli.rs new_puts_each_kind_of_every_preset_where_it_lives covers every kind of all three presets: the path, the template (research's headings), and that the page reads back as the kind it was made as. Refusing names the page already there, and also refuses a name that differs only in case, which would overwrite on a Mac.

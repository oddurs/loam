---
id: 21
title: Does a page need frontmatter at all?
type: question
status: backlog
milestone: v0.0
depends_on:
- 17
- 18
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- format
- cli
effort: s
---

## Question

Must every page carry YAML frontmatter, or can loam read a plain Markdown file
and infer what it needs?

## Why it has to be answered first

It decides what adopting a folder costs. poptop's pages have no frontmatter at
all. code-as-color's carry status as a bold line in the body. If frontmatter is
required, adoption is a rewrite of every page — and a tool that asks for that on
day one is not adopted.

## Options

1. **Required.** Simplest reader, and a wall in front of every existing folder.
2. **Optional, with inference.** `title` from the first H1, `kind` from the
   directory, `status` defaults to current, `summary` from the first paragraph.
   Frontmatter overrides any of them.
3. **Never.** Everything inferred. Nowhere to put `covers` or `supersedes`.

## What would settle it

Run option 2's inference by hand over every page in the three real folders and
count how many come out right. If nearly all do, inference is the format; if
many need correcting, frontmatter is.

## Answer

<!-- Filled in when the spike closes. -->

## Acceptance criteria

- [ ] Answer written, with the evidence that settled it
- [ ] The count, per folder, of pages inference got right

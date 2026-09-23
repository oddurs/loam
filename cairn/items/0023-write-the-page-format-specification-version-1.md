---
id: 23
title: Write the page format specification, version 1
type: spec
status: backlog
milestone: v0.0
depends_on:
- 15
- 16
- 17
- 18
- 21
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- format
effort: m
---

## Current text

None.

## Proposed text

`spec/README.md`, normative, standing alone, in RFC 2119 language — modelled on
cairn's, and sharing its §3 (structure of a file: delimiters, BOM, LF and CRLF,
the body ascribes no meaning) word for word where the rules are the same. Two
specifications that describe the same frontmatter differently would be a bug.

Sections:

1. Why a specification
2. Terminology — page, kind, docs root, project
3. Structure of a page — shared with cairn
4. Keys — `title`, `kind`, `status`, `summary`, `supersedes`, and room for
   `covers` / `reviewed` (v0.2) without a format bump
5. Inference — what a reader may derive when a key is absent, per the
   frontmatter question
6. Links — see its own spec item
7. Unknown keys — preserved on rewrite, never an error
8. Conformance

## What a conforming reader has to do differently

Everything; this is the first version.

## Acceptance criteria

- [ ] spec/README.md written, with an example page
- [ ] A reader can be implemented from it without consulting loam
- [ ] The shared structure section is identical to cairn's, and a test says so
- [ ] A corpus case exercises every **must**

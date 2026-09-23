---
id: 51
title: The manifest is a documented, versioned output
type: spec
status: backlog
milestone: v0.4
depends_on:
- 23
- 24
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- index
- format
effort: s
---

## Current text

None.

## Proposed text

A section, or a sibling document, specifying `loam index --json`: its top-level
shape, every field, which are always present, and a `manifest_version`. Additive
changes are free; anything else bumps it. The same promise cairn makes for its
`--json` output.

Per page: path, kind, title, summary, status, supersedes/superseded_by, headings
(for a site's table of contents), outgoing links, backlinks, cairn references,
covers, freshness.

## What a conforming reader has to do differently

A site built from the manifest reads nothing else except page bodies.

## Acceptance criteria

- [ ] Specified, with an example
- [ ] A JSON Schema file published beside it, and tests validate output against it

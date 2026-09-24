---
id: 51
title: The manifest is a documented, versioned output
type: spec
status: done
milestone: v0.4
assignee: Oddur Sigurdsson
depends_on:
- 23
- 24
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Specified, with an example
- [x] A JSON Schema file published beside it, and tests validate output against it

## 2026-09-23

spec/manifest.md and spec/manifest.schema.json (draft 2020-12). Each page entry is its reading exactly as the corpus shapes it, plus path, body_line, lead, headings (with GitHub slugs), backlinks, items and freshness; the top has kinds, sections (the index as data), and freshness notes/error. manifest_version 1, additive changes free. spec/manifest_check.py validates loam's output on this repository and the three real corpus folders in make check and CI; it also passed on uv (81 pages) and the 7,357-page stress set.

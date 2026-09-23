---
id: 28
title: Read a folder of pages into one model
type: feature
status: backlog
milestone: v0.1
depends_on:
- 23
- 26
- 27
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- cli
- format
effort: m
---

## Problem

Every command starts from the same thing: the config, the kinds, and every page
under the docs root parsed into one model with its inferred fields filled in.

## Proposal

One reader, driven by the corpus from the first commit. Errors are per file,
with a path and line, and one malformed page never stops the rest being read —
cairn's `0020` found out what happens otherwise.

Unknown frontmatter keys are kept, and a rewrite puts them back where they were.

## Costs

The inference rules are the most likely thing to be wrong, and they are
specified. Getting them wrong costs a format change, which is why the corpus
comes first.

## Acceptance criteria

- [ ] Every corpus page reads to its expected result
- [ ] A broken page is reported and every other page still reads
- [ ] Round-trip: reading and writing an untouched page changes no byte

---
id: 28
title: Read a folder of pages into one model
type: feature
status: done
milestone: v0.1
depends_on:
- 23
- 26
- 27
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] Every corpus page reads to its expected result
- [x] A broken page is reported and every other page still reads
- [x] Round-trip: reading and writing an untouched page changes no byte

## 2026-09-22

tree.rs reads every page into one model; tests/conformance.rs holds the binary to every corpus case, and make conformance runs spec/conformance.py against 'loam reading' as well. YAML is taken as parser events and resolved by the 1.2 core schema in yaml.rs, because no Rust YAML library resolves exactly that; writing it found the reference reader's own 1.1 slip (012 as octal ten), now fixed and in the corpus. Round-trip: no reading command writes (tests/cli.rs reading_changes_no_byte), and every writer edits frontmatter as text, so an untouched line is never re-emitted.

---
id: 69
title: Three agents try to break loam, and what they found is fixed
type: bug
status: doing
milestone: v0.4
created: 2026-09-23
updated: 2026-09-23
priority: p2
---

## What happens

Three agents were each asked to break one part of loam: the reader (what a page
says), the write path (what the writing commands do to a file), and freshness
(when a page is stale, and how fast loam decides). Each reported a dozen
reproduced failures. The worst: two concurrent writes lost one, `agent --write`
erased a file it could not decode, `loam` panicked on `[x]: <é`, a 248-byte
page took 620 MB through YAML aliases, and GitHub footnotes were reported as
broken links.

## What should happen

Each failure is reproduced, fixed, and kept fixed by a test; where the reader is
at fault, the spec says what is right and both readers agree on a corpus case.

## Reproduction

1. The reviewers' reports are summarised in the notes below and in the commit
   messages that fixed them.

## Acceptance criteria

- [x] Every write-path finding has a test in tests/writes.rs or write.rs, and passes
- [x] Every reader finding has a corpus case both readers agree on
- [ ] Every freshness finding is fixed or recorded as its own item
- [ ] The stress corpus and the fuzzer show no disagreement on well-formed input

## 2026-09-23

Reader findings fixed together. Spec changes: footnote labels are not definitions (§6.1); an unclosed < makes the plain form; code spans are found per paragraph or heading, not per line; frontmatter YAML ends in a line break, so a block scalar written last keeps its newline (§3.1); verbatim core tags are honoured, and !!int/!!float/!!bool values must be what the core schema would resolve (§3.2); three limits — 64-bit integers, 100 levels of nesting, 10,000 values with aliases expanded — make frontmatter malformed (§3.2); ./ and . segments are ignored in loam.toml paths (§7.1); cairn numbers are ASCII digits below 2^64 (§6.4); in heading text, inline links go before reference links (§6.3). Python only: underscore runs, uncaught ValueError and RecursionError, format = true / 1.0 accepted, and PyYAML's refusal of tabs in plain scalars (overridden, since 1.2 allows them). 14 new hostile cases.

## 2026-09-23

Left alone: on YAML that is malformed anyway, yaml-rust2 and PyYAML sometimes disagree on whether it is 'not YAML' or 'not a mapping' (a tab before a comment, content after a comment inside a plain scalar). Both readers report malformed-frontmatter and read no frontmatter; only the detail differs. Checked that realistic versions of each agree.

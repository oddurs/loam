---
id: 69
title: Three agents try to break loam, and what they found is fixed
type: bug
status: done
milestone: v0.4
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
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
- [x] Every freshness finding is fixed or recorded as its own item
- [x] The stress corpus and the fuzzer show no disagreement on well-formed input

## 2026-09-23

Reader findings fixed together. Spec changes: footnote labels are not definitions (§6.1); an unclosed < makes the plain form; code spans are found per paragraph or heading, not per line; frontmatter YAML ends in a line break, so a block scalar written last keeps its newline (§3.1); verbatim core tags are honoured, and !!int/!!float/!!bool values must be what the core schema would resolve (§3.2); three limits — 64-bit integers, 100 levels of nesting, 10,000 values with aliases expanded — make frontmatter malformed (§3.2); ./ and . segments are ignored in loam.toml paths (§7.1); cairn numbers are ASCII digits below 2^64 (§6.4); in heading text, inline links go before reference links (§6.3). Python only: underscore runs, uncaught ValueError and RecursionError, format = true / 1.0 accepted, and PyYAML's refusal of tabs in plain scalars (overridden, since 1.2 allows them). 14 new hostile cases.

## 2026-09-23

Left alone: on YAML that is malformed anyway, yaml-rust2 and PyYAML sometimes disagree on whether it is 'not YAML' or 'not a mapping' (a tab before a comment, content after a comment inside a plain scalar). Both readers report malformed-frontmatter and read no frontmatter; only the detail differs. Checked that realistic versions of each agree.

## 2026-09-23

Freshness findings, all fixed but one left as designed: git.rs now reads history with -z and --relative (quoted paths; a project in a subdirectory of its repository), counts a merge by its remerge diff (what it changed beyond merging), and diffs --since from the merge base. fresh.rs judges every page from one log over the union of their pathspecs, from the octopus merge base of their baselines, and assigns commits per page with an in-memory ancestry walk of rev-list --parents; page history is one log of the docs dir with -M --raw, followed through renames, and bodies come from one cat-file --batch. A page's own file never counts against it. 'Being updated' now needs an edit in or after the newest covered change. show and context judge only the pages they print and say when history could not be read. context counts its header, keeps back room for the left-out note only when something is left out, and refuses budgets under 200. An empty repository is nothing stale, with a note; --at before a review sets it aside, with a note. Timings on the reviewer's repositories: stale 5.9s -> 0.26s (5000 commits, 60 pages); show 49s -> 0.21s (200 pages); cairn 6.9s -> 0.43s. Same verdicts on all three before and after.

## 2026-09-23

Left as designed: 'context docs/q.md' does not list q.md itself. The caller has the page open; context is the pages around a file, and a page covering docs/ still appears.

## 2026-09-23

Found while testing the fixes: the write lock sat in the docs root, so review (which asks git for uncommitted files while holding it) saw .loam.lock as an uncommitted change, and a user's git status could show it mid-command. It now lives in the git directory, falling back to the docs root only without git.

## 2026-09-23

After both rounds: the 7,357-page stress corpus reads identically in both readers; CI green on ubuntu and macOS at d32af3a.

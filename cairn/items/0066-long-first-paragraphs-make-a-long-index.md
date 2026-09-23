---
id: 66
title: Long first paragraphs make a long index
type: feature
status: done
milestone: later
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
effort: s
---

## Problem

Adopting measure-of-the-world (0036): a summary is the whole first paragraph
(spec §5.3), and some first paragraphs run to eighty words. In the index they
make rows that are mostly one cell.

## Proposal

Decide whether the index shows the summary's first sentence, a length it cuts
at, or all of it and leaves the fix to the page (a `summary:` line, spec §11).
The first two are display rules only, and change no reading.

## Acceptance criteria

- [x] Decided, with a rendered index from each adopted folder beside the rule

## 2026-09-23

Decided: a summary from frontmatter is shown whole; one inferred from a first paragraph is cut to its whole sentences up to 160 characters, at least one. A display rule in the index only; no reading changes. Tried first as the first sentence alone, rendered over the three adopted folders: it lost measure-of-the-world's Contributing row, whose first sentence is a thank-you and whose second says what the page is. With the 160-character rule, measure-of-the-world's index changed three rows (Production Engine, Print Edition Checklist, Style Guide), each now one complete sentence; code-as-color's Outline row lost its second and third sentences; poptop's rows, all from frontmatter, did not change. A sentence ends at . ? or ! before a space, not after e.g., i.e., etc. and the like, and not inside a code span, link or parentheses.

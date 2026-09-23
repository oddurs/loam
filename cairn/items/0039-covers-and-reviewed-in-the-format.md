---
id: 39
title: covers and reviewed in the format
type: spec
status: done
milestone: v0.2
depends_on:
- 25
- 38
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p0
pillar:
- freshness
- format
effort: s
---

## Current text

§4 reserves room for `covers` and `reviewed` and says nothing about them.

## Proposed text

- `covers`: a list of globs relative to the repository root, in the form the
  covers question settles.
- `reviewed`: a mapping with `commit` and `date`. Both, because a commit can
  stop existing (a squash, a rebase) and a date cannot.

Optional keys, so no format bump — which is why room was left.

## What a conforming reader has to do differently

Nothing, if it does not compute freshness. The keys are preserved like any other.

## Acceptance criteria

- [x] spec/README.md updated
- [x] A corpus case exercises it

## 2026-09-23

spec §4.3 as proposed, with the pattern rules written out: paths from the repository root, ** across segments, * and ? within one, a directory covering what is below it, and ! taking files back out (quoted in YAML). reviewed is a mapping of commit (7 to 64 hex characters) and date (YYYY-MM-DD), both required, other keys preserved. A commit written unquoted as all digits is a number in YAML 1.2 and so malformed-key; loam's writer quotes it. hostile/covers-and-reviewed pins all of it, and both readers agree.

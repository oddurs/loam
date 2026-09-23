---
id: 39
title: covers and reviewed in the format
type: spec
status: backlog
milestone: v0.2
depends_on:
- 25
- 38
created: 2026-09-22
updated: 2026-09-22
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

- [ ] spec/README.md updated
- [ ] A corpus case exercises it

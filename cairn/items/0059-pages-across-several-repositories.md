---
id: 59
title: Pages across several repositories
type: decision
status: dropped
milestone: later
depends_on:
- 19
created: 2026-09-22
updated: 2026-09-22
priority: p3
pillar:
- index
effort: s
---

## Context

The obvious next wish after a docs index: one across every repository in
`~/Code`.

## Recommendation

**No.** A repository cannot see another one, and the boundary decision says loam
does what one repository can. Something else can read every repository's
manifest; that is what the manifest is for.

Filed and dropped at once so the answer is findable when the wish comes back.

## Consequences

The manifest has to be good enough that "something else" is an afternoon.

## Acceptance criteria

- [x] The decision is stated in one sentence under Recommendation
- [x] What it rules out is written down

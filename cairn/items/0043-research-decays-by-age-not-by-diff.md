---
id: 43
title: Research decays by age, not by diff
type: feature
status: backlog
milestone: v0.2
depends_on:
- 39
created: 2026-09-22
updated: 2026-09-22
priority: p2
pillar:
- freshness
effort: s
---

## Problem

A research page on "which Markdown crates exist" covers no code in the
repository. It goes stale because the world moved, not the code.

## Proposal

A kind may declare `stale_after = "180d"`. A page of that kind is stale when its
`reviewed.date` is older, independently of `covers`. Research pages may carry
`sources` with the date each was read, and `show` lists them.

## Acceptance criteria

- [ ] A research page older than the policy is reported, with the reason
- [ ] A kind without the policy is unaffected

---
id: 43
title: Research decays by age, not by diff
type: feature
status: done
milestone: v0.2
depends_on:
- 39
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] A research page older than the policy is reported, with the reason
- [x] A kind without the policy is unaffected

## 2026-09-23

stale_after on a kind, as 180d, 26w, 6m or 1y; loam.toml refuses anything else. The age is from the review's date, or the last body edit. sources is a loam convention, not a key of the format: show lists each one's title or URL and the date it was read.

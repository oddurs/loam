---
id: 42
title: A squash or rebase must not make every page stale
type: feature
status: done
milestone: v0.2
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
pillar:
- freshness
effort: m
---

## Problem

A reviewed commit on a branch that is later squash-merged no longer exists on
`main`. Compare from a commit that is not there and either nothing works or
everything is stale.

## Proposal

When `reviewed.commit` is not an ancestor of HEAD, fall back to `reviewed.date`:
compare from the last commit on the current branch at or before that date, and
say once that it did. `review` on the next run writes a commit that exists.

## Acceptance criteria

- [x] A test repository squash-merges a branch whose pages were reviewed on it, and `stale` reports only real changes
- [x] The fallback is announced, once per run

## 2026-09-23

Built differently from the proposal, and better for it: when the reviewed commit is not on this branch, the baseline is the commit that brought the review here, found by git log -S on the reviewed commit's name in the page. That commit carried the code the page was read against. The review's date is only the last resort. Comparing from the date, as proposed, would have made a branch squash-merged days after its reviews arrive with every page it reviewed stale, for the very changes they were reviewed against. The test squash-merges a branch, deletes it and garbage-collects, so the reviewed commits are truly gone.

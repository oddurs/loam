---
id: 49
title: 'context: the pages that matter for a path, within a budget'
type: feature
status: backlog
milestone: v0.3
depends_on:
- 33
- 39
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- agents
- freshness
effort: m
---

## Problem

An agent about to change `src/engine/scheduler.rs` does not know that
`docs/design/scheduling.md` exists, argues against exactly the change it is about
to make, and was reviewed last week. So it re-derives the design, or contradicts
it.

## Proposal

`loam context PATH… [--budget 8k]` returns the pages whose `covers` match the
paths, then pages they link to, ranked, each with its freshness. Within the
budget it prints full bodies; past it, titles and summaries. Stale pages are
flagged, not omitted — being told a page is stale is itself context.

This is the command an agent's pre-edit hook calls, and the most useful thing
loam does for an agent.

## Costs

Budgets in bytes are honest; in tokens they depend on a tokenizer loam should not
ship. Bytes, with the ratio documented.

## Acceptance criteria

- [ ] Given a covered path, returns its pages first
- [ ] Never exceeds the budget; says what it left out
- [ ] Wired into a Claude Code hook in the cookbook

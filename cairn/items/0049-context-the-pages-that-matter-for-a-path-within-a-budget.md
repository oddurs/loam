---
id: 49
title: 'context: the pages that matter for a path, within a budget'
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 33
- 39
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Given a covered path, returns its pages first
- [x] Never exceeds the budget; says what it left out
- [x] Wired into a Claude Code hook in the cookbook

## 2026-09-23

Three rings: covers (most paths, then the most specific pattern, first), mentions of the path in a page's text, and pages the covering pages link to. Each with its freshness; a stale page is flagged STALE in its heading, not left out. Budget in bytes, 8k default, 1k = 1024, with 'about four bytes a token' in the help. Whole pages while they fit, then summaries, then a 'left out' line; a final cut guarantees the budget, tested at 300, 600, 1k and 5000 bytes. The hook: Claude Code's PreToolUse cannot add context (it can only allow or refuse, per its documentation as a guide agent read it), so the recipe is a PostToolUse hook, once per file per session: docs/cookbook/loam-context.sh, documented in docs/guide/claude-code-hooks.md. Run against the documented hook input; not run in a live session.

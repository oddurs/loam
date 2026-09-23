---
id: 15
title: Where an architecture decision record lives
type: decision
status: done
milestone: v0.0
assignee: Oddur Sigurdsson
depends_on:
- 14
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
pillar:
- format
- links
effort: s
---

## Context

code-as-color keeps five ADRs in `docs/decisions/`, each with `**Status:**
accepted` in its body. cairn's `0071` claims its items already *are* decision
records. If both are true, a decision has two homes, and the tense rule does not
obviously break the tie: a decision is made at a moment (history), but "we use
Typst" is also how things are now.

## Options

1. **In cairn**, as items of type `decision`. They have a status, a date they
   were closed, `supersedes`, and a history `cairn log` can read. This backlog
   already does it.
2. **In the docs**, as pages of kind `decision`. Where people expect ADRs to be,
   and where code-as-color already keeps them.
3. **Both**: the decision in cairn, and a design page that states the current
   design in the present tense and cites the decisions behind it.

## Recommendation

**Option 3, with the decision itself in cairn.** A decision has alternatives,
a moment, and a successor when it is overturned — that is an item. What a reader
of the docs wants is not the record of the choice but its result, stated as
fact: that belongs in `design.md`, which cites `0002` for the argument.

## Costs

- People look for ADRs in `docs/decisions/`. loam's index can list cairn's
  decisions under the design kind (v0.4, resolving cairn references), so they
  are still found from the docs.
- code-as-color's five ADRs would move. `cairn import` or a short script.

## This is genuinely a preference

Option 2 is defensible and the author of both programs should pick. The
recommendation is the one that keeps each program's model clean.

## What it rules out

- A `decision` page kind as the home of a decision record.
- Keeping a decision in both places, with the page restating the argument.
- Status lines such as `**Status:** accepted` in a page body. A page's status
  is draft, current or superseded (0018), and a decision's is its item's.

## Acceptance criteria

- [x] The decision is stated in one sentence under Recommendation
- [x] What it rules out is written down
- [x] If option 1 or 3: an item filed to move code-as-color's ADRs

## 2026-09-22

Decided: option 3. The decision record is a cairn item; the design page states its result in the present tense and links the item. The recommendation was flagged as a preference for the author of both programs; it is taken because it is the only option under which the tense rule (0014) needs no exception, and loam's v0.4 cairn resolution (0054) can still list decisions from the docs index. The move is filed as 0061.

---
id: 14
title: Tense divides the backlog from the docs
type: decision
status: backlog
milestone: v0.0
depends_on:
- 13
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- format
- links
effort: s
---

## Context

With two programs holding written context in one repository, something has to
say which one a given piece of writing belongs in. If nothing does, both fill
with the same things and neither can be trusted to be complete.

## Options

- By author: people write docs, agents write items. (Arbitrary, and false today.)
- By length: short in cairn, long in docs. (cairn's items already run ninety lines.)
- **By tense.**

## Recommendation

**A cairn item records why something changed. A page says how it is now.**

cairn's items are worth more after they close because they answer *why is it
like this* (cairn `0071`). A page is worth something only while it is current.
One is history written at the moment of deciding; the other is a description
kept true.

Three seam rules follow:

1. **A question is an item; its answer, if it outlives the question, is a page.**
   harrow's `0022` is the pattern: the spike is opened, worked and closed in
   cairn. A finding longer than a paragraph goes to `docs/research/`, and the
   item links to it.
2. **A plan with steps that will be finished is backlog.** A milestone's body
   plus its items *is* the plan. A "plan" that describes the shape of the thing —
   measure-of-the-world's chapter outlines — is design, and is a page.
3. **A page cites the items that made it how it is.** The design page says what
   the design is; the decisions behind it are linked, not restated.

## Consequences

- `docs/plan/PLAN.md`-style files mostly stop existing; their content is either
  a milestone or a design page.
- Where a decision record lives is the hard case, and is its own decision.

## Acceptance criteria

- [ ] The decision is stated in one sentence under Recommendation
- [ ] What it rules out is written down
- [ ] The three seam rules survive being applied to every file in the three real docs folders

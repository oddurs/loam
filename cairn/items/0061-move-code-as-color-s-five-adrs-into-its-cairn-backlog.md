---
id: 61
title: Move code-as-color's five ADRs into its cairn backlog
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 15
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p2
pillar:
- links
effort: s
---

## What

0015 decided that a decision record lives in cairn, and that what it decided is
stated in the present tense on a design page. code-as-color keeps five ADRs in
`docs/decisions/` (PDF not web, Typst not LaTeX, no transpiler, figures are
code, ET Book), each with `**Status:** accepted` as its first paragraph — which
is also why loam infers "**Status:** accepted" as their summary.

In code-as-color, as part of adopting it (0036):

- each ADR becomes a closed `decision` item in its `cairn/items`, keeping its
  Context, Decision and Consequences, dated when it was decided;
- `docs/design.md` and `docs/architecture.md` state each result as fact and link
  the item (spec §6.4), where they do not already;
- `docs/decisions/` is removed, and any link to it is repointed.

## Acceptance criteria

- [x] Five closed decision items in code-as-color's backlog, one per ADR
- [x] Every link that named a file in `docs/decisions/` resolves after the move
- [x] `loam check` in code-as-color reports nothing that the move caused

## 2026-09-22

Done on code-as-color's loam branch (df0dde4): a decision type added to its cairn.toml, the five ADRs as closed items 0071 to 0075, design.md and architecture.md linking them under 'Decided', docs/decisions/ removed. The only link into it was the hand-kept index row, which the generated index replaced.

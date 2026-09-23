---
id: 64
title: 'order: a page''s place among its kind'
type: spec
status: backlog
milestone: v0.1
created: 2026-09-22
updated: 2026-09-22
priority: p2
---

## Current text

None. The index sorts a kind's pages by title.

## Proposed text

`order`, an optional integer: a page's place among the pages of its kind,
lowest first. Pages without one follow those with one, by title. A value that
is not an integer is `malformed-key`, like any other key of the wrong type.

## What a conforming reader has to do differently

Report `order` in the reading, and `malformed-key` for a value that is not an
integer. A new optional key, which spec §9 says costs no version.

poptop's hand-kept index is in a deliberate order — First run first — and
0030's criterion is that the generated one carries everything the hand-kept one
does. Title order loses that. A key in the frontmatter keeps the order with the
page it belongs to, rather than in a list in loam.toml that a rename would break.

## Acceptance criteria

- [ ] spec/README.md updated
- [ ] A corpus case exercises it

---
id: 54
title: Resolve cairn references by asking cairn
type: feature
status: backlog
milestone: v0.4
depends_on:
- 22
- 24
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- links
effort: m
---

## Problem

A page citing `0072` should show what `0072` is and whether it is still open —
and `check` should notice when it names an item that does not exist.

## Proposal

Run `cairn list --all --json` once per invocation and resolve references against
it — the integration cairn's manual prescribes rather than parsing the items
again. With cairn absent, references are reported as unchecked, not broken.

Also: list cairn items of type `decision` under the design kind in the index,
if the ADR decision went that way.

## Acceptance criteria

- [ ] A reference to a missing item is a `check` finding
- [ ] Works, degraded and saying so, with cairn not installed

---
id: 54
title: Resolve cairn references by asking cairn
type: feature
status: done
milestone: v0.4
assignee: Oddur Sigurdsson
depends_on:
- 22
- 24
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] A reference to a missing item is a `check` finding
- [x] Works, degraded and saying so, with cairn not installed

## 2026-09-23

Both criteria were already true by the time this was taken: a reference to a missing item has been broken-cairn-link since v0.1, resolved by file name (spec §6.4), and that needs no cairn. What asking cairn adds is what the item is: src/cairn.rs runs cairn list --all --json once, and show prints each cited item's title and status; the manifest gains cairn_items (cited items only: title, status, category, type) and cairn_error. Without cairn, show says the items are unchecked and the manifest's cairn_error says why; tested with PATH emptied. Not done: listing decision items under the design kind in the index — the pages-and-items design keeps decisions in cairn and has pages link them, so the index stays pages.

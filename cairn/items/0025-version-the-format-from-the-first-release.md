---
id: 25
title: Version the format from the first release
type: spec
status: backlog
milestone: v0.0
depends_on:
- 23
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- format
effort: s
---

## Current text

None.

## Proposed text

The config carries `format = 1`. A reader refuses a format it does not know
rather than misreading it. The spec lists what costs a format number (a key
changing meaning, a default changing) and what does not (a new optional key).

cairn added this at format 1 and found the list of what costs a number was the
useful part; write it on day one.

## What a conforming reader has to do differently

Refuse an unknown format with a message naming both versions.

## Acceptance criteria

- [ ] spec/README.md updated
- [ ] A corpus case exercises it

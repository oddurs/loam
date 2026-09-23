---
id: 25
title: Version the format from the first release
type: spec
status: done
milestone: v0.0
depends_on:
- 23
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] spec/README.md updated
- [x] A corpus case exercises it

## 2026-09-22

Written as spec §9, with the list of what costs a number. Writing it found one thing the item did not foresee: 'unknown configuration keys are ignored' lets the configuration grow, but a key that changes which files are pages would then be silently ignored by an older reader. §9 now says such a key costs a number even though it is optional. Corpus: hostile/format-unknown, format-missing, format-as-string; spec/reader.py refuses with a message naming both versions.

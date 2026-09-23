---
id: 48
title: Warn before writing a page that already exists under another name
type: feature
status: backlog
milestone: v0.3
depends_on:
- 33
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- agents
effort: m
---

## Problem

The commonest way a docs folder decays under agents is not a wrong page but a
second one: `research/markdown-crates.md` and, a week later,
`research/rust-markdown-parsers.md`. cairn's `0107` is the same problem for
items.

## Proposal

`loam new` compares the new title against existing titles and summaries of the
same kind, and when something is close prints it and asks — or, for an agent,
exits non-zero with the candidates unless `--anyway`. Plain token overlap; no
embeddings (see the boundary).

## Acceptance criteria

- [ ] The example pair above is caught
- [ ] False positives on the three folders counted, and few

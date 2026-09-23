---
id: 33
title: list, show and search
type: feature
status: done
milestone: v0.1
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
pillar:
- cli
effort: m
---

## Problem

"What is written about X?" is the first question before writing anything, and
answering it by opening a file browser is why people and agents write it again.

## Proposal

- `list` — pages, filterable by kind and status, as a table or `--json`
- `show PATH` — frontmatter, derived facts (backlinks later, freshness later),
  body
- `search TEXT` — titles, summaries and bodies; matches ranked title first.
  A scan of files, no index kept on disk (see the boundary).

## Acceptance criteria

- [x] All three have `--json`, documented
- [x] `search` on the three folders answers in well under a second

## 2026-09-22

All three have --json, documented in docs/reference/json-output.md. search takes about 20 ms on each real folder; tests/cli.rs fails it above 500 ms.

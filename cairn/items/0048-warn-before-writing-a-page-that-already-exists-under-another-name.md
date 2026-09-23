---
id: 48
title: Warn before writing a page that already exists under another name
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 33
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] The example pair above is caught
- [x] False positives on the three folders counted, and few

## 2026-09-23

Words of the new title against the title and summary of each page of the same kind, with a crude stemmer so parsers meets parse and crates meets crate. A page is raised when it shares two words and half the title, or the same title. The example pair is caught (shares markdown, pars, rust). Counted on the three real folders by asking new for every page's own title: first 4 other pages raised in 66, all false — three from measure-of-the-world's own name in titles and summaries, one from poptop's 'one' and 'process'. Words in a third or more of a kind's pages are now not evidence: 2 in 66, both false (One process in depth against Scrubbing to a moment; the Global writing guide against Production engine). A person at a terminal is asked; an agent or anything without a terminal is refused, exit 1, naming the pages, unless --anyway.

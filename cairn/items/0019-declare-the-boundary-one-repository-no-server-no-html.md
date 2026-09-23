---
id: 19
title: 'Declare the boundary: one repository, no server, no HTML'
type: decision
status: done
milestone: v0.0
assignee: Oddur Sigurdsson
depends_on:
- 13
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
pillar:
- format
- index
effort: s
---

## Context

"Indexable source you could build a docs site from" invites three expansions,
each of which would swallow the project: becoming a site generator, becoming a
search service, becoming a wiki. cairn's `0059` drew its line before anybody was
watching; this does the same.

## Recommendation

**loam reads and writes a directory of Markdown in one repository, and emits
what it knows as files and JSON. Nothing else.**

It will not:

- **render HTML.** Astro, mdBook, Zola and a dozen others do that well. loam's
  job is a manifest good enough that any of them can.
- **run a server**, including a search server. `search` is a scan of files.
- **embed, vectorise or keep a search index on disk.** The files are the index.
  An agent that wants semantic search can build it from the manifest; loam does
  not ship one to keep in step.
- **look across repositories.** A repository cannot see another one.
- **edit prose.** `$EDITOR` does that.
- **use the network**, not even to check that an external link still answers.
  A check that passes or fails with the weather is not a check, and the spec
  (§6) says a reader must not.

## Consequences

- `loam serve` does not exist, and its absence is not an omission.
- The manifest's schema is the product's most important interface after the
  page format, and is specified as carefully.

## Acceptance criteria

- [x] The decision is stated in one sentence under Recommendation
- [x] What it rules out is written down
- [x] Stated in the README

## 2026-09-22

Decided as recommended, and 'no network' added to the list: 0024 already assumed it ('absolute URLs: never resolved, never checked (no network, per the boundary)') and the boundary did not yet say so. Stated in README.md under 'What loam is not', and in spec §6 and §8.

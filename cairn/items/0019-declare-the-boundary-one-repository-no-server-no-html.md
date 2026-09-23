---
id: 19
title: 'Declare the boundary: one repository, no server, no HTML'
type: decision
status: backlog
milestone: v0.0
depends_on:
- 13
created: 2026-09-22
updated: 2026-09-22
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

## Consequences

- `loam serve` does not exist, and its absence is not an omission.
- The manifest's schema is the product's most important interface after the
  page format, and is specified as carefully.

## Acceptance criteria

- [ ] The decision is stated in one sentence under Recommendation
- [ ] What it rules out is written down
- [ ] Stated in the README

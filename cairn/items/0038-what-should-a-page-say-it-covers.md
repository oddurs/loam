---
id: 38
title: What should a page say it covers?
type: question
status: backlog
milestone: v0.2
depends_on:
- 36
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- freshness
- format
effort: s
---

## Question

What does a page declare so that loam can tell the code under it has changed —
and at what grain, so that a change to a comment does not make every page stale?

## Why it has to be answered first

It becomes a format key, and a key named in a hurry is a migration later. It
also decides whether the feature is useful or noise: too coarse and every page
is always stale; too fine and nobody writes it.

## Options

1. **Path globs** — `covers: [src/engine/**, src/runtime.rs]`. Language-blind,
   cheap, coarse.
2. **Symbols** — `covers: [engine::Engine::step]`. Precise, needs a parser per
   language, breaks on rename.
3. **Globs, weighted by the size of the change** — a one-line change reported
   differently from a rewrite. Globs in the format; judgement in the program.

## What would settle it

Annotate ten real pages from the adopted folders with globs, replay the last
three months of each repository's history, and for each flag ask: would I have
wanted to reread this page then? Count the answers.

## Answer

<!-- Filled in when the spike closes. -->

## Acceptance criteria

- [ ] Answer written, with the evidence that settled it
- [ ] The true and false positive counts from the replay

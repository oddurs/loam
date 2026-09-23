---
id: 13
title: A separate program that reads cairn, rather than a type inside it
type: decision
status: backlog
milestone: v0.0
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- format
- links
effort: s
---

## Context

cairn could hold pages today: declare `research`, `design` and `guide` types,
give them templates, done. It is worth being clear why that is the wrong answer
rather than merely an unfashionable one.

## Options

1. **Page types in cairn.** No new program. Every page gets a number, a status
   from open/active/done/dropped, and a place in `cairn next`.
2. **A `cairn doc` subcommand family.** One binary, two models inside it.
3. **A separate program** that shares cairn's conventions — Markdown with YAML
   frontmatter, a TOML schema, a written spec, generated files checked in CI —
   and reads cairn's output rather than linking its code.

## Recommendation

**A separate program.** Every part of cairn's model fits work and misfits pages:

| | a cairn item | a page |
| --- | --- | --- |
| identity | a number, because titles change | its path, because that is how links address it |
| shape | a flat directory | a tree people navigate |
| lifecycle | open → active → done | draft → current → superseded; never done |
| the question | what is ready? | is this still true? |

`cairn next` means nothing for a page, and staleness means nothing for an item.
Option 1 bends a model that is clean; option 2 puts two models behind one name
and makes each harder to explain.

cairn's own promise settles the rest: things built beside cairn *should read its
file format rather than link its code*. loam is the first test of whether that
sentence was true.

## Consequences

- Two binaries. That is a cost, and the price of each staying explicable.
- Conventions are shared by copying them deliberately, not by a common crate —
  cairn's `0060` refuses to ship a library target, and this does not ask it to.
- Where the two meet (links, harrow) is specified as file formats and `--json`
  outputs, never as function calls.

## Acceptance criteria

- [ ] The decision is stated in one sentence under Recommendation
- [ ] What it rules out is written down

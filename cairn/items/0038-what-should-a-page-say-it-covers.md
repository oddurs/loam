---
id: 38
title: What should a page say it covers?
type: question
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 36
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

**Path patterns (option 1) in the format, and the size of a change used to
order what is reported, never to hide it (option 3's weighting, as display).**
What decides whether staleness is signal or noise is not the grain of the key
but whether the patterns name what the page describes. Spec §4.3.

### The replay

Ten pages from the adopted folders were given `covers` by reading each page
and naming the files it describes. Then every first-parent commit of each
repository was replayed through `loam stale --at COMMIT`, with the pages'
`covers` as they are now: poptop, 196 commits over its whole history
(2026-09-04 to 09-21), and measure-of-the-world, 174 commits over nine months
(2026-01-04 to 09-09). A flag is a commit that changed a covered file while the
page's body had not changed since. Each flag was judged by reading the commit's
diff and the page: would I have wanted to reread the page then?

| Page | `covers` | Flags | Would reread | Would not |
| --- | --- | ---: | ---: | ---: |
| poptop reference/themes.md | src/theme.rs, themes | 3 | 2 | 1 |
| poptop design/colour.md | src/theme.rs, src/cvd.rs, themes | 4 | 3 | 1 |
| poptop reference/keys.md | src/keys.rs, src/command.rs | 0 | — | — |
| poptop reference/filter.md | src/query.rs | 0 | — | — |
| poptop reference/configuration.md | src/config.rs | 2 | 0 | 2 |
| poptop reference/output.md | src/export.rs, src/report.rs | 0 | — | — |
| poptop reference/platforms.md | src/collect | 2 | 0 | 2 |
| measure build.md, first pass | Makefile, scripts, src/preamble.tex | 60 | 11 | 49 |
| measure build.md, narrowed | Makefile, latexmkrc, requirements.txt | 9 | 9 | 0 |
| measure releases.md | .github/workflows | 1 | 1 | 0 |
| measure publication.md, first pass | src/metadata.tex, src/preamble.tex, Makefile | 10 | 2 | 8 |
| measure publication.md, narrowed | src/metadata.tex, src/preamble.tex | 6 | 2 | 4 |

**First pass: 82 flags, 19 worth rereading, 63 not.** **With the two broad
annotations narrowed, as a person would after seeing the first pass: 27 flags,
17 worth rereading, 10 not.** Four of those ten are an artefact of replaying:
in January `publication.md` was a generic checklist that described none of the
code its present `covers` names. Without them, 17 of 23.

What the 63 were:

- **A pattern broader than the page.** `build.md` covered all of `scripts/`,
  which is mostly the figures' content, and all of `src/preamble.tex`, which is
  mostly typography: 49 flags, none about the build. Narrowed to what the page
  describes — the Makefile, latexmkrc and requirements.txt — every flag was
  about the build, and the author's own Sep 9 rewrite of the page documents the
  same changes. **This is the finding.** The key is right; how it is written
  decides everything.
- **Reverts.** poptop's #148 accidentally reverted the theme work and #151 put
  it back the same day: one flag each, on two pages, for a change that netted
  out. No grain catches this; the ordering by size does not either.
- **Test code and plumbing inside a covered file.** A retry in a `#[cfg(test)]`
  module of `src/collect/linux.rs`; a helper made `pub` in `src/config.rs`.
  Symbols (option 2) would have skipped these two — and would not exist at all
  for measure-of-the-world, whose code is LaTeX and a Makefile. A parser per
  language for two flags in eighty-two is not a trade worth making.

**Weighting as a filter was tested and refused.** In poptop every flag not worth
rereading changed 23 lines or fewer and every one worth it 135 or more, so a
threshold looked promising; in measure-of-the-world the narrowed `build.md`'s
flags include a two-line Makefile change ("Update Makefile build
configuration") that was worth rereading, and a 3,708-line figure commit that
was not. Size orders the report — biggest first — and hides nothing.

**It found real staleness.** At the end of the replay, two pages are stale now
and were not known to be: measure-of-the-world's `releases.md` says the
workflow runs on version tags, and since cc486cc it also builds every push and
pull request; its `build.md` documents `make build`, `watch`, `clean`,
`distclean` and `figures`, and since 203c949 and 08b031d the Makefile also has
`claims`, `refs`, `figure-qa`, `status`, `verify`, `lint` and `derivations`.

### Judged by whom

Every verdict above is mine — the agent's — made from the diffs, not by the
books' author. The flags and commits are in the table so a person can check
them; the milestone's own criterion asks for a person's agreement, and is left
for one.

## Acceptance criteria

- [x] Answer written, with the evidence that settled it
- [x] The true and false positive counts from the replay

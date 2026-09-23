---
order: 3
summary: What loam compares a page against, why the size of a change orders the report but hides nothing, and what squashing does to a review.
covers:
  - src/fresh.rs
  - src/covers.rs
  - src/git.rs
reviewed:
  commit: c4481a37a7dbd5329ab14d815ae064eb0b706180
  date: 2026-09-23
---

# How loam decides a page is stale

A stale `architecture.md` is worse than none, because it is read with the
authority of something written down. Nothing in ordinary docs tooling knows a
page and its code have diverged. git does, and loam asks it — through the `git`
command, as `cairn log` does, so history is read exactly as the person's own
git reads it.

## A page is stale when what it covers changed after it was last read

A page names the files it describes in `covers`, and records in `reviewed` the
commit it was last read against ([spec §4.3](../../spec/README.md#43-covers-reviewed-and-generated)).
It is stale when a commit after that one changed a covered file. Staleness is
never written into the page: it is derived from the history every time, so it
cannot itself go stale.

The commit to compare from — the *baseline* — is, in order:

1. **The reviewed commit**, when it is on the current branch.
2. **The commit that brought the review here**, when it is not. A review made
   on a branch that is then squash-merged names a commit that does not exist
   on `main`. But the `reviewed:` line itself arrived on `main` in some commit,
   and that commit carried the code the page was read against. git's pickaxe
   finds it. The item proposed comparing from the review's date
   ([0042](../../cairn/items/0042-a-squash-or-rebase-must-not-make-every-page-stale.md));
   then a branch squash-merged a few days after its reviews would arrive with
   every page it reviewed already stale — for the very changes they had been
   reviewed against.
3. **The review's date**, as the last commit on or before it, when the review
   came from somewhere with no history here at all.
4. **For a page never reviewed, the last commit that changed what it says.**
   Whoever wrote that commit had the code in front of them. A commit that
   changes only frontmatter — adopting a folder, recording a review — does not
   count: it did not change what the page says. Nor does moving the page: its
   history is followed back through the move.

Each fallback is announced, once per run, so a number nobody asked for is
never silently in play.

Judging an earlier commit with `stale --at`, a review made after that commit
is set aside, and that is announced too: as of then, there was no review.

One more thing is announced: a shallow clone. It has no history before its
first commit, so every page looks as if it was written there and nothing can be
stale. That is continuous integration's default, and a check that passes for
ever because it cannot see is worse than no check.

## What counts as a change

Every commit after the baseline that changed a covered file, with whitespace
ignored: reindenting code does not make a page describing it untrue. A merge
counts for what it changed beyond merging — a conflict resolved by rewriting a
file, or a change slipped in — and not for what the merged branch changed,
which its own commits have counted already.

A page's own file is never a change to it. A page covering the directory it is
in would otherwise go stale by being written, and again by being reviewed.

The size of each change orders the report, biggest first, and hides nothing.
That was tested: replaying two repositories' histories, a threshold that looked
right for poptop — every flag worth rereading changed 135 lines or more, every
other 23 or fewer — would have hidden a two-line Makefile change in
measure-of-the-world that was worth rereading
([0038](../../cairn/items/0038-what-should-a-page-say-it-covers.md)).

## What is not stale

- **A page without `covers`** is *unknown*. Reporting it as fresh would be a
  claim loam cannot make.
- **A page being updated** — changed in or after the newest change to its code,
  or changed in the working tree — is set apart as *being updated*, and `check
  --stale` does not warn about it. An edit made before that change does not
  count: fixing a typo after one change says nothing about the thirty after it.
- **A generated page** is kept true by its generator.

## What a change just made stale

`stale` asks what history says. An agent that has just edited three files asks
something narrower: which pages has *this* change made wrong? `stale
--working-tree` answers it from the uncommitted changes alone, and `--since
REV` from the commits since this branch left `REV`, so what landed on `REV` in
the meantime is not counted — every page covering a changed file, whatever
its baseline, less those changed alongside. No review, no size, no ordering
beyond the pages' own: it is a list to work through before stopping, not a
judgement about history.

## Why patterns and not symbols

A page could name functions instead of files, and a change to a comment beside
them would then not count. The replay says it would have mattered little: of
63 flags not worth rereading, two were changes a symbol would have skipped (a
test module inside a covered file; a helper made public), and 49 were a pattern
broader than its page. And symbols need a parser per language, where one of the
two repositories replayed is LaTeX and a Makefile. The pattern is the format;
choosing it well is the author's job, and the one that decides whether
staleness is signal or noise.

## One question to git, not one per page

Every page is judged from one `git log` over everything any page covers, taken
from where all their baselines meet; each page then takes its share by which
commits are after its own baseline. The pages' own history is one more log, of
the docs directory, and every version of a page it compares comes through one
`git cat-file`. A repository of 5000 commits and 200 pages is judged in half a
second, where a log per page took most of a minute; `show` and `context` judge
only the pages they print.

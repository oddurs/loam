---
order: 3
summary: Say what code a page describes, find the pages it has moved out from under, and mark them read again.
covers:
  - src/cmd/stale.rs
  - src/cmd/review.rs
  - src/fresh.rs
---

# Keeping pages true

A page that describes code which has since changed reads exactly like one that
does not, and an agent reads it at the start of every session with the same
trust. loam can tell the two apart, if the page says what it covers.

## 1. Say what a page covers

```markdown
---
covers:
  - src/theme.rs
  - themes
---

# Themes
```

Each entry is a path from the repository root, and may use `*`, `?` and `**`.
A directory covers everything in it. An entry beginning `!` takes files back
out — quote it, because YAML reads a bare `!` as something else:

```yaml
covers:
  - src/engine
  - "!src/engine/tests"
```

Name what the page describes, not the directory it is near. When
measure-of-the-world's build page covered all of `scripts/`, 49 of 60 flags in
nine months of its history were changes to figure content; covering the
Makefile, `latexmkrc` and `requirements.txt` instead, every flag was about the
build ([0038](../../cairn/items/0038-what-should-a-page-say-it-covers.md)).

## 2. `loam stale`

This is real output, from measure-of-the-world with three of its pages given
`covers`:

```console
$ loam stale
docs/build.md   never reviewed; last edited 14 days ago at cc486cc
  Makefile   2 commit(s), +37 −3   "Add prose lint"
  latexmkrc  1 commit(s), +7 −0   "Add proof engine; cut overfull lines from 122 to 41"
docs/publication.md   never reviewed; last edited 14 days ago at a880c3d
  src/preamble.tex  1 commit(s), +33 −1   "Add proof engine; cut overfull lines from 122 to 41"
docs/releases.md   never reviewed; last edited 259 days ago at 0aa28b0
  .github/workflows  1 commit(s), +22 −0   "Fix the build pipeline so failures are visible"

3 stale, 0 being updated, 0 fresh, 13 unknown (no covers), 0 generated
```

It was right about all three. `build.md` documents five make targets and the
Makefile now has twelve; `releases.md` says the workflow runs on version tags,
and it now builds every push as well.

Pages are ordered by how much their code changed, not by how long ago. A page
with no `covers` is *unknown*, never *fresh*: loam has nothing to judge it by,
and saying nothing would be a claim. A page that has never been reviewed is
compared from the last commit that changed what it says — a change to its
frontmatter alone does not count. `--json` gives the same for a program.

Before committing, `loam stale --working-tree` asks a narrower question —
which pages cover what you have just changed — and `--since REV` asks it of the
commits since `REV`. That is the list to work through before you stop; an
agent's stop hook can ask it for you ([Claude Code hooks](claude-code-hooks.md)).

## 3. `loam review`

When you have read the page against the code and it is true — or you have made
it true:

```console
$ loam review docs/releases.md
reviewed docs/releases.md against d8b7603 (2026-09-23)
```

That writes the commit and the date into the page, and nothing else:

```yaml
reviewed:
  commit: d8b76035123ca5d73ba1cb9454e47416b9b111c8
  date: 2026-09-23
```

Commit it, and the page is fresh until its code changes again. `--note` adds a
line to the page's `## Review log` — starting one at the end of the page if it
has none — for when *why* it is still true is worth keeping. If a covered file has changes not yet committed, `review`
says so: the review would be against code no commit holds.

A review made on a branch that is later squashed or rebased still counts. loam
finds the commit that brought the review onto the current branch, and compares
from there, and says once per run that it did.

## 4. In CI

```sh
loam check --stale
```

reports each stale page as a warning on its `covers:` line, and in GitHub
Actions as an annotation on the pull request. It exits 0: a stale page is not a
broken build, and a check that fails for one gets turned off within the week.
A project that wants the gate adds `--strict`.

A page changed in the same pull request as its code is not reported — its
author is plausibly updating it.

CI must fetch the whole history. A shallow clone — `actions/checkout`'s
default — has none before its one commit, so every page looks freshly written
and nothing is ever stale; loam says so when it sees one. In GitHub Actions:

```yaml
- uses: actions/checkout@v7
  with:
    fetch-depth: 0
```

## Research, which goes stale by age

A page about the world rather than the code — which Markdown crates exist,
what a vendor's API allows — goes stale because the world moved. Give its kind
a lifetime:

```toml
[[kind]]
name = "research"
dir = "research"
stale_after = "180d"
```

A research page reviewed longer ago than that is stale, whatever it covers.
`show` lists the `sources` it gives, with the date each was read:

```yaml
sources:
  - title: A survey of Markdown crates
    url: https://example.com/survey
    read: 2026-09-01
```

## Generated pages

A page a program writes — `make status` — is kept true by the program. Say so,
and loam leaves it out:

```yaml
generated: make status
```

---
covers:
  - src/cmd/init.rs
  - src/cmd/render.rs
  - src/index.rs
order: 1
reviewed:
  commit: ac82a5f4bd513492a6325dd4d3e0926682c69c83
  date: 2026-09-23
---

# Adopting a docs folder

Point loam at a `docs/` folder that already exists, see what it makes of every
page, and replace the hand-kept index with a generated one. Nothing here
requires changing a page.

Every command and all output below is real. It was run on a fresh copy of
[poptop](https://github.com/oddurs/poptop), whose `docs/` holds 37 pages in
`guide/`, `reference/`, `design/` and `roadmaps/` and has no frontmatter at all.

## 1. `loam init`

```console
$ loam init
docs/README.md                               page       "poptop documentation"
docs/design/answering-questions.md           design     "The keys, and the questions they answer"
docs/design/collection-cost.md               design     "What it costs to watch"
…
docs/roadmaps/README.md                      roadmap    "Roadmaps — design rationale"

wrote loam.toml: 37 page(s), kinds guide (docs/guide/), reference (docs/reference/), design (docs/design/), roadmap (docs/roadmaps/), page (docs/)
no page was changed.

next: correct any guess above in loam.toml, then `loam check`.
      docs/README.md has no markers yet: add `<!-- loam:index:begin -->` and `<!-- loam:index:end -->` where the index goes.
```

`init` finds `docs/` (or `doc/`; `--docs` names another), proposes a kind for
each directory that holds pages, and adds a kind at the docs root for anything
else. It writes [`loam.toml`](../reference/loam-toml.md) and nothing more. Every
line is a guess you can see: the kind it chose, and the title it read from the
page's first heading. Past forty pages it gives a count for each kind instead,
and lists only the pages with something to fix.

The index goes in the folder's `README.md`, as here. A site's docs folder —
mkdocs, VitePress, Docusaurus — usually opens with an `index.md` instead, and
then that is the index, rather than a second home page beside it.

A folder without directories gets one kind. A folder you would rather sort
into a known shape can start from a preset: `loam init --preset diataxis`
(guide, reference, design) or `--preset standard` (and research).

If the guesses are wrong, correct `loam.toml`: rename a kind, give it a
`description`, or set `index = false` on one the index should leave out.

## 2. `loam check`

```console
$ loam check
ok: 37 page(s), 0 warning(s)
```

poptop's links were already sound. A link to a file git ignores — a reference
generated when the site is built, never committed — is not called broken, since
no checkout has it; `check` says how many there were instead. On
measure-of-the-world, the same command
found seven broken links, six of which worked on the author's Mac:

```console
loam: docs/contributing.md:24: link to `STYLEGUIDE.md`, which does not exist; `styleguide.md` differs only in case [broken-link]
```

Every finding has a file and a line, and [a code](../reference/findings.md).

## 3. The index

poptop's `docs/README.md` opens with a paragraph and a list, then keeps a table
of every page by hand. Replace the tables with two marker lines:

```markdown
- **[Design](#design)** — "why is it like this?" The arguments, kept.

<!-- loam:index:begin -->
<!-- loam:index:end -->

## The repository itself
```

then:

```console
$ loam render
rendered docs/README.md
$ loam check --strict --render
ok: 37 page(s), 0 warning(s)
```

Everything outside the markers is left as it was; a marker counts only alone
on its line and outside code, so a sentence or an example that mentions one is
safe. Between them, loam writes a section per kind, in the order `loam.toml`
declares them, and a row per page: its title, and its summary — the opening
sentences of the first paragraph under its title, as many as fit in 160 bytes,
unless its frontmatter gives a `summary`, which is shown whole. [The index](../design/the-index.md) says why.

To keep the hand-kept order and the hand-written notes, give each page an
`order` and a `summary`:

```markdown
---
order: 1
summary: "what is on screen, the three regions, the first four keys"
---

# First run
```

Then enable the hook in `loam.toml`, so that every command that changes pages
renders the index again:

```toml
[hooks]
after-change = "loam render --quiet"
```

## 4. Everyday use

```console
$ loam list --kind guide
PATH                           KIND   STATUS   TITLE
docs/guide/first-run.md        guide  current  First run
docs/guide/groups.md           guide  current  Groups and trees
…
$ loam new design "How replay finds a day" --draft
docs/design/how-replay-finds-a-day.md
$ loam mv docs/guide/whats-slow.md docs/guide/finding-what-is-slow.md
moved docs/guide/whats-slow.md → docs/guide/finding-what-is-slow.md
  rewrote 1 link(s) in README.md
  rewrote 1 link(s) in docs/README.md
  rewrote 1 link(s) in docs/guide/first-run.md
```

[Writing pages](writing-pages.md) covers those, and
[checking in CI](checking-docs-in-ci.md) how to keep it all true.

## Review log

- 2026-09-23, at f8a7645: init now writes [agents] status = draft; the walk-through's output and claims are unchanged
- 2026-09-23, at a6213f6: the index's markers count only on their own lines outside code, and inferred summaries are cut to whole sentences within 160 bytes; both now said

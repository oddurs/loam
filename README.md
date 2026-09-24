# loam

A repository's written context — research, design, reference, guides — kept as
Markdown pages under a schema, the way [cairn](https://github.com/oddurs/cairn)
keeps its backlog.

Loam is the soil a project grows in. Here it is the `docs/` folder you already
have: loam reads it as it is, generates the index you were keeping by hand,
finds the link that broke last week, and moves a page without breaking the
links to it. A plain Markdown file with a heading is already a page. Nothing has
to be rewritten to adopt it.

## How it fits

| | holds | unit | the question it answers |
| --- | --- | --- | --- |
| **cairn** | intent: why something changed | a numbered item that closes | what is ready? |
| **loam** | knowledge: how it is now | a page at a path that is kept true | is this still true? |
| **harrow** | a view onto cairn | — | what needs me? |

**A cairn item records why something changed. A page says how it is now.** A
question is an item; its answer, if it outlives the question, is a page. A plan
with steps is a milestone. A decision is an item, and the design page states
what was decided and links it. The reasoning is in
[Pages, items and the line between them](docs/design/pages-and-items.md).

## Quickstart

On [poptop](https://github.com/oddurs/poptop), whose 37 pages have no
frontmatter at all — real output, abridged:

```console
$ loam init
docs/README.md                               page       "poptop documentation"
docs/design/answering-questions.md           design     "The keys, and the questions they answer"
…
wrote loam.toml: 37 page(s), kinds guide (docs/guide/), reference (docs/reference/), design (docs/design/), roadmap (docs/roadmaps/), page (docs/)
no page was changed.

$ loam check
ok: 37 page(s), 0 warning(s)

$ loam render                      # after putting two marker lines in docs/README.md
rendered docs/README.md

$ loam mv docs/guide/whats-slow.md docs/guide/finding-what-is-slow.md
moved docs/guide/whats-slow.md → docs/guide/finding-what-is-slow.md
  rewrote 1 link(s) in README.md
  rewrote 1 link(s) in docs/README.md
  rewrote 1 link(s) in docs/guide/first-run.md
```

The whole walk-through is [Adopting a docs folder](docs/guide/adopting-a-docs-folder.md).

## Is it still true?

A page can say what code it describes, and loam asks git whether that code has
moved on without it. On measure-of-the-world, with three pages given `covers`:

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

All three were right. `loam review PAGE` records that a page has been read
against the code as it is now, and `loam check --stale` puts the same warnings
on a pull request without failing it. See [Keeping pages true](docs/guide/keeping-pages-true.md).

## Commands

| | |
| --- | --- |
| `loam init` | Adopt the docs folder here: write `loam.toml`, change no page |
| `loam check` | Broken links and anchors, malformed frontmatter, unclaimed pages — each with a file and line |
| `loam render` | Generate the index between its markers; `--check` for CI |
| `loam index --json` | The whole docs folder as one [versioned document](spec/manifest.md), to build a site from |
| `loam new KIND TITLE` | A page where its kind lives, from the kind's template |
| `loam list`, `show`, `search` | What is written, and where; all with `--json` |
| `loam mv OLD NEW` | Move a page or a directory, and rewrite every link to it |
| `loam supersede OLD NEW` | Record that one page replaces another, on both pages |
| `loam stale` | Pages whose covered code changed since they were last reviewed |
| `loam review PAGE` | Record that a page was read against the code as it is now |
| `loam context PATH` | The pages about a file, within a budget: what an agent reads before changing it |
| `loam set PAGE KEY=VALUE` | Frontmatter from the command line, changing no other byte |
| `loam agent` | The instructions an agent needs, generated from `loam.toml` |

Every command is in [Commands](docs/reference/commands.md).

## Install

On Linux or macOS, the latest release:

```sh
curl -fsSL https://raw.githubusercontent.com/oddurs/loam/main/install.sh | sh
```

puts `loam` in `~/.local/bin` (`LOAM_DIR` to choose another, `LOAM_VERSION`
for a release other than the latest), after checking it against its published
checksum. In GitHub Actions, the [action](action.yml) installs it and runs the
check in one step — see [Checking docs in CI](docs/guide/checking-docs-in-ci.md).

From source, anywhere Rust 1.98 runs:

```sh
cargo install --locked --git https://github.com/oddurs/loam
```

The crate is `loam-md` — `loam` was taken on crates.io — and the command it
installs is `loam`.

## The format

A page is specified independently of this program, in
[`spec/README.md`](spec/README.md): what a page is, how its title, summary, kind
and status are read when nobody wrote them down, which links are broken, and
what costs a format version. [`spec/reader.py`](spec/reader.py) is a second
reader, written from the specification, and [`spec/corpus`](spec/corpus) holds
three real docs folders and the cases written to break a reader. loam and the
reader in `spec/` are held to the same corpus in CI.

## What loam is not

loam reads and writes a directory of Markdown in one repository, and emits what
it knows as files and JSON. Nothing else. It will not:

- **render HTML.** Astro, mdBook, Zola and the rest do that well. loam's job is
  a manifest good enough that any of them can.
- **run a server**, including a search server. `search` is a scan of files.
- **embed, vectorise, or keep a search index on disk.** The files are the index.
- **look across repositories.** A repository cannot see another one.
- **edit prose.** `$EDITOR` does that.
- **use the network.** Not even to check an external link.

## Status

v0.4: the docs folder is a source a site can be built from. `loam index
--json` prints it as one [versioned document](spec/manifest.md) — every page,
the index's sections, headings with their anchors, backlinks, the cairn items
each page cites and whether it is still true — and
[a recipe](docs/guide/building-a-site-from-the-manifest.md) builds a site from
that and the pages' bodies alone. Before it: v0.3, agents write into it
([Working with agents](docs/guide/working-with-agents.md)); v0.2, pages say
what code they describe and loam says when it has moved on. The plan is in
[`ROADMAP.md`](ROADMAP.md).

```sh
cairn list --view decide    # decisions and questions still open
cairn next                  # what can start
```

MIT licensed. See [LICENSE](LICENSE).

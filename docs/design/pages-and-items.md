---
order: 1
summary: Why a page is not a cairn item, what goes in each, and what loam will never do.
---

# Pages, items and the line between them

loam keeps a repository's written context the way [cairn](https://github.com/oddurs/cairn)
keeps its backlog: Markdown files, a schema in a TOML file, a written
specification, and generated files checked in CI. It is a separate program
rather than a kind of item inside cairn
([0013](../../cairn/items/0013-a-separate-program-that-reads-cairn-rather-than-a-type-inside-it.md)),
because every part of cairn's model fits work and misfits pages:

| | a cairn item | a page |
| --- | --- | --- |
| identity | a number, because titles change | its path, because that is how links address it |
| shape | a flat directory | a tree people navigate |
| lifecycle | open, active, done | draft, current, superseded — never done |
| the question | what is ready? | is this still true? |

The two meet only through files: loam reads the names of cairn's item files to
check a link to one, and nothing else.

## Tense divides them

**A cairn item records why something changed. A page says how it is now**
([0014](../../cairn/items/0014-tense-divides-the-backlog-from-the-docs.md)).
Three rules follow, and they sorted every file of three real docs folders
without needing a fourth:

1. A question is an item. Its answer, if it outlives the question, is a page.
2. A plan with steps that will be finished is backlog: a milestone and its
   items. A plan that describes the shape of the thing — a book's chapter
   outlines — is design, and a page.
3. A page cites the items that made it the way it is, and does not restate
   their arguments.

So a decision record is a cairn item, and the design page states what was
decided and links it
([0015](../../cairn/items/0015-where-an-architecture-decision-record-lives.md)).
[This page](#pages-items-and-the-line-between-them) does exactly that.

## A page is its path

Links address a page by its path, GitHub renders them, and every site generator
routes by them, so the path is the page's identity, with no number beside it
([0016](../../cairn/items/0016-a-page-is-addressed-by-its-path-not-by-a-number.md)).
The cost is that a rename breaks links; `loam mv` is what pays it.

## Nothing is required of a page

A plain Markdown file with a heading is a page, and everything loam needs from
it has a defined answer when nothing is written down
([0021](../../cairn/items/0021-does-a-page-need-frontmatter-at-all.md)). Over
the 69 files of the three folders loam was designed against, the title it reads
was right for 68 of them. Frontmatter overrides an answer; it is never the price
of being read.

## What loam will never do

loam reads and writes a directory of Markdown in one repository, and emits what
it knows as files and JSON
([0019](../../cairn/items/0019-declare-the-boundary-one-repository-no-server-no-html.md)).
It does not render HTML, run a server, keep a search index or embeddings on
disk, look across repositories, edit prose, or use the network — not even to
check that an external link still answers.

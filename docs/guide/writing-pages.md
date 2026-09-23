---
covers:
  - src/cmd/new.rs
  - src/cmd/mv.rs
  - src/cmd/supersede.rs
  - src/write.rs
order: 2
---

# Writing, moving and replacing pages

What a good page looks like to loam, and the three commands that change pages:
`new`, `mv` and `supersede`.

## A page

A page is a Markdown file under the docs root. It needs nothing else:

```markdown
# How replay finds a day

A recording is one file per day, and replay reads the index at its end before
anything else, so opening a year of history costs one seek per day.

## The index
```

loam reads the title from the first level-1 heading, the summary from the
paragraph directly under it, the kind from the directory, and the status as
`current`. Write the title and that first paragraph as if they were the only
two lines anybody will read, because in an index they are.

Frontmatter is for what the body cannot say:

```markdown
---
status: draft
summary: What replay reads first, and why that makes a year cheap to open.
---
```

The keys, and exactly what loam infers when one is absent, are in the
[specification](../../spec/README.md#4-keys).

## `loam new`

```console
$ loam new research "Markdown rendering in a forty-column pane"
docs/research/markdown-rendering-in-a-forty-column-pane.md
```

The page goes in its kind's directory, named from its title by cairn's rule,
with the kind's `template` under the title. `--draft` marks it a draft. It
never overwrites: if the name is taken, it says which page has it.

## `loam mv`

```console
$ loam mv docs/design/old-name.md docs/design/new-name.md
```

A page is addressed by its path, so a rename breaks every link to it. `mv`
moves the file and rewrites every relative link to it — and every relative
link out of it, whose paths change when it moves — in every Markdown file git
knows about, not only pages. Moving a directory moves everything in it.

`--dry-run` lists each change and makes none. A link that still works after
the move is left exactly as written. Each file is written whole or not at all,
so an interrupted move leaves every file either as it was or as it will be.

## `loam supersede`

```console
$ loam supersede docs/design/architecture.md docs/design/architecture-v2.md
docs/design/architecture.md is superseded by docs/design/architecture-v2.md
  updated docs/design/architecture-v2.md
  updated docs/design/architecture.md
```

The new page gains `supersedes`; the old one gains `superseded_by`, `status:
superseded`, and a line at the top for whoever arrives from an old link:

```markdown
> **Superseded** by [Architecture, again](architecture-v2.md).
```

The old page is kept. What it argued is still evidence, and links to it should
land somewhere that says where to go instead. `check` then points at every link
that still leads there. Running `supersede` twice changes nothing the second
time.

## What loam will not do to a page

It never reformats frontmatter. A key loam does not know is never parsed and
written back, so it cannot be reordered or requoted; a key loam sets is quoted
only when it would otherwise read back as something else. Line endings, a byte
order mark and every line loam was not asked to change are kept byte for byte.

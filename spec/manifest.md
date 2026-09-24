# The loam manifest

**Manifest version 1.** What `loam index --json` prints: a whole docs folder as
one JSON document, so that a site — or anything else — can be built from it
reading nothing but the manifest and the pages' own bodies. The page format is
the [specification](README.md); this is the output of one program that reads
it, specified so that programs downstream of loam can rely on it.
[`manifest.schema.json`](manifest.schema.json) states the same as a JSON
Schema, and loam's tests validate its output against it.

## Versioning

`manifest_version` is an integer. A change a program written against an earlier
version could misread — a key removed or renamed, a type or a meaning changed —
increments it. Adding a key does not, so a reader **must** ignore keys it does
not know. That is the promise cairn makes of its `--json` output. The version
is independent of the page format's (`format`), which the manifest also
states.

## Shape

```json
{
  "manifest_version": 1,
  "format": 1,
  "generator": "loam 0.3.0",
  "docs": "docs",
  "index": "docs/README.md",
  "cairn": "cairn/items",
  "kinds": [
    {"name": "guide", "dir": "docs/guide", "heading": "Guide",
     "description": "How do I do this? A task, start to finish, with real output.",
     "indexed": true}
  ],
  "sections": [
    {"role": "kind", "kind": "guide", "heading": "Guide",
     "description": "How do I do this? A task, start to finish, with real output.",
     "pages": ["docs/guide/adopting-a-docs-folder.md", "…"]}
  ],
  "pages": {
    "docs/guide/keeping-pages-true.md": {
      "path": "docs/guide/keeping-pages-true.md",
      "title": "Keeping pages true",   "title_from": "heading",
      "…": "every key of the page's reading",
      "body_line": 12,
      "lead": "Say what code a page describes, find the pages it has moved out from under, and mark them read again.",
      "headings": [
        {"line": 13, "level": 1, "text": "Keeping pages true", "slug": "keeping-pages-true"},
        {"line": 19, "level": 2, "text": "1. Say what a page covers", "slug": "1-say-what-a-page-covers"}
      ],
      "backlinks": [{"path": "docs/guide/checking-docs-in-ci.md", "line": 72}],
      "items": [38],
      "freshness": {"path": "docs/guide/keeping-pages-true.md", "state": "fresh", "…": "…"}
    }
  },
  "cairn_items": {
    "38": {"title": "What should a page say it covers?", "status": "done",
           "category": "done", "type": "question"}
  },
  "cairn_error": null,
  "freshness": {"notes": [], "error": null}
}
```

Every key below is always present. A value that can be absent is `null`, never
a missing key.

### The folder

- **`format`** is the page format version the folder is read under (spec §9).
- **`generator`** is the program and its version, for a person reading a
  report; a program **must not** branch on it.
- **`docs`** is the docs root and **`index`** the index page, both paths from the
  repository root (spec §7). **`cairn`** is the cairn items directory, or `null`.
- **`kinds`** are the kinds in the order `loam.toml` declares them. `dir` is the
  directory the kind claims, from the repository root; `heading` is its title in
  the index; `description` is `null` when it has none; `indexed` is false for a
  kind the index leaves out.

### Sections

**`sections`** is the index, in order: what `loam render` writes between its
markers, as data. Each has a `role` — `kind` for a kind's pages, `other` for
pages no declared kind holds, `superseded` for superseded pages — and `kind`,
the kind's name for a `kind` section or `null`. `heading` and `description`
are as the index shows them. `pages` are paths into `pages`, in the index's
order: `order` first, then title. A section with no pages is left out, and so is
the index page itself. A site's navigation is these sections.

### Pages

**`pages`** has one entry per page, keyed by its path, and each entry is the
page's **reading** exactly as [spec/corpus/README.md](corpus/README.md#the-shape-of-a-reading)
shapes it — `frontmatter`, `title`, `kind`, `status`, `summary`, each with its
`_from`, `order`, `covers`, `reviewed`, `generated`, `supersedes`,
`superseded_by`, `anchors`, `links` and `findings` — and then:

- **`path`**, the key again, for a program holding only the entry.
- **`body_line`**, the line of the file the body begins on (spec §3.1): the
  first line after the frontmatter, or 1 when there is none. A site takes the
  body from there.
- **`lead`**, the summary as the index shows it: a summary taken from the first
  paragraph, cut to its opening sentences within 160 bytes; one written in
  frontmatter, whole. Its links are as the page wrote them, relative to the
  page, where the index rewrites them to work from the index. `null` when
  there is no summary.
- **`headings`**, every heading in the body in order, with its `line`, its
  `level` (1–6), its `text` as it reads rendered (spec §6.3, before it is
  made a slug), and its `slug`, the anchor that reaches it, numbered as GitHub
  numbers repeats. A table of contents is these.
- **`backlinks`**, every link to this page from another page, as `path` and
  `line`, ordered by path and then line. The index is not counted, since it
  links every page, and nor is a page's link to itself. A link to a
  `#fragment` of the page counts.
- **`items`**, the numbers of the cairn items the page cites (spec §6.4),
  ascending, each once.
- **`freshness`**, whether the page is still true, as one page of `loam stale
  --json`: its `state` — `fresh`, `stale`, `updating`, `unknown` (no `covers`)
  or `generated` — the `baseline` it was compared from, and the changes since.
  `null` when there is no history: outside git, with `--no-history`, or when
  reading it failed.

### Cairn items

**`cairn_items`** is what [cairn](https://github.com/oddurs/cairn) says of each
item a page cites, keyed by its number as a string: its `title`, its
`status`, the `category` cairn groups that status in (`open`, `active`,
`done`, `dropped`), and its `type`. Only cited items are listed. It is `null`
when the project names no cairn directory, and when cairn could not be asked —
not installed, or failing — in which case **`cairn_error`** says why; otherwise
`cairn_error` is `null`. Whether a reference resolves does not depend on cairn
(spec §6.4): that is in each link.

### Freshness

**`freshness`** at the top says what was learned from history as a whole:
`notes`, the sentences `loam stale` prints once per run (a shallow clone, a
review made on a squashed branch); and `error`, why history could not be read,
or `null`. A site that marks stale pages **should** show a note or an error
rather than present every page as fresh.

## What a site reads

The manifest and each page's body, from `body_line` on. Nothing else: not the
frontmatter again, not `loam.toml`, not git. A link in a body is resolved as
`links` records it, and `class` says which are pages, which are cairn items and
which are files.

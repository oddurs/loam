---
status: draft
summary: Build a static site from loam's manifest and the pages' bodies, reading nothing else.
covers:
  - docs/cookbook/site.py
  - src/manifest.rs
  - spec/manifest.md
order: 7
---

# Building a site from the manifest

loam does not render HTML, and does not mean to. What it does is make a docs
folder something a site can be built from without reading the folder again:
`loam index --json` prints the [manifest](../../spec/manifest.md), and a site
needs nothing else but the pages' own bodies.

## The manifest

```sh
loam index --json > manifest.json
```

For each page, its reading — title, kind, status, summary, links — and what
loam knows beyond the page: where it sits in the index, its headings with the
anchors GitHub would give them, the pages that link to it, the cairn items it
cites, and whether it is still true. The index's sections come in order, so a
site's navigation is the index people already read. It is versioned apart from
loam, with a [JSON Schema](../../spec/manifest.schema.json) beside it; loam's
CI validates its own output, and the three real folders in `spec/corpus`,
against that schema on every push.

## A site from it

[`docs/cookbook/site.py`](../cookbook/site.py) is a static site in about a
hundred and fifty lines of Python. It reads the manifest and each page's body,
from `body_line` on, and nothing else — no frontmatter, no `loam.toml`, no git:

```console
$ loam -C ../poptop index --json > poptop.json
$ LOAM_ROOT=../poptop python3 docs/cookbook/site.py poptop.json site
built 36 page(s) into site
```

- **Navigation** is the manifest's `sections`, in the index's order.
- **A draft** is marked from its `status`; **a stale page** gets a banner from
  its `freshness`, saying how many commits changed what it covers.
- **On this page** is its `headings`, and each heading's anchor is the
  manifest's `slug`, so every `#fragment` written against GitHub still lands.
- **Linked from** is its `backlinks`.
- **A link to another page** — the manifest's `links` say which are pages —
  goes to that page's HTML.

On poptop, every one of the 1,559 links between the 36 pages it built, anchors
included, landed. A link to a file outside the docs, as to source code, is
left as written; a real site would point it at the repository.

The recipe needs Python Markdown (`pip install markdown`). Astro, mdBook or
Zola would do the rendering better; what carries over is the reading of the
manifest.

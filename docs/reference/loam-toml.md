---
covers:
  - src/config.rs
  - src/cmd/init.rs
order: 2
summary: Every key loam.toml can hold, and which of them a reader of the format must honour.
reviewed:
  commit: a6213f6f41f75d99ed3c7dd8d1d86e326ffab1c8
  date: 2026-09-23
---

# loam.toml

The configuration, at the root of the repository. It is [TOML](https://toml.io).
`loam init` writes one with a comment on each part.

The keys marked *format* are the ones the [specification](../../spec/README.md#71-configuration)
defines and any reader must honour. The rest are loam's own, and a reader of the
format may ignore them. An unknown key is warned about, never refused.

```toml
format = 1

[docs]
root = "docs"

[links]
cairn = "cairn/items"

[index]
path = "README.md"

[hooks]
after-change = "loam render --quiet"

[check.severity]
unclaimed = "error"

[[kind]]
name = "guide"
dir = "guide"
description = "How do I do this? A task, start to finish, with real output."

[[kind]]
name = "page"
dir = "."
title = "Other pages"
```

## Top level

| Key | | |
| --- | --- | --- |
| `format` | *format* | Required. The format version, `1`. loam refuses any other, naming both. |

## `[docs]`

| Key | | |
| --- | --- | --- |
| `root` | *format* | The docs root, relative to the repository. Default `docs`. |

Paths here and in `[links]` and `[[kind]]` may be written `./docs`, `docs/` or
`docs`: empty segments and `.` are ignored.

## `[links]`

| Key | | |
| --- | --- | --- |
| `cairn` | *format* | The directory of the project's [cairn](https://github.com/oddurs/cairn) items. A link into it is checked as a reference to an item, by its number. `init` reads it from `cairn.toml`. |

## `[index]`

| Key | | |
| --- | --- | --- |
| `path` | | The index, relative to the docs root. Default `README.md`. |

## `[hooks]`

| Key | | |
| --- | --- | --- |
| `after-change` | | A shell command run in the repository after `new`, `mv` or `supersede` changes pages. A command beginning `loam ` runs the loam that is running, not whichever is first on `$PATH`. |

## `[agents]`

| Key | | |
| --- | --- | --- |
| `status` | | `"draft"`: a page an agent writes with `new` starts as a draft. `"current"`, or absent: it does not. `init` writes `"draft"`. |

## `[check.severity]`

Each key is a [finding's code](findings.md); each value is `"error"`,
`"warning"` or `"ignore"`.

## `[[kind]]`

Kinds, in the order the index presents them. A page's kind is the one whose
directory holds it, the deepest claim winning; a kind whose `dir` is `.` takes
whatever no other kind claims. A page's frontmatter may name its kind instead.

| Key | | |
| --- | --- | --- |
| `name` | *format* | Required, and unique. |
| `dir` | *format* | The directory it claims, relative to the docs root. Default `.`. One kind per directory. |
| `title` | | Its heading in the index. Default: the name, capitalised. |
| `description` | | A paragraph under that heading. |
| `template` | | The body `new` writes under the title. If it begins with frontmatter, that frontmatter opens the page. |
| `index` | | `false` leaves the kind's pages out of the index. Default `true`. |
| `stale_after` | | How long a page of this kind stays true without a review, as `180d`, `26w`, `6m` or `1y`. For research, which goes stale because the world moves, not the code. |

---
order: 3
summary: What `--json` prints for check, list, show and search, for programs and agents.
---

# JSON output

`check`, `list`, `show` and `search` print JSON on standard output with
`--json`, and nothing else there. Keys may be added; none will be removed or
change meaning without a new major version of loam. Every path is relative to
the repository, with `/` between its parts.

## `list`

An array, one object per page, in path order:

```json
[
  {
    "path": "docs/guide/first-run.md",
    "title": "First run",
    "kind": "guide",
    "status": "current",
    "summary": "what is on screen, the three regions, the first four keys"
  }
]
```

`title`, `kind` and `summary` are `null` when the page has none. `status` is
always one of `draft`, `current`, `superseded`, or what the page wrote if it is
none of those.

## `search`

The same objects as `list`, best match first, each with three more keys:

| Key | |
| --- | --- |
| `rank` | `title`, `summary` or `body`: where every word was first found. |
| `line` | The line of the first match in the body, or `null`. |
| `text` | That line, trimmed, or `null`. |

Exit status is 1 when nothing matches, with `[]` printed.

## `show`

One object: the page's reading, exactly as the conformance corpus shapes it
([spec/corpus/README.md](../../spec/corpus/README.md#the-shape-of-a-reading)),
plus `path` and `body`, the text after the frontmatter.

## `check`

An array of findings, sorted by path, then line:

```json
[
  {
    "path": "docs/contributing.md",
    "line": 24,
    "code": "broken-link",
    "severity": "warning",
    "detail": "STYLEGUIDE.md",
    "message": "link to `STYLEGUIDE.md`, which does not exist; `styleguide.md` differs only in case"
  }
]
```

`code` is one of the [findings](findings.md). `detail` is the value concerned —
a link's destination as written, a key, a status — or `null`. `message` is for
a person, and its wording may change; match on `code`.

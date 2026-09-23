---
covers:
  - src/reading.rs
  - src/cmd/list.rs
  - src/cmd/show.rs
  - src/cmd/search.rs
  - src/cmd/stale.rs
  - src/cmd/check.rs
order: 3
summary: What `--json` prints for check, list, show, search and stale, for programs and agents.
---

# JSON output

`check`, `list`, `show`, `search` and `stale` print JSON on standard output with
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
plus `path`, `body`, the text after the frontmatter, and `freshness`, as one
page of [`stale`](#stale) gives it, or `null` outside a git repository.

## `stale`

Real output, from measure-of-the-world, one page of it:

```json
{
  "notes": [],
  "pages": [
    {
      "added": 22,
      "age": 259,
      "baseline": {
        "commit": "0aa28b0a963cf4aaa5ffd25b3b5cf70faf2ef7a0",
        "date": "2026-01-07",
        "how": "last-edit"
      },
      "commits": [
        {
          "commit": "cc486cc96e95bea21f46f3956aec23335665d9d1",
          "date": "2026-09-09",
          "subject": "Fix the build pipeline so failures are visible"
        }
      ],
      "path": "docs/releases.md",
      "patterns": [
        {
          "added": 22,
          "commits": 1,
          "latest": "Fix the build pipeline so failures are visible",
          "pattern": ".github/workflows",
          "removed": 0
        }
      ],
      "removed": 0,
      "stale_after": null,
      "state": "stale"
    }
  ]
}
```

| Key | |
| --- | --- |
| `state` | `stale`, `updating` (stale, but changed alongside its code), `fresh`, `unknown` (no `covers`) or `generated`. |
| `baseline.how` | What the page was compared from: `reviewed`, `introduced` (the commit that brought a review onto this branch, after a squash or rebase), `dated` (the last commit on or before the review's date), `last-edit` (never reviewed: the last commit that changed its body) or `new` (never committed). |
| `age` | Days from the baseline's date to today, or `null`. |
| `stale_after` | The kind's limit in days, when the page has outlived it; otherwise `null`. |
| `patterns` | Per including pattern of `covers`, what changed under it, most first. |
| `notes` | What `stale` would print once per run: how many pages fell back, and to what. |

Exit status is 1 when any page is `stale`.

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

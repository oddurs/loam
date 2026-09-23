# The corpus

Small repositories, and the reading a conforming reader must produce for each.
`spec/conformance.py` runs a reader over every one and compares.

- **`real/`** — three docs folders that existed before this format did, copied
  as they were. They are what a reader actually meets. See [NOTICE](NOTICE.md)
  for where each came from and the terms it is under.
- **`hostile/`** — cases written to break a reader: a byte order mark, CRLF,
  frontmatter that never closes, YAML 1.1 booleans, headings in fences, slugs
  with combining marks, links that escape the repository.

Each case is a directory holding a `loam.toml`, the files the case needs, and
`expected.json`. A case that has ever been here stays here. Changing what one
expects means the format changed (spec §9).

## The shape of a reading

A reader under test is run with the path of a case. It prints the reading as
JSON on standard output and exits 0, or refuses the repository (spec §9) and
exits 1. A case the reader must refuse expects `{"refused": true}`.

```json
{
  "format": 1,
  "pages": {
    "docs/guide/start.md": {
      "frontmatter": {"status": "draft"},
      "title": "Start here",        "title_from": "heading",
      "kind": "guide",              "kind_from": "directory",
      "status": "draft",            "status_from": "frontmatter",
      "summary": "The first page.", "summary_from": "paragraph",
      "supersedes": [],
      "superseded_by": [],
      "anchors": ["start-here", "install"],
      "links": [
        {"line": 7, "destination": "../design/why.md#costs", "class": "page",
         "target": "docs/design/why.md", "fragment": "costs"}
      ],
      "findings": [
        {"line": 7, "code": "broken-anchor", "detail": "../design/why.md#costs"}
      ]
    }
  }
}
```

- **`pages`** has one entry per page (spec §7.2), keyed by its path from the
  repository root.
- **`frontmatter`** is the mapping as parsed under the YAML 1.2 core schema, or
  `null` when the page has none or it is malformed.
- **`title`, `kind`, `status`, `summary`** are the values of spec §4 and §5, or
  `null`. Each has a **`_from`**: `frontmatter`; or where §5 found it —
  `heading`, `paragraph`, `directory`, `default`; or `null` when there is no
  value. A value of the wrong type is not a value (§4), so its `_from` is where
  §5 found one instead. A status of `superseded` inferred from `superseded_by`
  (§5.5) is `default`, like `current`.
- **`order`** is the integer, or `null` when absent or not an integer.
- **`covers`** is the list of patterns as written, a single string becoming a
  list of one, or `[]` when absent or malformed. **`reviewed`** is
  `{"commit": …, "date": …}`, or `null`. **`generated`** is the string, or
  `null`.
- **`supersedes`, `superseded_by`** are the values as written, a single string
  becoming a list of one, or `[]` when absent or malformed.
- **`anchors`** are the page's anchors in the order of spec §6.3: heading slugs,
  then HTML `id` and `name` values.
- **`links`** are every link spec §6.1 finds, in the order they appear in the
  file (by line, then left to right, a link inside another's text before it).
  - `destination` is as written, without angle brackets.
  - `class` is `external`, `outside` (a target above the repository), `page`,
    `cairn` (spec §6.4), or `file` (anything else in the repository, whether it
    exists or not).
  - `target` is the resolved path (spec §6.2), `""` for the repository itself,
    or `null` for `external` and `outside`.
  - `fragment` is the percent-decoded fragment, or `null` — always `null` for
    `external`, and reported as for any other link for `outside`.
  - A `cairn` link also has `item`, the number, and when the reference is stale,
    `current`, the path of the item's file now.
- **`findings`** are spec §6.5's, sorted by line, then code, then detail.
  `detail` is the value concerned: the destination as written for a link, the
  value as written for a supersession, the key for `malformed-key`, the value
  for `unknown-status` and `unknown-kind`, `null` for `untitled`, `unclaimed` and
  `invalid-encoding`, and for `malformed-frontmatter` exactly one of
  `no closing delimiter`, `not YAML` or `not a mapping`.

This shape is how the corpus states what it expects. It is not the manifest a
program built on the format will emit, which is specified separately and later.

---
covers:
  - src/main.rs
  - src/cmd
order: 1
summary: Every command, every option, and what each exit status means.
---

# Commands

Every command takes `-C DIR`, to run as if started in `DIR`, and `--no-hooks`,
to skip the [hooks](loam-toml.md#hooks) in `loam.toml`. A command finds
`loam.toml` in the working directory or the nearest directory above it. A path
naming a page may be given from the working directory, from the repository, or
from the docs root.

## Exit status

| Status | Meaning |
| --- | --- |
| 0 | Success. |
| 1 | The command ran and found what it reports: a failing finding (`check`), an index out of date (`render --check`), no match (`search`), a stale page (`stale`). |
| 2 | The command could not run: no `loam.toml`, a format this loam does not read, a page that is not there, a path that is taken. The message says which. |

## `init`

Adopt the docs folder here: write `loam.toml`, change no page, and print what
loam read from every page. Refuses to replace an existing `loam.toml`.

| Option | |
| --- | --- |
| `--docs DIR` | The docs folder. Found as `docs/` or `doc/` when not given. |
| `--preset NAME` | `diataxis` (guide, reference, design), `standard` (and research) or `minimal` (one kind), instead of a kind per directory. |
| `--force` | Replace an existing `loam.toml`. |

## `check`

Report broken links and anchors, malformed frontmatter, unclaimed pages and
the rest of the [findings](findings.md), each as `loam: path:line: message
[code]` on standard error. In GitHub Actions each is also printed as a workflow
command, so it appears as an annotation on the line it concerns.

| Option | |
| --- | --- |
| `-s`, `--strict` | Fail on warnings as well as errors. |
| `--render` | Also fail when the index is not what `render` would write. |
| `--stale` | Also report pages whose covered code changed since they were reviewed, as warnings on their `covers:` line. A page changed alongside its code is left out. |
| `--json` | Print the findings as [JSON](json-output.md#check) on standard output. |
| `-q`, `--quiet` | Print nothing when everything passes. |

## `render`

Write the index between its markers in `docs/README.md` (or wherever
[`index.path`](loam-toml.md#index) says), creating the file if there is none.
Refuses a file that exists without the markers, and says what to add.

| Option | |
| --- | --- |
| `--check` | Change nothing; exit 1 if the index is not what rendering would write. |
| `-q`, `--quiet` | Print nothing on success. |

## `new KIND TITLE`

Write a page in `KIND`'s directory, named from `TITLE`, with the title as its
heading and the kind's template under it, and print its path. Never overwrites.

| Option | |
| --- | --- |
| `--draft` | Give the page `status: draft`. |

## `list`

Pages, as a table of path, kind, status and title.

| Option | |
| --- | --- |
| `--kind KIND` | Only pages of this kind. |
| `--status STATUS` | Only pages with this status. |
| `--json` | Print [JSON](json-output.md#list). |

## `show PAGE`

What loam read from one page — title, kind, status and summary, each with where
it came from, its supersession, its links and findings — then the page itself.

| Option | |
| --- | --- |
| `--json` | Print the page's reading and its body as [JSON](json-output.md#show). |

## `search WORDS…`

Pages containing every word, in any case: those with every word in the title
first, then in the title and summary, then anywhere. Each hit shows the first
matching line. A scan of the files every time; nothing is kept on disk.

| Option | |
| --- | --- |
| `--kind KIND` | Only pages of this kind. |
| `--json` | Print [JSON](json-output.md#search). |

## `mv OLD NEW`

Move a page, a file or a directory, and rewrite every relative link to it and
from it in every Markdown file git knows about. Into `NEW` if it is an existing
directory. Never overwrites.

| Option | |
| --- | --- |
| `-n`, `--dry-run` | Print each change, and change nothing. |

## `stale`

Pages whose covered code changed since they were last reviewed, most changed
first, each with the patterns, commits and lines behind it; then pages being
updated alongside their code; then a count of fresh, unknown (no `covers`) and
generated pages. A page never reviewed is compared from the last commit that
changed its body. Needs git.

| Option | |
| --- | --- |
| `--all` | Also list the fresh, unknown and generated pages by name. |
| `--at REV` | Judge as the repository was at `REV`, with the pages as they are now: for replaying history. |
| `--json` | Print [JSON](json-output.md#stale). |

## `review PAGE…`

Record that each page was read against the code at `HEAD`: sets `reviewed` to
that commit and today's date, keeping anything else under `reviewed`. Warns,
without refusing, when a covered file has uncommitted changes.

| Option | |
| --- | --- |
| `--note TEXT` | Add a line to a `## Review log` at the end of the page. |

## `supersede OLD NEW`

Record that `NEW` replaces `OLD`, on both pages: `supersedes` on the new one;
`superseded_by`, `status: superseded` and a notice on the old one. Running it
again changes nothing.

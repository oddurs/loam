---
covers:
  - src/main.rs
  - src/cmd
order: 1
summary: Every command, every option, and what each exit status means.
reviewed:
  commit: f6c73a6296c30e690671aa74fe29b0b9155d56a8
  date: 2026-09-23
---

# Commands

Every command takes `-C DIR`, to run as if started in `DIR`, and `--no-hooks`,
to skip the [hooks](loam-toml.md#hooks) in `loam.toml`. A command finds
`loam.toml` in the working directory or the nearest directory above it. A path
naming a page may be given from the working directory, from the repository, or
from the docs root; one that is not a page is answered with the page it most
likely meant.

## Exit status

| Status | Meaning |
| --- | --- |
| 0 | Success. |
| 1 | The command ran and found what it reports: a failing finding (`check`), an index out of date (`render --check`), no match (`search`), a stale page or one to reread (`stale`), a page like the one `new` was asked for. |
| 2 | The command could not run: no `loam.toml`, a format this loam does not read, a page that is not there, a path that is taken. The message says which. |

## `init`

Adopt the docs folder here: write `loam.toml`, change no page, and print what
loam read from every page — or, past forty pages, a count for each kind and
only the pages with something to fix. The index is the folder's `README.md`,
or its `index.md` if that is the page a site generator opens with; a new
`README.md` only when there is neither. Refuses to replace an existing
`loam.toml`.

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

## `index`

Print the index — the block `render` writes between the markers — without
writing it. With `--json`, print the **manifest** instead: every page's
reading, the index's sections in order, each page's headings with their
anchors, its backlinks, the cairn items it cites and whether it is still true.
The two are made from the same data, so they cannot disagree. The manifest is
specified in [spec/manifest.md](../../spec/manifest.md), versioned, with a JSON
Schema beside it: a site can be built from it and the pages' bodies alone.

| Option | |
| --- | --- |
| `--json` | Print the manifest. |
| `--no-history` | Leave out freshness, which reads git's history. |

## `new KIND TITLE`

Write a page in `KIND`'s directory, named from `TITLE`, with the title as its
heading and the kind's template under it, and print its path. Never overwrites.

A page an agent writes starts as a draft when `loam.toml` says `[agents] status =
"draft"`. `new` refuses, with exit status 1, when a page of the same kind looks
like the one asked for, and names it; a person at a terminal is asked instead.

| Option | |
| --- | --- |
| `--draft` | Give the page `status: draft`. |
| `--agent NAME` | Write as this agent; also read from `LOAM_AGENT`, `CAIRN_AGENT`, `AI_AGENT`, or Claude Code's `CLAUDECODE`. |
| `--anyway` | Write it even though a page like it exists. |

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

Also lists the cairn items the page cites, each with the title and status
cairn gives it — or, with cairn not installed, says they are unchecked — and
the pages that link to this one, the index apart.

## `search WORDS…`

Pages containing every word, in any case and any order — a quoted phrase is
taken as its words. The best match is first: a word in the title counts most,
then in the summary or the path, then each time it appears in the body. Each
hit shows the line with the most of the words. The index is left out, since it
repeats every title. When no page has every word, the pages with the most of
them are named on standard error, and the exit status is still 1. A scan of
the files every time; nothing is kept on disk.

| Option | |
| --- | --- |
| `--kind KIND` | Only pages of this kind. |
| `--json` | Print [JSON](json-output.md#search). |

## `mv OLD NEW`

Move a page, a file or a directory, and rewrite every relative link to it and
from it in every Markdown file git knows about. Into `NEW` if it is an existing
directory. Never overwrites, and never moves `loam.toml` itself.

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
| `--working-tree` | Instead, the pages covering files with uncommitted changes, whatever their history: what to reread before stopping. |
| `--since REV` | Instead, the pages covering files changed on this branch since it left `REV`, not counting what changed on `REV` meanwhile. With `--working-tree`, both. |
| `--json` | Print [JSON](json-output.md#stale). |

## `review PAGE…`

Record that each page was read against the code at `HEAD`: sets `reviewed` to
that commit and today's date, keeping anything else under `reviewed`. Warns,
without refusing, when a covered file has uncommitted changes.

| Option | |
| --- | --- |
| `--note TEXT` | Add a line to a `## Review log` at the end of the page. |

## `set PAGE KEY=VALUE…`

Change a page's frontmatter: `key=value` sets, `key=` removes, `key+=value` and
`key-=value` add to and take from a list of strings — a list holding anything
else is refused rather than rewritten. No other byte of the page changes, and
a key written twice is changed where YAML reads it, the last time.
The format's own keys are held to their types — a `status` of draft, current or
superseded, an `order` that is a whole number, a declared `kind` — and
`reviewed` is left to `review`. Any other key is written as typed: `weight=3` is
a number.

## `context PATH…`

The pages that matter for these files: those whose `covers` match, most
specific first; then those that mention a path; then those the covering pages
link to — each with whether it is still true, or why that could not be read.
Whole pages while they fit the budget, then their summaries, then the rest
named as left out. Nothing printed goes over the budget, its first line
included.

| Option | |
| --- | --- |
| `--budget BYTES` | The most to print: `8000`, or `8k` for 8192, and at least 200. Default `8k`, about 2,000 tokens. |
| `--json` | Print [JSON](json-output.md#context). |

## `agent`

Print the instructions an agent needs — where each kind of page lives, and the
loop of search, new, set, supersede, stale and review — generated from
`loam.toml`.

| Option | |
| --- | --- |
| `-w`, `--write FILE` | Insert the block in `FILE`, as `AGENTS.md`, or replace it where it already is, leaving the rest of the file alone. The markers count only alone on their lines and outside code; a file that is not UTF-8 is refused, not written over. |

## `supersede OLD NEW`

Record that `NEW` replaces `OLD`, on both pages: `supersedes` on the new one;
`superseded_by`, `status: superseded` and a notice on the old one. Running it
again changes nothing.

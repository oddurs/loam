---
covers:
  - src/cmd/check.rs
  - action.yml
  - install.sh
order: 4
summary: One command that fails a build when a link breaks, a page cannot be read, or the index has drifted.
reviewed:
  commit: 94f9e51559c2c49eab556c5924f5727389f7ca2c
  date: 2026-09-23
---

# Checking docs in CI

One command fails a build when a link breaks, a page cannot be read, or the
index has drifted from the pages:

```sh
loam check --strict --render
```

`--strict` fails on warnings as well as errors; a broken link is a warning,
because on a working copy it is often a page not written yet. `--render` also
fails when the index is not what `loam render` would write. Exit status is 0
when nothing fails, 1 when something does, and 2 when the check could not run
— no `loam.toml`, or one from a format this loam does not read.

For a program rather than a log, `--json` prints every finding with its path,
line, code, severity and message ([JSON output](../reference/json-output.md)).

## In GitHub Actions

```yaml
- uses: actions/checkout@v7
  with:
    fetch-depth: 0          # for --stale; the check alone needs no history
- uses: oddurs/loam@v0.3.0
```

The action installs that release of loam — a binary, no Rust — and runs
`loam check --strict --render`; each finding becomes an annotation on the pull
request's line. `args` runs something else, and an empty `args` only installs
it, for steps of your own:

```yaml
- uses: oddurs/loam@v0.3.0
  with:
    args: check --stale
```

Anywhere else, [`install.sh`](../../install.sh) does the same for Linux and
macOS: `curl -fsSL https://raw.githubusercontent.com/oddurs/loam/main/install.sh | sh`.

## Making a finding fatal, or quiet

`loam.toml` can move any finding's severity:

```toml
[check.severity]
unclaimed = "error"      # every page must be in a kind's directory
untitled = "ignore"      # this folder has pages without headings, on purpose
```

The codes are listed in [Findings](../reference/findings.md).

## loam's own CI

This repository runs `make check`, which includes `loam check --strict
--render` on these pages, beside the tests and the conformance corpus, and
then `loam check --stale`, whose warnings annotate a pull request without
failing it. For the whole of staleness, see
[Keeping pages true](keeping-pages-true.md).

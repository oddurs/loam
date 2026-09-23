# Releases

## What a version means

Read for a book, not a library.

| | |
|---|---|
| **MAJOR** | A new edition. Chapter numbering may change; page references break. |
| **MINOR** | New or substantially rewritten chapters and figures. |
| **PATCH** | Corrections, typography, engine work with no effect on the text. |

A pre-release suffix (`1.0.0-rc.1`) marks it as a prerelease on GitHub
automatically.

## Cutting one

```sh
node engine/cli.ts release minor
```

Which does, in order:

1. **Preflight** — refuses a dirty tree or a branch other than `main`. A release
   must be reproducible from its tag.
2. **Check** — types, tests, figures, captions, citations. A release that does
   not pass its own checks is not a release.
3. **Promote the changelog** — moves `## [Unreleased]` into a dated version
   section. Refuses if it is empty: write down what changed before tagging it.
4. **Bump** `package.json`, commit as `chore(release): vX.Y.Z`, and create an
   annotated tag carrying the notes.

It deliberately **does not push**. The tag is the trigger, so pushing it should
be a conscious act:

```sh
git push origin main v0.2.0
```

## What CI then does

`.github/workflows/release.yml` fires on `v*`, re-runs `check` from a clean
checkout, builds, names the artefact after the tag, extracts the release body
from `CHANGELOG.md` via `engine/cli.ts changelog`, and publishes a GitHub
release with the PDF attached.

Release notes are written once, by a human, in the changelog. Nothing generates
prose.

## The version stamp

Every build writes `build/version.typ` and the colophon prints it. A draft says
so in the book itself:

> This copy was built on 2026-09-12 from commit `a1b2c3d4` with uncommitted
> changes. It is a draft, not a release: figure numbers, section numbering and
> page references may all move before the next tagged version.

which is what stops an untraceable PDF circulating for six months.

---
id: 16
title: A page is addressed by its path, not by a number
type: decision
status: done
milestone: v0.0
depends_on:
- 13
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
pillar:
- format
effort: s
---

## Context

cairn gives every item a number because titles change and a reference must not.
Pages are different: people and Markdown link them by relative path, GitHub
renders those links, and every site generator routes by them.

## Options

1. **Path is identity.** Renaming is a move, and the program rewrites every link.
2. **A number or stable slug in frontmatter**, with paths free to change.
3. **Path, with an optional `aliases:`** list of former paths, so an old link
   still resolves and a site can emit redirects.

## Recommendation

**Option 1, with `mv` doing the rewriting.** A number nobody links by is a
second identity to keep in step with the first. The cost of a path identity —
links rotting on rename — is exactly the cost `loam mv` exists to remove, the
same way `cairn set 1 key=v2.0` rewrites every reference.

Option 3 stays open for when there is a published site with inbound links to
protect. That is a v0.4 concern and should be decided then, not guessed now.

## Consequences

- Renaming a file by hand, outside loam, breaks links. `check` reports it; it
  cannot know where the page went.
- The manifest keys pages by path.

## What it rules out

- An `id`, number or slug key in a page's frontmatter as its identity.
- Numbering pages in their file names as a requirement. A project may still
  name `0001-pdf-not-web.md` that way; the digits mean nothing to loam.
- `aliases:` in version 1 of the format. It is reserved (spec §4.3), not
  defined, so adding it later costs no format number.

## Acceptance criteria

- [x] The decision is stated in one sentence under Recommendation
- [x] What it rules out is written down

## 2026-09-22

Decided as recommended. Spec §2 makes paths exact and case-sensitive, because measure-of-the-world's docs link STYLEGUIDE.md and BUILD.md to files named styleguide.md and build.md: they work on a case-insensitive Mac and are broken on GitHub. A path identity has to be compared the way GitHub compares it.

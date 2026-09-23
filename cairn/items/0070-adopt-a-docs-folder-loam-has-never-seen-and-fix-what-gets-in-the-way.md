---
id: 70
title: Adopt a docs folder loam has never seen, and fix what gets in the way
type: chore
status: done
milestone: v0.4
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
---

## Why

Every folder loam was designed against is one its author wrote. The first test
of whether it helps anyone else is to adopt a folder written by other people,
for another tool, and use every command on it as a newcomer would. uv's
(astral-sh/uv, 81 pages, built with mkdocs) is the one used.

## Acceptance criteria

- [x] `init`, `check`, `search`, `new`, `show`, `list`, `mv`, `set` and `context` have each been run on it
- [x] Every friction found is fixed with a test, or recorded as its own item
- [x] `make check` passes

## 2026-09-23

Found on uv, and fixed: (1) 55 of 57 check warnings were links to cli.md and settings.md, which uv's .gitignore excludes because they are generated at build time; a broken link to a file git ignores is now ignored-link, quiet by default, with a note giving the count — 3 warnings remain, one a real broken anchor in uv. (2) init pointed the index at a new docs/README.md beside mkdocs's index.md, and render created it; init now uses index.md when that is what exists. (3) init printed 81 lines; past 40 pages it counts per kind and lists only pages with something to fix, counting findings as check reports them. (4) search "docker cache" found nothing: a quoted phrase was one word. It now splits words, scores title > summary/path > body occurrences, leaves out the index, and names the closest pages when none has every word. (5) the index copied a summary's #fragment link unchanged, broken from the index; it now points at the page. (6) new wrote 'Using uv in Docker containers' beside 'Using uv in Docker'. (7) show cache.md said only 'not a page'; it now suggests docs/concepts/cache.md; list/search --kind guides printed nothing instead of naming the kinds. (8) mv rewrote ./cache.md as caching.md; the ./ is kept. (9) set covers+= a path matching nothing said nothing until check.

## 2026-09-23

The duplicate check, measured by taking each page out and asking new for its title again: before, uv raised 6 non-duplicates in 67 and missed the Docker one. Now: narrows() catches a title that is an existing title plus words (all words, common ones included, so 'Appendix outlines' does not narrow 'Appendix prompts'); likeness() needs one shared word in the other page's title, since a summary alone mentions too much; and a landing page (index.md or README.md with other pages under its directory) is never offered. Result: 0 in the corpus's 64 titles, 3 close siblings in uv's 67, the Docker duplicate caught. Shared words are shown as spelled in the titles, not as stems ('docker', not 'dock').

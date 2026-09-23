---
id: 22
title: How does a page link to a cairn item?
type: question
status: done
milestone: v0.0
assignee: Oddur Sigurdsson
depends_on:
- 14
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
pillar:
- links
effort: s
---

## Question

What does a reference from a page to a cairn item look like in the file, so that
it renders on GitHub, survives the item being retitled, and can be checked?

## Why it has to be answered first

cairn renames an item's file when its title changes: `0072-relationships-...md`
becomes something else. A plain relative link rots the first time a title is
improved, which on this author's backlogs is often.

## Options

1. **A relative link to the file.** Renders everywhere. Rots on retitle.
2. **A scheme**, `[0072](cairn:72)`. Survives retitling. GitHub renders it as a
   dead link.
3. **A relative link whose target loam repairs**: the filename's numeric prefix
   is the identity, so `check --fix` rewrites
   `../cairn/items/0072-old-title.md` to the current filename. Renders
   everywhere, survives retitling for anyone who runs check.
4. **Ask cairn to keep the link stable** — a redirect file, or no slug in the
   filename. A change to cairn for loam's benefit; a poor trade.

## What would settle it

Write a page with each form, push it, and look at it on GitHub and in an Astro
build. Then retitle the item and see what each does.

## Answer

**Option 3: an ordinary relative link to the item's file, whose identity is the
number the file name begins with.** A reader recognises it by where it points —
a file directly in the project's cairn items directory, named in `loam.toml` as
`links.cairn` — and resolves it by number. A link whose number exists under
another file name is *stale*, a finding a writer repairs by rewriting the file
name; one whose number does not exist is broken. Spec §6.4.

### The evidence

The same three links were rendered by GitHub's Markdown API (`gh api markdown`,
GFM mode, the renderer github.com uses for a `.md` file) and by Astro's own
Markdown pipeline (`@astrojs/markdown-remark` 7.3.1), on 2026-09-22:

```markdown
[scheme](cairn:72)
[relative](../cairn/items/0072-old-title.md)
```

| | GitHub | Astro |
| --- | --- | --- |
| `cairn:72` | **the link is removed**: the text is left, with no `<a>` at all | `<a href="cairn:72">`: a link a browser cannot follow |
| relative | a working link | `<a href="../cairn/items/0072-old-title.md">`, unchanged |

So a scheme (option 2) is never a link anywhere a page is read, before or after
a retitle; it survives renaming by never having worked. The relative link works
on GitHub, which is where these pages are read most, and after a retitle it
fails in the same way any renamed file does — which `check` can see, and, unlike
any other broken link, can repair without asking, because the number in the old
name is still right.

In a site, a relative link to a cairn item points at a file the site may not
serve. That is the site's to map, from the manifest (0051) and cairn's own
answers (0054), and it is the same whichever form the page uses.

Option 4 was not tried: it asks cairn to change for loam, and option 3 needs
nothing from cairn but its file names, which cairn's specification says should
be `<id>-<slug>.md` (cairn spec §5).

One limit, recorded in spec §6.4: cairn's spec also says nothing may depend on
an item's file name, and a project may render its ids with a prefix
(`MP-1002`). Version 1 resolves by the leading digits of the name, so it covers
the default naming and not a prefixed one, and trusts the name where an item's
`id` disagrees with it. Resolving by asking cairn (0054) removes both limits
without changing how a page writes the link.

## Acceptance criteria

- [x] Answer written, with the evidence that settled it

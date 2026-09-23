---
covers:
  - src/index.rs
order: 2
summary: Why the index is generated between markers in a file people also write in, and what it lists.
---

# The index

A docs folder's `README.md` usually holds a table of every page, kept by hand.
It is right on the day it is written and drifts from then on, the way a
hand-kept `ROADMAP.md` did before `cairn render` replaced it. loam generates
the table instead ([0030](../../cairn/items/0030-generate-the-docs-index-instead-of-keeping-it-by-hand.md)).

## Between markers, not from a header file

cairn renders a whole `ROADMAP.md` from its items and a header kept in a
separate file. An index is different: it is a page people also write in.
poptop's opens with what its three kinds of page are for, and ends with links
out of the docs folder. So loam owns only what lies between two lines,

```markdown
<!-- loam:index:begin -->
<!-- loam:index:end -->
```

and the rest of the file is the project's, kept exactly as written. That is
cairn's header and footer, kept in the one file a reader actually opens.

## What it lists

A section per kind, in the order `loam.toml` declares them, headed by the
kind's `title` and `description`. In each, a row per page: its title as a link,
marked *(draft)* when it is one, and its summary. Rows are in `order`, then by
title ([0064](../../cairn/items/0064-order-a-page-s-place-among-its-kind.md)),
because a guide's first page is first on purpose. A summary's own links are
rewritten to work from the index.

Pages that no declared kind holds are listed under *Other*, rather than hidden,
because `check` is already saying something about them. Superseded pages go in
a last section, each with what replaced it. A kind with `index = false` is left
out, for pages a project lists in its own words.

## Adopted, not written from nothing

The criterion for this was that poptop's generated index carry everything its
hand-kept one did. It does: every page, every row's note (moved into the page
as `summary`), the order, and the prose above the Design table (moved into the
kind's `description`). Two rows show the page's full title where the hand-kept
table used a shorter label; [0065](../../cairn/items/0065-a-label-shorter-than-the-title-for-listings.md)
asks whether that needs a key of its own.

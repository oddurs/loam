# Wrapped links

Prose wrapped at a margin puts [the text of a
link](other.md) on two lines, and [a longer one that
runs over three lines of the
paragraph](other.md#a-section "with a title") as well.

A code span can wrap too: `[not a
link](missing.md)` is inside one.

The destination cannot: [text](other.md
"title on the next line") is not a link, as CommonMark would have it.

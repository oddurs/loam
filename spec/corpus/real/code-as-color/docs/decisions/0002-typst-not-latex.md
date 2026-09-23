# Typst rather than LaTeX

**Status:** accepted

## Context

Both were installed. LaTeX is the default for technical books.

## Decision

Typst compiles the whole book in 400 ms against LaTeX's many seconds, reads SVG with embedded raster natively — no Inkscape step — and its scripting is a real language, which is what makes the query-based running heads and the generated matrix appendix possible at all.

## Consequences

A smaller ecosystem and fewer packages. We write our own page master, which we wanted to do regardless.

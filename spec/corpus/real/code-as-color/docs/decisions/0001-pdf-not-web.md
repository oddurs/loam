# PDF, not an interactive web book

**Status:** accepted

## Context

The subject argues for interactivity: the reader should be able to drag a chroma slider and watch the gamut boundary move. We chose print anyway.

## Decision

Typography and permanence. A PDF set in a real page master is a better reading experience for a 300-page argument than any web page, it does not bitrot, and it can be printed. Interactivity is a genuine loss; the repository is the compromise — every figure is executable, and disagreeing with one means editing it and rebuilding.

## Consequences

The book cannot show most of its own central diagram, and says so on the page rather than pretending otherwise. A reader who wants to explore must clone the repo.

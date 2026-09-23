# ET Book, with a documented fallback

**Status:** accepted

## Context

ET Book is Tufte's own Bembo digitisation, MIT licensed. It maps 229 codepoints and has no Greek, no combining macron and no math relations.

## Decision

Its old-style figures are the right choice for a book this full of numbers in running prose, and it is the face the layout is modelled on. The coverage gap is handled rather than avoided: variables go in math mode, and engine/draw/fonts.ts parses the cmap to pick a family per string, because the SVG renderer resolves one family per text element.

## Consequences

A second face appears for Greek and combining marks. check fails on any label the body face cannot draw, so the failure mode is a build error rather than a tofu box in a printed book.

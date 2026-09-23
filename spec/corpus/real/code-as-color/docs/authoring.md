# Authoring

## A chapter

One outline file per chapter, in `outline/chNN.outline.ts`, the same shape as a
figure — a typed declaration the engine discovers, validates and emits from.

```ts
export default defineChapter({
  id: "ch07",
  number: 7,
  part: "II",
  title: "Perceptual Spaces, and the Cylinders on Top of Them",
  status: "outline",
  epigraph: { text: "…", source: "Isaac Newton, Opticks, 1704", unverified: true },
  lead: `XYZ is linear and useless for judgement; RGB is device-bound. …`,
  sections: [
    {
      title: "CIELAB",
      argument: `The cube root as a compressive nonlinearity, the linear segment
        near black and why it is there, …`,
      words: 1600,
      figures: ["lightness-comparison"],
      sources: ["cie-15-colorimetry"],
    },
  ],
  exercises: [{ prompt: "…", kind: "code" }],
});
```

```sh
node engine/cli.ts outline            # the whole book, as a table
node engine/cli.ts outline show ch07  # one chapter in full
node engine/cli.ts outline check      # validate without writing
node engine/cli.ts outline sync       # regenerate book/parts and book/main.typ
node engine/cli.ts outline docs       # regenerate docs/outline.md
```

### Status, and when the engine stops touching a chapter

| `status` | Meaning |
|---|---|
| `outline` | Planned. `sync` generates `book/parts/chNN.typ` from this declaration. |
| `drafting` | You are writing prose into `book/parts/`. The engine leaves the file alone. |
| `written` | Finished. Left alone entirely. |

Move a chapter to `drafting` **before** you write into its `.typ` file, or the
next `sync` will overwrite you. The outline declaration stays useful either way:
it still supplies the chapter's place in the running order, its word budget, and
which figures it owns — so a later chapter keeps cross-referencing rather than
reprinting them.

### What the emitter does for you

- Escapes the characters where Typst markup and TypeScript string literals
  disagree. A bare `L*` in prose is an unclosed emphasis delimiter that swallows
  the rest of the paragraph; it becomes `$L^*$`. `@` and `#` are escaped.
- Strips the reassurance adverbs listed in [`voice.md`](voice.md), because a
  style rule nobody enforces is a style rule nobody follows.
- Places each figure exactly once, in the first section that needs it, and
  cross-references it everywhere after.
- Refuses to write a chapter whose generated markup has unbalanced emphasis.

### What `outline check` catches

A section that cites a source not in the research store (error), plans a figure
nobody has drawn (warning), states a topic instead of an argument (error), is
secretly a chapter at 2,400+ words (warning), or duplicates a title (error).
Plus numbering gaps, interrupted parts, a part starved of under a twelfth of the
book, and any epigraph still marked `unverified`.

## A figure

```sh
node engine/cli.ts new figure spectral-locus-labels --chapter ch04
```

A figure is a pure function from nothing to an SVG document:

```ts
export default defineFigure({
  id: "cie-1931-chromaticity",
  chapter: "ch04",
  title: "The CIE 1931 chromaticity diagram",
  caption: `Every colour a human eye can be shown…`,
  claim: "The gamut of any display is a small triangle inside the visible set.",
  sources: ["cie-15-colorimetry"],
  placement: "wide",
  render() { /* … */ return plot.document(); },
});
```

Rules the engine enforces:

- The rendered width must equal the placement width. Use `theme.widths.wide`,
  not a number you liked the look of.
- Any label containing a glyph ET Book lacks must go through `familyFor()`.
- A figure is placed **once** with `#fig("id")`; everywhere else uses
  `#figref("id")`.
- Captions may not contain `#4%` (Typst code) or a bare `L*` (emphasis).

Then:

```sh
node engine/cli.ts figures <id> --verbose
node engine/cli.ts proof --figures --ppi 120
```

## A source

```sh
node engine/cli.ts cite add 10.1002/col.20070 --tier primary
node engine/cli.ts cite add https://bottosson.github.io/posts/oklab/ --key ottosson-oklab
```

Then open the record and write the two things the fetcher cannot: **Why it
matters**, and at least one distilled claim with a locator and a confidence.
See [`research.md`](research.md).

## The loop

```sh
node engine/cli.ts watch    # figures rebuild on change; Typst recompiles
```

Watch re-renders in a child process, because ES module caches cannot be evicted
and an in-process rebuild would serve you the figure you had at startup.

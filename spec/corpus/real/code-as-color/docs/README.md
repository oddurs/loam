# Documentation

Context for *Three Numbers* — the book, and the engine that builds it.

| | |
|---|---|
| [`concept.md`](concept.md) | What the book argues, who it is for, and what it is not |
| [`outline.md`](outline.md) | The structure: five parts, nineteen chapters, and why they are in that order |
| [`voice.md`](voice.md) | The style guide, including the tics we have agreed to avoid |
| [`design.md`](design.md) | Typography and the diagram grammar — the Tufte decisions and their costs |
| [`architecture.md`](architecture.md) | How the engine fits together |
| [`authoring.md`](authoring.md) | How to add a chapter, a figure, a source |
| [`research.md`](research.md) | The citation and claim system |
| [`writing-process.md`](writing-process.md) | The loop each chapter goes through, and which gates are machine-checked |
| [`releases.md`](releases.md) | How a release is cut and what a version number means |
| [`decisions/`](decisions/) | Architecture decision records — the calls we made and what they cost |

## The short version

The book's thesis is that colour is a lossy projection of an
infinite-dimensional signal onto three numbers, and that almost every practical
difficulty in the subject is a consequence of that compression.

The engine's commitment is that **nothing in the book is transcribed**: every
figure, matrix and number is computed here from published colorimetric data, and
the build fails if a citation points at nothing.

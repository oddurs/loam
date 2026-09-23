# Voice

## The register

Precise, concrete, and willing to name the exact failure. Numbers over
adjectives. Derivations in front of the reader. Where the book corrects a common
belief, it corrects it *once* and moves on.

## Tics we have agreed to avoid

Measured across the outline prose, and cut:

| Tic | Why |
|---|---|
| `honestly`, `genuinely`, `actually`, `precisely` | Reassurance words. A confident book does not keep announcing its own rigour; it just has it. The generator strips these — see `deReassure` in `engine/outline/emit.ts`. |
| Manufactured aphoristic epigraphs | Nineteen of identical shape (confident debunk + metaphor) is a mannerism, not a voice. Use real quotes where the field has them — Newton, Maxwell, Albers — and no epigraph where it does not. |
| "X and its Lies" | The debunking posture is load-bearing for thirty pages and exhausting for three hundred, and it casts the reader as a dupe being rescued. One such title in the book, not three. |
| Heavy em-dash use | Was running at 5.6 per thousand words. Aim for half that. |

## Things to keep

- Concrete measured numbers in the prose: "2.0% at V = 0.60", not "a small error".
- Naming what a thing *is* before saying what is wrong with it.
- Admitting the limits of a model in the same breath as using it.
- British spelling: *colour*, *normalise*, *analogue*. American in code and in
  quoted standards (`color`, `gray`).

## Citations

Every non-obvious claim traces to a distilled claim in `research/sources/`.
If a statement is not traceable there, it is either common knowledge or it is
unsupported, and `node engine/cli.ts check` will say which.

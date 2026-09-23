# loam

*Working name — see [`0020`](cairn/items/0020-settle-the-name.md).*

A repository's written context — research, design, reference, guides — kept as
Markdown under a schema, the way [cairn](https://github.com/oddurs/cairn) keeps
its backlog.

Nothing is built yet. The plan is in [`ROADMAP.md`](ROADMAP.md), and the
arguments behind it are in `cairn/items`.

## How it fits

| | holds | unit | the question it answers |
| --- | --- | --- | --- |
| **cairn** | intent: why something changed | a numbered item that closes | what is ready? |
| **loam** | knowledge: how it is now | a page at a path that is kept true | is this still true? |
| **harrow** | a view onto cairn | — | what needs me? |

A question is a cairn item, and its answer, if it outlives the question, is a
page. A plan with steps is a milestone. A page cites the items that made it the
way it is.

## Start here

```sh
cairn list --view decide    # the decisions v0.0 is waiting on
cairn next                  # what can start
cairn roadmap
```

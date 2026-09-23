# loam

*Working name — see [`0020`](cairn/items/0020-settle-the-name.md).*

A repository's written context — research, design, reference, guides — kept as
Markdown under a schema, the way [cairn](https://github.com/oddurs/cairn) keeps
its backlog.

No program is built yet. What exists is the format: [`spec/README.md`](spec/README.md)
says what a page is, how its title and summary are read when nobody wrote them
down, and which links are broken, precisely enough to implement a reader from.
[`spec/reader.py`](spec/reader.py) is one, and [`spec/corpus`](spec/corpus)
holds three real docs folders and the cases written to break it. The plan is in
[`ROADMAP.md`](ROADMAP.md), and the arguments behind it are in `cairn/items`.

```sh
python3 spec/conformance.py     # the reader against the corpus
python3 spec/shared.py          # the text shared with cairn's spec, still shared
python3 spec/reader.py spec/corpus/real/poptop | less
```

## How it fits

| | holds | unit | the question it answers |
| --- | --- | --- | --- |
| **cairn** | intent: why something changed | a numbered item that closes | what is ready? |
| **loam** | knowledge: how it is now | a page at a path that is kept true | is this still true? |
| **harrow** | a view onto cairn | — | what needs me? |

A question is a cairn item, and its answer, if it outlives the question, is a
page. A plan with steps is a milestone. A page cites the items that made it the
way it is.

## What loam is not

loam reads and writes a directory of Markdown in one repository, and emits what
it knows as files and JSON. Nothing else. It will not:

- **render HTML.** Astro, mdBook, Zola and the rest do that well. loam's job is
  a manifest good enough that any of them can.
- **run a server**, including a search server. `search` is a scan of files.
- **embed, vectorise, or keep a search index on disk.** The files are the index.
- **look across repositories.** A repository cannot see another one.
- **edit prose.** `$EDITOR` does that.
- **use the network.** Not even to check an external link.

## Start here

```sh
cairn list --view decide    # decisions and questions still open
cairn next                  # what can start
cairn roadmap
```

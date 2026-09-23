---
order: 5
summary: Give an agent the contract, and let loam keep it from writing the same page twice, in the wrong place, or as the truth.
covers:
  - src/cmd/agent.rs
  - src/cmd/context.rs
  - src/cmd/set.rs
  - src/agent.rs
---

# Working with agents

Asked to research, an agent writes `docs/research-notes.md`. Asked again next
week, it writes `docs/research/notes-2.md`, having never read the first. loam
gives it what it needs to do better: the rules, generated from your
configuration; the pages that matter for the file in front of it; and a refusal
when it is about to write a page that already exists under another name.

## 1. Give it the contract

```sh
loam agent --write AGENTS.md
```

puts a block between `<!-- loam:begin -->` and `<!-- loam:end -->` in
`AGENTS.md` (or `CLAUDE.md`), leaving the rest of the file alone — it uses
only marker lines outside code, and will not write into a file it cannot read
as text. Run it again
whenever `loam.toml` changes; it replaces the block in place, and changes
nothing when nothing changed. `loam agent` alone prints it.

The block is generated from `loam.toml`, so it cannot describe a kind or a
directory the project does not have. It tells an agent where each kind of page
lives and what question it answers; to search before writing; to use `loam new`
rather than a path of its own; to replace a page with `loam supersede`, never a
`-v2.md`; to say what a page covers, and for research, where it came from and
when it was read; and, after changing code, to reread the pages that cover it.

## 2. What an agent writes is a draft

With this in `loam.toml` — `loam init` writes it —

```toml
[agents]
status = "draft"
```

a page an agent creates with `loam new` starts as `status: draft`, and the
index marks it *(draft)*. A person makes it current:

```sh
loam set docs/research/markdown-crates.md status=current
```

loam knows an agent is writing when it is told — `--agent NAME`, or
`LOAM_AGENT` (or `CAIRN_AGENT`) in the environment — or when it recognises one:
`AI_AGENT`, which agents have begun to set, and Claude Code's `CLAUDECODE`.

**This is a guard rail and not a boundary: an agent with a shell can edit the
frontmatter.** It keeps an honest agent from passing off an hour's research as
a reviewed page. It does not stop one that means to.

## 3. A second page on the same thing is refused

```console
$ loam new research "Rust Markdown parsers"
research already has 1 page(s) like it:
  docs/research/markdown-crates.md  "Markdown crates"  (shares: markdown, pars, rust)
nothing written. Update the page above, or `loam new … --anyway` if this is different.
```

`new` compares the new title with the title and summary of every page of the
same kind, by the words they share — no embeddings, no index on disk. Words
common to a third or more of the kind's pages, like a project's own name, are
not counted. A person at a terminal is asked; an agent, or anything without a
terminal, is refused with exit status 1 unless it passes `--anyway`. Tried
against every title in the three folders loam was designed against, it raised
two pages that were not duplicates in 66
([0048](../../cairn/items/0048-warn-before-writing-a-page-that-already-exists-under-another-name.md)).

## 4. The pages for the file in front of it

```console
$ loam context src/fresh.rs
# loam context for src/fresh.rs (within 8192 bytes)

## docs/design/staleness.md — How loam decides a page is stale
(covers src/fresh.rs; fresh)
…
```

Pages whose `covers` match the path come first, the most specific first; then
pages that mention the path; then pages the covering pages link to. Each says
whether it is still true. A stale page is flagged, never left out: being told a
page is stale is itself context.

`--budget` caps the output in bytes — `8k` by default, about four bytes to a
token. Pages go in whole while they fit, then as their summaries, and the rest
are named as left out. [Claude Code hooks](claude-code-hooks.md) wires this in
so it arrives without being asked for.

## 5. Before it stops

```console
$ loam stale --working-tree
docs/design/staleness.md   covers src/fresh.rs, changed and not yet committed
  update it, or if it is still true: loam review docs/design/staleness.md
```

names the pages covering what the agent has changed and not committed;
`--since REV` does the same for the commits since `REV`. A page changed
alongside the code is left out. The exit status is 1 when there is something to
reread, which is what a stop hook needs.

## Frontmatter from the command line

```sh
loam set docs/research/markdown-crates.md covers+=src/render.rs summary="Which crates parse Markdown."
```

`key=value` sets, `key=` removes, `key+=` and `key-=` add to and take from a
list of strings — a list holding anything else is refused rather than
rewritten. The one key changes, and no other byte of the page. The format's own keys
are held to their types, so `loam set` cannot write what `loam check` would
then report.

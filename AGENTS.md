<!-- cairn:begin -->
## Roadmap and issues

This project tracks its roadmap and issues with `cairn`. Every item is a Markdown file under `cairn/items`, described by the schema in `cairn.toml`.

**Do not create ad-hoc TODO, PLAN or NOTES files.** Create a cairn item instead, so the work appears on the board and in the generated roadmap.

### The loop

1. `cairn next` — what is ready to start. It excludes anything blocked by unfinished dependencies and puts work already in progress first.
2. `cairn claim <ID>` — take it before you start, so no one duplicates the work. `cairn claim --next` picks and claims the top-ranked unclaimed item in one step, and prints its body so you can begin immediately.
3. Do the work. Record what you learn: `cairn set <ID> <field>=<value>` for fields, `cairn note <ID> "<TEXT>"` for anything that needs a sentence — why you chose something, what you tried, what to watch for.
4. `cairn tick <ID> <N>` as each acceptance criterion becomes true — `cairn show <ID> --criteria` lists them numbered. Tick what is true, not what would let you close.
5. `cairn close <ID>` when it is done, or `cairn release <ID>` to hand it back.
6. `cairn check` before you report finished. It must pass.

### Commands

```sh
cairn next --json                 # ready work, ranked
cairn claim --next                # take the next ready item
cairn search <TEXT> --json        # titles, bodies and labels
cairn list --json                 # all open items
cairn list --filter 'blocked=false,priority=p0'
cairn show <ID> --json            # one item, including its body
cairn new "<TITLE>" --type <TYPE> --milestone <MILESTONE>
cairn set <ID> status=<STATUS>    # also labels+=x, or any field below
cairn note <ID> "<TEXT>"          # append reasoning; never replaces
cairn show <ID> --criteria        # acceptance criteria, numbered
cairn tick <ID> <N>               # tick one; --all for every one
cairn close <ID>
cairn check                       # validate; run before finishing
cairn render                      # regenerate ROADMAP.md
```

### Schema

- **Types**: `decision`, `question`, `spec`, `feature`, `bug`, `chore`, `docs`, `milestone`, `pillar`
- **Statuses**: `backlog` (open), `planned` (open), `doing` (active), `blocked` (active), `done` (done), `dropped` (dropped)
- **`priority`**: one of p0, p1, p2, p3 — p0 blocks the milestone it is in — **you may propose this in a note, not set it**
- **`effort`**: one of s, m, l, xl — Rough size, not an estimate
- **`supersedes`**: names any items, by id, several allowed — an earlier decision this one replaces
- **Milestones**: `v0.0`, `v0.1`, `v0.2`, `v0.3`, `v0.4`, `later`
- **Saved views** (`cairn list --view NAME`): `decide`, `now`, `next`, `spec`, `triage`

### Rules

1. Before starting work, find or create the item and set it to an active status.
2. Use the fields above rather than inventing new ones; add new fields to `cairn.toml` first.
3. Never hand-edit the generated roadmap file — change items and run `cairn render`.
4. `cairn check` must pass before the work is considered done.

<!-- cairn:end -->

<!-- loam:begin -->
## Written context

This project keeps its written context — research, design, reference, guides — as Markdown pages under `docs/`, read and checked by `loam`. The index at `docs/README.md` is generated: never edit between its markers.

**Search before you write.** Another session has probably written about this already. Update that page rather than start another; a second page on the same thing is how a docs folder decays.

### Where pages go

- **`guide`** in `docs/guide/` — How do I do this? A task, start to finish, with real output.
- **`reference`** in `docs/reference/` — What does this mean? Facts, tables and settings, checked against the code where they can be.
- **`design`** in `docs/design/` — Why is it like this? The shape of the thing, and the argument for it.
- **`page`** in `docs/`

### The loop

1. `loam search <WORDS>` for what is already written, and `loam context <PATH>` for the pages about a file you are about to change. Read them.
2. To add a page: `loam new <KIND> "<Title>"` — never a path you made up. It refuses when a page like it exists: update that page instead, or pass `--anyway` if yours is really about something else.
3. A page you create starts as `status: draft`. A person makes it `current`; do not do that yourself. loam recognises Claude Code; any other agent identifies itself with `LOAM_AGENT=<name>` in the environment, or `--agent <name>`.
4. Write the page: its title as the `# heading`, then a first paragraph saying what the page is for — that paragraph is its summary in the index. Say which files it describes, so loam can tell when they change: `loam set <PAGE> covers+=<PATH>`. Research gives its `sources`, each with the date it was read.
5. A page that replaces another: `loam supersede <OLD> <NEW>` — never a `-v2.md` beside the old one. Renaming: `loam mv <OLD> <NEW>`, which rewrites every link.
6. After changing code, `loam stale --working-tree` names the pages covering what you changed. Update each, or if it is still true, `loam review <PAGE>`.
7. `loam check` must pass before you finish.

### Pages and backlog items

A cairn item records why something changed; a page says how it is now. A question is an item; its answer, if it outlives the question, is a page, and the item links to it. A plan with steps is a milestone and its items, not a page. A page cites the items behind it by a relative link to the item's file.

### Commands

```sh
loam search <WORDS> --json        # what is written, titles first
loam context <PATH> --budget 8k   # the pages about a file, within a budget
loam list --kind <KIND> --json    # every page of a kind
loam show <PAGE>                  # one page, and whether it is still true
loam new <KIND> "<TITLE>"         # a page where its kind lives
loam set <PAGE> <KEY>=<VALUE>     # frontmatter; covers+=<PATH> adds to a list
loam supersede <OLD> <NEW>        # one page replaces another
loam mv <OLD> <NEW>               # rename, rewriting every link
loam stale --working-tree         # pages your uncommitted changes affect
loam review <PAGE>                # record that a page was read against the code
loam check                        # validate; run before finishing
```
<!-- loam:end -->

# The loam page format

**Version 1.** A specification for keeping a repository's written context —
research, design, reference, guides — as a directory of Markdown pages in the
repository itself.

This document is normative and stands alone: a reader for the format can be
implemented from it without consulting any particular program. [loam][] is the
reference implementation, not the definition.

[loam]: https://github.com/oddurs/loam

---

## 1. Why a specification

A docs folder outlives the tool that indexes it. Somebody must be able to read
those pages in ten years, with whatever software exists then, and know what
they meant. That is only true if the format is written down.

There is a second reason, particular to pages. A page is read by GitHub, by
editors, by site generators and by agents long before any of them has heard of
loam. So the format asks almost nothing of a page: a plain Markdown file with a
heading is already a conforming page, and everything loam wants to know about
it has a defined answer when nothing has been written down. Frontmatter is how
a page overrides an answer, not the price of being read.

The format is a sibling of the [cairn item format][cairn-spec], which keeps a
backlog the same way. Where the two describe the same thing — how frontmatter
is delimited, how YAML scalars resolve — this document uses cairn's words
exactly, and a test in this repository holds the two to each other (§10).
Where they differ, it is because a page is not an item: a page is addressed by
its path rather than a number, lives in a tree rather than a flat directory,
and is never finished.

[cairn-spec]: https://github.com/oddurs/cairn/blob/main/spec/README.md

## 2. Terminology

The key words **must**, **must not**, **should**, and **may** are to be
interpreted as in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

A **repository** is the directory containing a configuration file, `loam.toml`
(§7). Every path in this document is relative to it, uses `/` as its separator,
and is compared exactly: two paths are the same only if they are the same
sequence of Unicode characters. A reader **must not** rely on a filesystem that
ignores case, because the places a repository is read — a Linux checkout,
GitHub, a site generator — do not.

The **docs root** is the directory under the repository that holds the pages.

A **page** is a Markdown file under the docs root (§7.2).

A **kind** is a category of page the project declares — `guide`, `reference`,
`design`, or whatever the project calls its own. Kinds belong to the project,
not to this format.

**Whitespace** means spaces (U+0020) and tabs (U+0009), and nothing else:
removing whitespace does not remove a no-break space or any other character
Unicode calls a space.

A **reader** is a program that reads pages. A **writer** is one that creates or
changes them. A **finding** is something a reader reports about a page without
refusing to read it (§6.5).

## 3. Structure of a page

A page is a file whose name ends in `.md`, encoded in UTF-8. It may begin with
frontmatter, and need not.

A file that is not valid UTF-8 is a finding (`invalid-encoding`), and a reader
**must not** read anything from its content.

A page a reader may read nothing from — this one, and the malformed page of
§3.1 — is still a page. It has the kind and the status §5.4 and §5.5 give a
page with no frontmatter, since neither depends on content. It has no title,
no summary, no anchors and no links, and `untitled` is not reported for it: the
finding that says why has been reported already.

### 3.1 Frontmatter

A page with frontmatter contains, in order:

<!-- shared with the cairn item format, §3: begin -->
1. A line consisting of exactly `---`, optionally preceded by a UTF-8 byte order
   mark.
2. Zero or more lines of YAML, the **frontmatter**.
3. A line which, after trailing whitespace is removed, consists of exactly `---`
   or `...`: the **closing delimiter**. The *first* such line ends the
   frontmatter; later ones are body text.
<!-- shared: end -->
4. The **body**: everything after the closing delimiter, which is Markdown.

A file whose first line (after an optional byte order mark) is not exactly
`---`, once trailing whitespace is removed, has no frontmatter, and the whole of
it is the body. That is the one place this format departs from cairn's, where
such a file is not an item at all: most pages that exist have no frontmatter,
and requiring it would put a rewrite of every page in front of adopting the
format.

A file that begins with an opening delimiter and has **no** closing delimiter
is **malformed**. A reader **must** report it (§6.5) and **must not** read
anything from its content — not frontmatter, and not a title or a summary from
what might be its body. Its first line says frontmatter was intended, and
reading YAML comments as headings would be a guess.

The frontmatter **must** be a YAML mapping. Frontmatter that is not valid YAML,
or is valid YAML but not a mapping, is malformed: a reader **must** report it,
and **must** then read the page as though it had no frontmatter, taking the
body from after the closing delimiter as usual. Empty frontmatter, or
frontmatter that is only comments, is an empty mapping.

<!-- shared with the cairn item format, §3: begin -->
Lines are separated by either LF or CRLF. A reader **must** accept both. A
writer **must** reproduce whichever the file used, and **should** use LF for a
file it creates.
<!-- shared: end -->

A byte order mark is not part of the content, and a reader **must** discard it.
Line numbers, where this document uses them, count from 1 at the first line of
the file, frontmatter included.

A page with no frontmatter whose first line is a thematic break written `---`
will be read as opening frontmatter. Write that break as `***`, or do not begin
a page with one.

### 3.2 YAML scalars

<!-- shared with the cairn item format, §6: begin -->
The frontmatter is YAML, and YAML resolves unquoted scalars before any of this
applies. A reader **must** resolve them according to the **YAML 1.2 core
schema**.

Naming the version is not pedantry. YAML 1.1 and 1.2 disagree about exactly the
values people write by hand, and a reader using the wrong one will report
different content for the same file:

| Written | YAML 1.2 core (required) | YAML 1.1 (wrong here) |
| --- | --- | --- |
| `no` | the string `no` | the boolean false |
| `12:30` | the string `12:30` | the integer 750, read as sexagesimal |
| `0x1F` | the integer 31 | the integer 31 |
| `1.20` | the float 1.2 | the float 1.2 |

Under 1.2 core, only `true` and `false` (and their capitalised and upper-case
spellings) are booleans; `yes`, `no`, `on` and `off` are strings.

This is worth checking rather than assuming: several widely used YAML libraries
still implement 1.1 by default, including PyYAML. The reference reader in
`reader.py` adjusts for it, and says where.

A writer **must** quote any value it emits that would otherwise change meaning
when read back. This confines the problem to files written by hand or by other
tools, where the remedy is to quote such values.
<!-- shared: end -->

The 1.2 core schema has no dates, so `2026-09-22` unquoted is the string
`2026-09-22`. That is what a page means by it.

Nor has it a merge key: `<<` is an ordinary key, and nothing is merged. An
explicit tag is honoured if it names a type of the core schema — `!!str`,
`!!int`, `!!float`, `!!bool`, `!!null`, `!!seq`, `!!map` — so `title: !!str 2026`
is the string `2026`. Any other tag, `!!timestamp` or `!!set` or a local
`!custom`, makes the frontmatter malformed, as YAML that does not parse does.

### Example

```markdown
---
kind: research
status: draft
summary: What three other tools do about stale documentation, and what they miss.
supersedes: ../research/staleness-2025.md
---

# How other tools notice stale docs

Most do not notice at all. The ones that do compare modification times, which
a formatting commit defeats.

## Sphinx
```

The same page with no frontmatter at all is also a page. Its title and summary
are read from the heading and the first paragraph, its kind from the directory
it is in, and its status is `current` (§5).

## 4. Keys

The frontmatter is a mapping. Every key is optional.

| Key | Type | Notes |
| --- | --- | --- |
| `title` | string | What the page is called in an index. When absent, see §5.2. |
| `kind` | string | Names a kind the project declares (§7.1). When absent, see §5.4. |
| `status` | string | One of `draft`, `current`, `superseded` (§4.1). When absent, see §5.5. |
| `summary` | string | A sentence or two saying what the page is for. When absent, see §5.3. |
| `supersedes` | sequence of strings | Pages this one replaces, each written as a link destination (§4.2). A reader **must** also accept a single string, meaning a sequence of one. |
| `superseded_by` | sequence of strings | Pages that replace this one, written the same way. A reader **must** also accept a single string. |
| `order` | integer | The page's place among the pages of its kind, lowest first, for anything that lists them. Pages with no `order` follow those with one. |
| `covers` | sequence of strings | The files the page describes, as patterns (§4.3). A reader **must** also accept a single string. |
| `reviewed` | mapping | When the page was last read against the code it covers (§4.3). |
| `generated` | string | What generates the page, when a program does (§4.3). |

A value of the wrong type — a sequence where a string is expected, a number, a
boolean, a mapping — is a finding (`malformed-key`), and a reader **must** then
treat the key as absent, so that §5 supplies its value. `title: 2026`, unquoted,
is the integer 2026 by §3.2 and therefore such a finding; write `title: "2026"`.
A key whose value is null is absent.

### 4.1 Status

A page is in one of three states, and never done.

- **`draft`** — written, not yet trusted. What an agent's first pass at research
  is, and what a half-finished design is.
- **`current`** — what the page says is meant to be true. The default.
- **`superseded`** — kept, but replaced. Superseded research is still evidence,
  and a link to it should land somewhere that says what replaced it, so a
  superseded page is not deleted.

The set is fixed by this format, not declared by a project. That is what lets
anything built from a docs tree — an index, a site, an agent's context — treat a
draft as a draft without reading anybody's configuration.

Any other value is a finding (`unknown-status`). A reader **must** report the
value as written rather than substitute one.

Whether a page is *stale* — whether what it describes has changed since it was
last read — is not a status. It is a fact derived about a page, not written
into it, and a current page can be stale.

### 4.2 Supersession

A value of `supersedes` or `superseded_by` is a path written as a link
destination is (§6.2): relative to the page it is in, or beginning with `/` for
the repository root. It **must** resolve to a page. One that does not is a
finding (`broken-supersession`).

Supersession is stated on both pages. When page A lists page B under
`supersedes`, B **should** list A under `superseded_by`, and the reverse. A
reader **must** report each side that lacks its partner as a finding
(`one-sided-supersession`) on the page that states it. A writer that records
one side **should** record the other in the same change.

### 4.3 Covers, reviewed and generated

These three say what a page describes and when it was last known to be true.
A reader that computes nothing from them still reads them, reports a value of
the wrong type, and preserves them.

**`covers`** is a list of patterns naming files in the repository. A page covers
a file when the file's path matches at least one pattern that does not begin
with `!`, and none that does. A pattern is a path relative to the repository —
a leading `/` is ignored — whose segments, separated by `/`, are matched against
the file path's segments:

- `**`, as a whole segment, matches any number of segments, including none;
- any other segment matches one segment, in which `*` matches any run of
  characters and `?` matches exactly one, and every other character matches
  itself;
- a pattern also matches every file below a directory it matches, so `src/engine`
  covers `src/engine/step.rs`.

`!src/engine/tests` therefore uncovers what `src/engine` would cover. An `!` must
be quoted in YAML, where it otherwise begins a tag: `- "!src/engine/tests"`.

**`reviewed`** is a mapping of two strings: `commit`, the hexadecimal name of the
commit the page was read against, at least seven characters; and `date`, the
day it was, as `YYYY-MM-DD`. Both, because a commit can stop existing — a branch
squashed or rebased away — and a date cannot. A `reviewed` without both, or with
either not a string of that form, is `malformed-key`. Other keys inside it are
preserved and mean nothing here.

**`generated`** names what writes the page, as its author would say it: `make
status`. Such a page is kept true by its generator, not by being read, and
anything that reports whether a page is still true **should** leave it out.

Whether a page is stale — whether what it covers has changed since it was
reviewed — is a fact a program derives from these and from the repository's
history. It is not written into a page, and this format does not define it.

### 4.4 Reserved keys

`aliases` is reserved for a later revision of this format. A reader of this
version treats it as it treats any key it does not recognise (§4.5). A project
**must not** use it to mean anything else.

### 4.5 Unknown keys

> **A reader must preserve keys it does not recognise.**

A writer rewriting a page's frontmatter **must** keep every key it did not
change, with its value, and **should** keep them in the order they were read.
This is the rule that makes version skew survivable: without it, an older
reader opening a newer project silently deletes whatever the newer one wrote.

An unknown key is never an error and never a finding. Pages carry frontmatter
for other programs — site generators, editors — and those keys are none of this
format's business.

## 5. Inference

Every key in §4 has an answer when the frontmatter does not give one. This
section defines it exactly, so that two readers report the same title and the
same summary for a page nobody wrote frontmatter for.

Frontmatter always wins. A key present with a value of the right type is the
answer, and nothing in the body is consulted for it.

### 5.1 The block scan

Inference, anchors (§6.3) and links (§6.1) all depend on telling a heading from
a paragraph from code. Markdown's own grammar is large, and two readers that
each implement "Markdown" will disagree at its edges. So this section defines a
small line-based scan that every reader **must** perform exactly. It agrees
with [CommonMark][] and GitHub on everything the real pages this format was
drawn from contain, and where it differs from them, this section decides what
a reader reports.

[CommonMark]: https://spec.commonmark.org/

The scan reads the body line by line, from the top, and sorts every line into
one of four kinds of **block**: *code*, *heading*, *paragraph* and *break*. A
line is **indented** if it begins with a tab or with four spaces. A line is
**blank** if it is empty or holds only spaces and tabs.

1. **Fenced code.** A line of up to three spaces, then three or more backticks
   or three or more tildes, opens a fence — unless the fence is of backticks
   and the rest of the line contains a backtick. Every following line is code
   until a line of up to three spaces, then a run of the same character at
   least as long as the opening run, then only spaces or tabs. That line is
   code too, and closes the fence. A fence never closed runs to the end of the
   body.
2. **Blank lines** end the paragraph being gathered, and belong to no block.
3. **Indented code.** An indented line is code if the line before it was blank,
   or was the start of the body, and no paragraph is being gathered; or if the
   line before it was itself indented code. So indented code begins only after
   a blank line, and continues through indented lines and the blank lines
   between them until a line that is not indented.
4. **Setext headings.** A line of up to three spaces, then one or more `=` or
   one or more `-`, then only spaces or tabs, when a paragraph is being
   gathered and that paragraph's first line does not begin a list item or a
   quote (below): the gathered lines become a heading instead of a paragraph —
   level 1 for `=`, level 2 for `-` — whose text is those lines, each with
   surrounding whitespace removed, joined with single spaces. Its line is the
   first of them.
5. **ATX headings.** A line of up to three spaces, then one to six `#`, then
   either the end of the line or a space or tab. The number of `#` is the
   level. The text is the rest of the line with surrounding whitespace
   removed, and then with any trailing run of `#` removed if it is the whole
   text or is preceded by a space or tab, and whitespace removed again.
6. **Breaks.** A line of up to three spaces, then three or more of the same
   character, `-`, `*` or `_`, with only spaces or tabs between and after them.
   (A line of `-` directly under a paragraph is a setext heading by rule 4
   first.)
7. **Paragraphs.** Any other line is gathered into the current paragraph, or
   begins one. A line that begins a **list item** — up to three spaces, then
   `-`, `*` or `+`, or one to nine digits followed by `.` or `)`, then a space,
   a tab or the end of the line — or a **quote** — a first non-space character
   of `>` — ends the paragraph being gathered and begins a new one, unless that
   paragraph itself began with a list item or a quote.

A heading, a code line and a break each end the paragraph being gathered.

The rules are applied in the order written. The scan does not recognise HTML
blocks, tables, or the structure inside lists and quotes; their lines are
paragraph lines, and a reader **must not** treat them otherwise. Three
consequences are worth knowing, all divergences from GitHub a page is unlikely
to meet, and the price of two readers agreeing:

- A heading inside a quote (`> # Title`) is not a heading. GitHub gives it an
  anchor; a reader of this format does not, and reports a link to it as broken.
- A paragraph nested four spaces deep inside a list item, after a blank line,
  is indented code, and its links are not found.
- A paragraph that begins with an HTML tag is, on GitHub, raw HTML, in which a
  backtick is an ordinary character. Here it is a paragraph like any other, and
  its code spans are code spans.

### 5.2 Title

When `title` is absent, the title is the text of the first level-1 heading in
the body, as the scan (§5.1) gives it — inline Markdown kept as written. A
heading is the page's **title heading** when it supplies the title this way, or
would have if `title` were absent.

A page with neither has no title. That is a finding (`untitled`), and a program
that has to show something **may** show the file name.

### 5.3 Summary

When `summary` is absent, the summary is read from the first block after the
title heading, or from the first block of the body if the page has no title
heading:

1. A paragraph whose lines, once every image is removed from them, are blank —
   a banner, a screenshot, a row of badges — is passed over, and the block
   after it is considered instead. An image here is `![text](destination)` or
   `![text][label]`, alone or as the text of a link.
2. If the block is a paragraph, and its first line does not begin a list item,
   a quote, a table row (`|`) or HTML (`<`), and none of its lines is a link
   reference definition (§6.1), the summary is its lines, each with surrounding
   whitespace removed, joined with single spaces. Inline Markdown is kept as
   written.
3. Otherwise — a heading, code, a break, a list — the page has no summary.

Only the first block is considered. A page that goes straight from its title to
`## Background` has no summary; it does not borrow a sentence from further
down, which is almost never the sentence that says what the page is.

A page with no summary is not a finding. Plenty of good pages have none.

### 5.4 Kind

When `kind` is absent, the kind is the one that claims the directory the page
is in (§7.1): of the kinds whose directory is that directory or one of its
ancestors, the one whose directory is longest. A kind whose directory is the
docs root claims every page no other kind does.

A page no kind claims has no kind, and that is a finding (`unclaimed`). A
`kind` in frontmatter that names no kind the project declares is a finding
(`unknown-kind`); a reader **must** report the value as written.

### 5.5 Status

When `status` is absent, it is `superseded` if `superseded_by` is present and
not empty, whether or not what it names exists — a name that resolves to no
page is a finding of its own (§4.2) — and `current` otherwise.

## 6. Links

A page's links are part of the format. What a reader finds, how it resolves
it, and what counts as broken are defined here so that two readers report the
same broken links for the same tree.

No part of this section uses the network. A reader **must not** fetch anything
to decide whether a link is broken.

### 6.1 Finding links

A reader **must** find, in every block of the body that is not code (§5.1),
headings included:

- **Inline links and images**: `[text](destination)` and `![text](destination)`,
  optionally with a title after the destination — `"title"`, `'title'` or
  `(title)` — separated from it by whitespace. Whitespace is also allowed after
  the `(` and before the `)`. The text may contain
  brackets, balanced; a bracket preceded by a backslash does not count. The
  text may run over several lines of one paragraph, as wrapped prose makes it
  do; the destination, and the title and `)` after it, are on one line.
- **Link reference definitions**: a line, not inside a heading, of up to three
  spaces, then `[label]:`, then optional spaces or tabs, then the destination.
  The definition is a link whether or not anything uses the label.

A destination is either everything between `<` and the next `>`, or a run of
characters containing no space or tab in which parentheses are balanced and a
character preceded by a backslash is taken literally. A backslash before an
ASCII punctuation character is removed from the destination before it is
resolved. An empty destination is not a link.

Code spans are not searched. A code span is a run of one or more backticks, the
text after it, and the next run of exactly the same number of backticks on the
same line; a backtick run with no partner is literal text. Autolinks
(`<https://…>`), bare URLs, and links in raw HTML are not links for this
format: the first two are absolute (§6.2) and would never be checked anyway.

A link's **line** is the line of the file its destination is on. Within a
paragraph, links are listed in the order of their lines, and then of where
their destinations begin.

Code spans are found in a paragraph as a whole, so one may run over a line
break, as a link's text may.

### 6.2 Resolving a destination

A destination is resolved as a browser would resolve it from the page's own
location, with the repository standing in for a web server's root:

1. A destination beginning with a scheme — a letter, then letters, digits, `+`,
   `.` or `-`, then `:` — or with `//`, is **external**: `https:`, `mailto:`,
   and so on. It is never resolved and never checked.
2. Otherwise, everything from the first `#` is the **fragment**, without the
   `#`; everything from the first `?` in what remains is discarded. Both the
   path and the fragment are percent-decoded as UTF-8. A fragment that is
   empty is no fragment.
3. An empty path is **this page**.
4. A path beginning with `/` is relative to the repository. Any other path is
   relative to the directory the page is in.
5. Empty segments and `.` are removed, and each `..` removes the segment before
   it. A `..` with no segment before it — a path that leaves the repository —
   is a finding (`escapes-repository`).

The result is the link's **target**. A target resolves when a file or a
directory at exactly that path exists in the repository. A link to a directory
is a link to the directory; GitHub shows it, and a reader **must** accept it.

### 6.3 Anchors

When a link has a fragment and its target is a file whose name ends in `.md` —
a page, or any other Markdown file in the repository — the fragment **must**
name an anchor in that file. The fragment is compared with the file's anchors
exactly.

The **anchors** of a Markdown file are:

1. The **slug** of every heading in its body, in order, as the scan of §5.1
   finds them — frontmatter excluded, and a file that is malformed by §3.1 has
   none.
2. The value of every `id` or `name` attribute of an HTML tag in a heading or
   a paragraph of its body (§5.1), code spans excepted (§6.1) — so
   `## <a name="install"></a>Installing` gives the anchor `install` as well as
   its slug. A tag is `<`, a letter, and everything to the next `>`, and may
   run over several lines of one paragraph. An attribute is the name `id` — or `name`, on an `a` tag only,
   since that is the one element a browser follows a `name` to — preceded by
   whitespace or a line break, so `data-id` is not one; then optional
   whitespace, `=`, optional whitespace, and a value in double or single
   quotes. Tag and attribute names are compared without regard to ASCII
   case. These are listed after the slugs, once each, and a value already
   among the slugs is not listed again.

A heading's slug is computed the way GitHub computes it, because that is the
rule everybody already sees working. First the heading's **text**, meaning
what it reads as once rendered, is taken from its source. A code span (§6.1)
reads as its content, exactly — the backticks gone, and nothing inside it
touched by the steps below: `` `--format <type>` `` reads as `--format <type>`.
Outside code spans:

1. Every image is removed.
2. Every inline link and every full reference link, `[text][label]`, is
   replaced by its text.
3. Every HTML tag, `<` through the next `>`, is removed.
4. A backslash before an ASCII punctuation character is removed.
5. A run of `_` is removed when it has a letter or digit (general category L or
   N) on one side and not on the other: it marks emphasis, not a name. `_under_` becomes `under`;
   `snake_case` is unchanged.
6. HTML character references — `&amp;`, `&#233;`, `&eacute;` — are decoded.

`*` needs no rule of its own: step 7 removes it.

Then the text is turned into a slug:

7. It is lowercased, by Unicode's default case mapping.
8. Every character is removed that is not a hyphen-minus, a space (U+0020), or
   a **word character**: one with the Unicode property Alphabetic, or in general
   category Mark, Decimal_Number or Connector_Punctuation, or one of U+200C and
   U+200D. This is the definition of a word character in
   [Unicode Technical Standard #18, Annex C][uts18].
9. Every space is replaced with a hyphen-minus.

[uts18]: https://www.unicode.org/reports/tr18/#Compatibility_Properties

Slugs within one file are made unique as GitHub makes them. A reader keeps a
count for each slug it has produced. For each heading, if its slug has not
been produced before, it is used, with a count of 0. If it has, the count for
that slug is increased by one and the slug becomes the original followed by `-`
and the count, and this is repeated, from the same original, until the result
has not been produced before; the result is then recorded with a count of 0.
So three headings `Hello World` give `hello-world`, `hello-world-1` and
`hello-world-2`, and a heading `Hello World-1` after the second of them gives
`hello-world-1-1`.

Examples: `Café résumé` → `café-résumé`; `Roadmaps — design rationale` →
`roadmaps--design-rationale`; `100% ✓ done` → `100--done`; `C++ / C#` →
`c--c`.

A heading with a GitHub emoji shortcode, `:tada:`, renders as the emoji and
loses the word, and its slug here keeps it. A reader is not required to know
GitHub's list of shortcodes; write the emoji itself in a heading that is linked
to. Characters assigned by a version of Unicode newer than a reader knows may
also slug differently from one reader to another.

A fragment on a link to anything but a Markdown file — `main.rs#L10` — is not
checked.

### 6.4 References to cairn items

A page cites the backlog items that made it the way it is, and a cairn item's
file name changes whenever its title does: `0072-old-title.md` becomes
`0072-better-title.md`, and a plain relative link to the old name breaks.

A reference to an item is therefore an ordinary relative link to its file —
which renders, and works, on GitHub and in every site generator — whose
identity is the number the file name begins with, not the rest of the name:

```markdown
The format is versioned from the first release ([0025](../cairn/items/0025-version-the-format-from-the-first-release.md)).
```

A project names the directory its cairn items are kept in (§7.1). A link whose
target is a file directly in that directory, whose name begins with one or
more digits and ends in `.md`, is a **cairn reference** to the item with that number; leading
zeros are not significant. It is resolved against the Markdown files directly
in that directory whose names begin with the same number:

- If the target is one of them, the reference resolves.
- If it is not, but one exists, the reference is **stale**: it names the right
  item by an old name. That is a finding (`stale-cairn-link`), and it is the one
  finding a writer can repair without asking anybody, by rewriting the
  destination's file name to the current one.
- If none exists, the reference is broken (`broken-cairn-link`).

A project that names no cairn directory has no cairn references; links into
its backlog are ordinary links.

This reads an item's number from its file name, which the cairn format names
`<id>-<slug>.md` by default and does not require. A backlog whose file names do
not begin with the number — one whose identifiers render with a prefix, such as
`MP-1002` — has no cairn references a reader of this version can recognise;
links into it are ordinary links. A reader **must not** open an item file to
read its `id` for this purpose: doing so would make the reading depend on
another format's contents, and two readers of different vintages of it could
disagree.

A scheme such as `cairn:72` would survive retitling, and is not used: GitHub
removes a link with a scheme it does not know, leaving its text and no link at
all.

### 6.5 Findings

A finding is reported with the page, a line, a code, and where there is one,
the value it concerns. A finding about a link is reported on the link's line.
Every other finding — about the frontmatter, a key or value in it, or the page
as a whole — is reported on line 1. A YAML parser need not say where in the
frontmatter a key was, so a reader is not asked to. A finding does
not stop a reader reading the rest of the page or the rest of the tree.

| Code | Meaning |
| --- | --- |
| `invalid-encoding` | §3: the file is not UTF-8. |
| `malformed-frontmatter` | §3.1: no closing delimiter, not YAML, or not a mapping. |
| `malformed-key` | §4: a key of this format with a value of the wrong type. |
| `unknown-status` | §4.1: a status that is not one of the three. |
| `broken-supersession` | §4.2: `supersedes` or `superseded_by` names no page. |
| `one-sided-supersession` | §4.2: the other page does not say so too. |
| `untitled` | §5.2: no `title` and no level-1 heading. |
| `unclaimed` | §5.4: no `kind`, and no kind claims the directory. |
| `unknown-kind` | §5.4: a `kind` the project does not declare. |
| `broken-link` | §6.2: the target does not exist. |
| `escapes-repository` | §6.2: the target is outside the repository. |
| `broken-anchor` | §6.3: the target exists and has no such anchor. |
| `stale-cairn-link` | §6.4: the item exists under another file name. |
| `broken-cairn-link` | §6.4: no item has that number. |

A link that is found to be broken is reported once, with the most specific
code that applies.

A program **may** report more than this — a page nothing links to, a page no
index lists — but those are its own judgements, not the format's.

## 7. The project

### 7.1 Configuration

A repository contains a configuration file named `loam.toml` at its root. It is
[TOML 1.0](https://toml.io/en/v1.0.0). The keys that bear on reading pages are
these, and a reader **must** honour them:

| Key | Type | Notes |
| --- | --- | --- |
| `format` | integer | The format version (§9). Required. |
| `docs.root` | string | The docs root, relative to the repository. Default `docs`. |
| `links.cairn` | string | The directory holding the project's cairn items, relative to the repository (§6.4). No default. |
| `kind` | array of tables | The project's kinds, in the order it wants them presented. |
| `kind.name` | string | Required, and unique among the kinds. |
| `kind.dir` | string | The directory the kind claims, relative to the docs root. Default `.`, the docs root itself. At most one kind may claim a directory. |

```toml
format = 1

[docs]
root = "docs"

[links]
cairn = "cairn/items"

[[kind]]
name = "guide"
dir = "guide"
description = "How do I do this? A task, start to finish, with real output."

[[kind]]
name = "design"
dir = "design"
description = "Why is it like this? The shape of the thing and the argument for it."

[[kind]]
name = "page"
description = "Anything that has not found its kind yet."
```

A configuration will say much else — a kind's template, its description, how
the index is laid out. None of it changes what a page *is*, and a reader that
ignores all of it conforms.

A reader **must** ignore keys in the configuration that it does not recognise,
and **should** warn about them. Stating this is what allows the configuration
to gain an optional key without a new format version — but only a key a reader
can safely ignore (§9).

Where the configuration expresses something as an ordered sequence — the kinds
— the order is significant. A reader that presents kinds **should** present
them in it, and a tool rewriting the configuration **must not** reorder them.

### 7.2 Which files are pages

A page is a file under the docs root whose name ends in `.md`, at any depth,
unless a directory or file name on its path below the docs root begins with `.`
or `_`. Those are left for other purposes — a site generator's partials, a
template directory — and are not pages.

`README.md` is a page. In a docs tree, a directory's README is the page GitHub
shows for it, and it is linked to like any other; this differs from cairn,
where a README among items is not an item.

Every other file — an image, an HTML file, a Markdown file outside the docs
root — is not a page, and is still a target a link may name (§6.2).

## 8. What this format does not do

It does not describe how a page is rendered, beyond what §5 and §6 need to know
about headings, paragraphs and links. It does not describe an index, a
manifest, or a site. It does not describe anything outside one repository.

## 9. Versioning and compatibility

The configuration records the format version as `format`. This document
describes version 1.

1. A reader encountering a version it does not understand, or a configuration
   with no `format`, **must** refuse the project and say so, naming the
   version it found and the version it understands. It **must not** read it on
   a best-effort basis: misreading pages is worse than declining to read them.
2. A reader of pages **must** preserve keys it does not recognise (§4.5), and a
   reader of the configuration **must** ignore them (§7.1).

What costs a version number, and a migration path for existing projects:

- a key changing what it means, or its type;
- a default changing — what an absent key is inferred to be (§5), or where the
  docs root is;
- anything in §5 or §6 changing what a reader reports for a page that already
  exists: a title, a summary, a slug, whether a link is broken;
- an optional key becoming required;
- a key being removed;
- the set of statuses changing (§4.1).

What does not:

- a new optional key in the frontmatter, including the one §4.4 reserves;
- a new optional key in the configuration, **provided** a reader that ignores
  it still reports the same reading. A key that changes which files are pages,
  or what a reader reports for one, is not optional in this sense: an older
  reader would ignore it, as §7.1 tells it to, and misread the project without
  knowing. Such a key costs a version number;
- a new finding code, for a condition that was not reported before;
- anything a program does with pages beyond reading them.

The list of what costs a number is the useful part of this section. cairn
added versioning at its first release and found that deciding, case by case,
whether a change needed a new version was where the mistakes were made.

## 10. Conformance

An implementation conforms if it satisfies §3 through §7 and §9.

The **reading** of a repository is what a conforming reader reports for it:
for every page, by path, its frontmatter as parsed, its title, kind, status and
summary with where each came from, its anchors, its links, and its findings.
`spec/reader.py` prints it as JSON; the shape of that JSON is a convenience of
the corpus, not part of the format.

The corpus in [`spec/corpus`](corpus/) holds small repositories and the reading
each must produce. It holds real docs folders, copied as they were, because
those are what a reader actually meets, and hostile cases, because those are
what a reader gets wrong. `spec/conformance.py` runs `spec/reader.py`, or any
other reader that prints the same JSON, against it:

```sh
python3 spec/conformance.py                       # the reference reader
python3 spec/conformance.py -- ./my-reader        # yours: run with a repository path
```

A case that has ever been in the corpus stays in it. Changing what one expects
means the format changed, which costs a version number.

The shared sections of §3 are marked in this document's source, and
`spec/shared.py` checks that each is word for word in the cairn item format's
specification. Two specifications that described the same frontmatter
differently would be a bug in one of them.

## 11. Conventions (non-normative)

Nothing here is required of a reader. It is what makes pages read well to
people, to GitHub, and to §5.

- **Put the title in a level-1 heading on the first line**, and leave `title`
  out of the frontmatter. A title in both places will drift.
- **Say what the page is for in its first paragraph**, directly under the
  title, in a sentence or two. That paragraph is the summary an index shows.
  Put a status line, a banner or a generated-by notice below it, not above.
- **Use frontmatter for what the body cannot say**: `status: draft`,
  `supersedes`, and a `summary` when the first paragraph is not one.
- **Link to pages by relative path**, and to cairn items by relative path to
  their file. Both render on GitHub, and a reader can check both.

---

Copyright © 2026 Oddur Sigurdsson. Copying and distribution of this
specification, with or without modification, are permitted in any medium without
royalty provided this notice is preserved.

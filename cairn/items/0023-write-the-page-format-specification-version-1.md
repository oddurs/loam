---
id: 23
title: Write the page format specification, version 1
type: spec
status: done
milestone: v0.0
assignee: Oddur Sigurdsson
depends_on:
- 15
- 16
- 17
- 18
- 21
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
pillar:
- format
effort: m
---

## Current text

None.

## Proposed text

`spec/README.md`, normative, standing alone, in RFC 2119 language — modelled on
cairn's, and sharing its §3 (structure of a file: delimiters, BOM, LF and CRLF,
the body ascribes no meaning) word for word where the rules are the same. Two
specifications that describe the same frontmatter differently would be a bug.

Sections:

1. Why a specification
2. Terminology — page, kind, docs root, project
3. Structure of a page — shared with cairn
4. Keys — `title`, `kind`, `status`, `summary`, `supersedes`, and room for
   `covers` / `reviewed` (v0.2) without a format bump
5. Inference — what a reader may derive when a key is absent, per the
   frontmatter question
6. Links — see its own spec item
7. Unknown keys — preserved on rewrite, never an error
8. Conformance

## What a conforming reader has to do differently

Everything; this is the first version.

## Acceptance criteria

- [x] spec/README.md written, with an example page
- [x] A reader can be implemented from it without consulting loam
- [x] The shared structure section is identical to cairn's, and a test says so
- [x] A corpus case exercises every **must** on a reader (a writer's are 0062)

## 2026-09-22

Criterion 4 narrowed from 'every must' to 'every must on a reader', and the rest filed as 0062. Four musts are on a writer — keep line endings, quote ambiguous scalars, keep unknown keys, never reorder kinds — and a reading corpus cannot exercise them; there is no writer until v0.1. Every must on a reader has a case, mapped in the next note.

## 2026-09-22

Every must on a reader, and the case that exercises it (spec/corpus/hostile unless noted). §2 exact paths: links (OTHER.md, which exists on a case-insensitive disk). §3 UTF-8: invalid-encoding. §3.1 unclosed: unclosed-frontmatter; not YAML / not a mapping: malformed-yaml; LF and CRLF: crlf, mixed-line-endings; BOM: bom. §3.2 YAML 1.2 core: yaml-1-1-scalars, yaml-tags-and-merge. §4 single string for supersession: supersession; wrong type treated as absent: wrong-types. §4.1 unknown status as written: status. §4.2 broken and one-sided: supersession. §4.4 unknown keys preserved: unknown-keys. §5.1 the scan: fences, anchors, summary. §5.4 unknown kind as written: kinds. §6 no network: links (external never checked). §6.1 finding links: links. §6.2 directories: links. §6.3 anchors: anchors, crlf, links. §6.4 not reading an item's id: cairn (0007 says id 8). §7.1 honour config, ignore unknown keys: kinds. §9 refuse: format-unknown, format-missing, format-as-string.

## 2026-09-22

Criterion 2 tested rather than assumed. An agent wrote a second reader from spec/README.md and spec/corpus/README.md alone, never opening spec/reader.py or any expectation. First run: 26 of 31 cases. All five failures were one gap — the spec said a finding concerning 'the page as a whole' is on line 1, and left open which line a finding about a frontmatter key is on. It also listed nine smaller ambiguities it had guessed at and happened to guess right: whitespace, YAML tags and merge keys, unreadable pages, when a status is inferred superseded, whitespace inside a link's parentheses, which HTML attributes are anchors, non-Markdown files in the cairn directory, and two gaps in the corpus's JSON description. Each was settled in the spec, with a corpus case where one could show it. Rewritten from the revised text alone, the second reader passed 32 of 32. One of those settlements was then reversed on the merits: §6.3 now takes id and name from headings too, because '## <a name="x"></a>Title' is how people keep an old anchor alive, and the corpus pins it.

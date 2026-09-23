---
id: 17
title: Page kinds belong to the project, and presets come from real folders
type: decision
status: done
milestone: v0.0
depends_on:
- 13
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
pillar:
- format
- cli
effort: s
---

## Context

The three docs folders already in `~/Code` show three different taxonomies:

- **poptop**: guide / reference / design — Diátaxis without tutorials, and its
  README says what question each answers.
- **code-as-color**: concept, outline, voice, design, architecture, authoring,
  research, decisions — named by subject, one page each.
- **measure-of-the-world**: build, releases, styleguide, and a `plan/` folder.

No fixed set of kinds fits all three, and cairn's position — *the schema is
yours* — is the answer that already works.

## Recommendation

**Kinds are declared in the config, like cairn's types.** A kind has a name, a
directory, a template, a one-line description of the question it answers, and
(from v0.2) a freshness policy. Nothing is looked up by a kind's name.

`init` offers presets, derived from those folders rather than invented:

- `diataxis` — guide, reference, design (poptop)
- `standard` — guide, reference, design, research
- `minimal` — one kind, `page`, for a folder that has not decided

## Consequences

- A page in a directory no kind claims is either inferred as the default kind
  or reported, depending on the frontmatter question.
- The index groups by kind in declared order, so the order is the project's.

## What it rules out

- A fixed set of kinds in the format. The format knows kinds only as names a
  project declares, each claiming a directory (spec §5.4, §7.1).
- Behaviour keyed to a kind's name. A kind called `research` gets no special
  treatment for being called that; what it does differently is configuration.
- Kinds that claim files by pattern, or several directories each, in version 1.
  One directory per kind, longest claim wins, and a kind at the docs root
  catches the rest; frontmatter `kind:` covers every exception found so far.

## Acceptance criteria

- [x] The decision is stated in one sentence under Recommendation
- [x] What it rules out is written down
- [x] Each of the three real folders is expressible without renaming a file

## 2026-09-22

Decided as recommended. Criterion 3 is shown by spec/corpus/real: code-as-color is 'decision' claiming decisions/ plus a root kind; poptop is guide, reference, design and roadmap by directory plus a root kind for its README; measure-of-the-world is 'plan' claiming plan/ plus a root kind. No file renamed. A kind whose dir is the docs root is the 'minimal' preset and the catch-all in the others.

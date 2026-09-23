---
id: 67
title: set a frontmatter key from the command line
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
effort: s
---

## Problem

Adopting poptop and code-as-color (0036) meant moving each row's note from the
hand-kept index into its page as `summary`, and its position as `order`: 37
pages, done with a script. There is no command for it. `supersede` edits
frontmatter safely (it keeps unknown keys, line endings and quoting), but only
for its own two keys.

## Proposal

`loam set PAGE key=value`, as `cairn set` is for items: the same surgical edit
`supersede` uses, for any key, with the format's own keys validated. An agent
writing research needs it as much as a person adopting a folder.

## Acceptance criteria

- [x] Sets, replaces and removes a key, and changes no other byte
- [x] Refuses a value of the wrong type for a key of the format

## 2026-09-23

loam set PAGE key=value, key= to remove, key+= and key-= for lists — cairn's syntax. Built on the same text edit supersede and review use, so an untouched key keeps its bytes and line endings (tests/agents.rs set_sets_replaces_and_removes_a_key_and_nothing_else, on a CRLF page). The format's keys are held to their types: status to the three, order to an integer, kind to a declared kind, title and the rest to one value; reviewed is refused in favour of loam review. A key the format does not define is written as typed, so weight=3 is a number.

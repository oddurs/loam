---
id: 67
title: set a frontmatter key from the command line
type: feature
status: backlog
milestone: v0.3
created: 2026-09-22
updated: 2026-09-22
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

- [ ] Sets, replaces and removes a key, and changes no other byte
- [ ] Refuses a value of the wrong type for a key of the format

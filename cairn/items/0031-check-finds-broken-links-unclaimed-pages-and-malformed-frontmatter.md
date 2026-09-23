---
id: 31
title: check finds broken links, unclaimed pages and malformed frontmatter
type: feature
status: backlog
milestone: v0.1
depends_on:
- 24
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p0
pillar:
- cli
- links
effort: m
---

## Problem

A docs folder's commonest defect is a link to a page that was renamed. Nothing
reports it until a reader clicks.

## Proposal

`loam check` reports, each with a path and line:

- links to pages or anchors that do not exist
- pages in no kind's directory, when the config says that is an error
- frontmatter that does not match the schema
- a superseded page with no `superseded_by`, and the reverse

Warnings by default, `--strict` to fail, `--json` for programs. The output
format is cairn's, so an editor's problem matcher for one works for the other.

## Acceptance criteria

- [ ] Each finding above has a corpus case
- [ ] Exit codes documented and tested
- [ ] Clean on the three adopted folders, or every finding is a real defect

---
id: 1
key: v0.0
title: Written down
type: milestone
status: backlog
created: 2026-09-22
updated: 2026-09-22
priority: p0
---

Nothing is built until the format is specified and the decisions that shape it are made.

This is the order cairn arrived at the hard way (its `0070`: change the
specification before changing the format). loam gets to start there. The cost
is a few weeks with nothing to run; the saving is not having to migrate
somebody's docs folder because a key was named in a hurry.

## What it claims

A reader for the page format can be written from `spec/README.md` without
consulting any program, and every decision the spec depends on is closed with
its reasoning in this backlog.

## Acceptance criteria

- [ ] Every decision and question in this milestone is closed
- [ ] `spec/README.md` is version 1
- [ ] The corpus holds the three real docs folders and passes the conformance script

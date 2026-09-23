---
id: 20
title: Settle the name
type: decision
status: backlog
milestone: v0.0
created: 2026-09-22
updated: 2026-09-22
priority: p1
effort: s
---

## Context

`loam` is a working name. cairn lost `cairn` on crates.io and ships as
`cairn-md`; harrow could not publish there at all. The name is cheap to change
now — one directory, one config key — and expensive after a release.

## What the name has to do

- Be short and typeable as a command: four or five letters.
- Be free on crates.io, in Homebrew, and as `oddurs/<name>` on GitHub.
- Not collide with a command already on a typical `$PATH`.
- Sit in the family: a cairn marks the trail, a harrow works the ground.

## Options

- **loam** — the soil the work grows in. What written context is to a project.
- **till** — glacial till is the ground stones are left in, and tilling is what
  a harrow finishes. The pun is good; the word is a common English conjunction,
  which makes it hard to search for.
- **tor**, **scree**, **moraine** — more of the landscape; `tor` collides with
  the Tor project.

## Recommendation

loam, if it is free. Check before deciding.

## Acceptance criteria

- [ ] Availability checked on crates.io, Homebrew and GitHub, and recorded here
- [ ] The decision is stated in one sentence under Recommendation
- [ ] The repository directory and `project.name` match it

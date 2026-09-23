---
id: 20
title: Settle the name
type: decision
status: done
milestone: v0.0
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

## Availability, checked 2026-09-22

| Where | `loam` | Notes |
| --- | --- | --- |
| crates.io | **taken** | An unrelated crate, "File-based tree storage", v0.7.0, maintained (last release August 2026). `loam-md` is free. So are none of the alternatives: `till`, `tor`, `scree` and `moraine` are all taken too. |
| Homebrew | free | No formula or cask named `loam` in homebrew-core. |
| GitHub | free | `oddurs/loam` did not exist. |
| `$PATH` | free | No `loam` on a macOS machine with Homebrew and a Rust toolchain installed. Not checked on Linux distributions. |

## Recommendation

**The program is `loam`, published on crates.io as `loam-md`, exactly as cairn
ships as `cairn-md`.**

The crates.io name was the only one not free, and losing it costs the same thing
it cost cairn: `cargo install loam-md` installs a binary called `loam`. Every
alternative on the list is also taken on crates.io, so moving off `loam` would
buy a worse word for no gain there. Nobody types a crate name more than once.

## What it rules out

- Renaming later without cause. After a release, the name is in people's
  configs, CI and muscle memory.
- A `-cli` or `-rs` suffix: `-md` is the family convention, and says what the
  thing is about.

## Acceptance criteria

- [x] Availability checked on crates.io, Homebrew and GitHub, and recorded here
- [x] The decision is stated in one sentence under Recommendation
- [x] The repository directory and `project.name` match it

## 2026-09-22

project.name is loam and the directory is ~/Code/loam; the GitHub repository is created as oddurs/loam in the same change that closes v0.0. The crate name only matters from 0027 (scaffold), which must use package name loam-md and [[bin]] name loam.

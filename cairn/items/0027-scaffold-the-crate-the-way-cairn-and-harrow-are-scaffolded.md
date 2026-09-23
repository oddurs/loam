---
id: 27
title: Scaffold the crate the way cairn and harrow are scaffolded
type: chore
status: backlog
milestone: v0.1
depends_on:
- 20
created: 2026-09-22
updated: 2026-09-22
priority: p1
pillar:
- cli
effort: s
---

## What

Rust, the same toolchain pin, `clippy -D warnings`, `deny.toml`, a `Makefile`
whose `check` target is what CI runs, CI on Linux and macOS. `cairn check` and
`cairn render --check` in CI for this backlog; later, `loam check` for loam's
own docs.

Copy, do not reinvent. Where harrow and cairn differ, pick one and write down
why in the Makefile.

## Acceptance criteria

- [ ] `make check` runs fmt, clippy, tests, deny, cairn check
- [ ] CI green on an empty `main` that prints its version

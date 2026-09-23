---
id: 27
title: Scaffold the crate the way cairn and harrow are scaffolded
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
depends_on:
- 20
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
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

- [x] `make check` runs fmt, clippy, tests, deny, cairn check
- [x] CI green on an empty `main` that prints its version

## 2026-09-22

A Makefile, not harrow's scripts/task: loam's CI needs cairn and Python as well as cargo, and the Makefile says why. make check runs fmt, clippy -D warnings, the tests, cargo-deny, both readers over the conformance corpus, spec/shared.py, cairn check --render --strict and loam's own check --strict --render. CI runs exactly make check on ubuntu-latest and macos-latest, green on 2026-09-23. The toolchain is harrow's pin, 1.98. cairn is built in CI at a pinned commit, since no cairn release yet knows closed_at.

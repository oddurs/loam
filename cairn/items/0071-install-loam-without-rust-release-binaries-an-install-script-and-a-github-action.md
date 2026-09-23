---
id: 71
title: 'Install loam without Rust: release binaries, an install script, and a GitHub Action'
type: feature
status: blocked
milestone: v0.4
created: 2026-09-23
updated: 2026-09-23
priority: p2
---

## Problem

Trying loam means installing Rust 1.98 and compiling it, and so does every CI
run of every project that checks its docs with it. That is minutes per run, and
a reason not to start.

## Proposal

A release workflow that builds loam for each platform on a `v*` tag and
publishes the archives with checksums; `install.sh`, which fetches one; and a
composite action at the repository root, `oddurs/loam@vX.Y.Z`, which installs
that release and runs `loam check --strict --render`.

## Costs

A release is now a tag, and the tag must match `Cargo.toml`'s version (the
workflow refuses otherwise). Windows is not built: loam has never been run there.

## Acceptance criteria

- [x] The workflow builds all four targets and each binary runs on its runner
- [x] install.sh installs an archive and refuses one that fails its checksum
- [ ] A release exists, and the action installs it and runs in a workflow

## 2026-09-23

Built by hand at 35934784965: four targets green, publish skipped. The Linux archive is a static-pie binary; the macOS arm64 one runs uv's check here. install.sh tested against a file:// copy of the archive, and with a wrong checksum. Waiting on a person: the first tag, v0.3.0, publishes a public release, which is theirs to decide. After it: a workflow in some repository using oddurs/loam@v0.3.0, to tick the last criterion.

---
id: 57
title: Is an MCP server worth it when the command line is this small?
type: question
status: backlog
milestone: later
depends_on:
- 46
- 49
created: 2026-09-22
updated: 2026-09-22
priority: p3
pillar:
- agents
effort: s
---

## Question

cairn serves its backlog over MCP. Does loam need to, or do `context`, `search`
and `new` from a shell cover it?

## Why it has to be answered first

An MCP server is a second interface to keep in step (cairn found its server
skipping checks the CLI enforced). It should earn that.

## Options

- None: the contract tells agents which commands to run.
- A thin server exposing `context`, `search`, `new`, `stale`.

## What would settle it

A month of agent sessions using only the CLI, noting every time a tool call would
have been better than a shell command.

## Answer

<!-- Filled in when the spike closes. -->

## Acceptance criteria

- [ ] Answer written, with the evidence that settled it

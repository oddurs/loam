---
id: 47
title: Pages an agent writes start as drafts
type: feature
status: done
milestone: v0.3
depends_on:
- 18
- 46
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
pillar:
- agents
effort: s
---

## Problem

Research an agent wrote an hour ago and a design a person reviewed last month
look identical in a docs folder, and are read with identical trust.

## Proposal

`agent_status = "draft"` in the config makes `loam new` from an agent — detected
the way cairn detects one, or declared by a flag — write `status: draft`.
Making it current is a person's edit to the frontmatter. The contract
says so.

Say plainly that this is a guard rail and not a boundary: an agent with a shell
can edit the frontmatter. cairn's manual makes the same admission and is better
for it.

## Acceptance criteria

- [x] Agent-created pages are drafts; person-created are not
- [x] The index marks drafts
- [x] The limitation is documented, in those words

## 2026-09-23

[agents] status = "draft", which init now writes. An agent is one when it says so — --agent NAME, LOAM_AGENT, or CAIRN_AGENT so one variable serves both programs — or when loam recognises one: AI_AGENT, which Claude Code and others have begun to set, and Claude Code's CLAUDECODE. That last part departs from cairn, which detects nothing; the reasoning is in src/agent.rs: a guard rail an agent must remember to turn on is mostly not there, and a false detection costs a draft a person promotes with one command. The limitation is documented in docs/guide/working-with-agents.md in the item's words: a guard rail and not a boundary, since an agent with a shell can edit the frontmatter.

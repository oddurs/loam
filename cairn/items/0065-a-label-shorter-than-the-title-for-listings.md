---
id: 65
title: A label shorter than the title, for listings
type: feature
status: backlog
milestone: later
created: 2026-09-22
updated: 2026-09-22
priority: p2
effort: s
---

## Problem

Adopting poptop (0036): its hand-kept index linked two pages by a shorter name
than their titles — "Output" for *Output for scripts and reports*, "Prior art"
for *Prior art, and where poptop actually differs*. The generated index uses
the title, so those two rows got longer. Nothing was lost, but a person had
chosen those labels.

## Proposal

An optional key, `label`, used wherever a page is listed rather than read: the
index, `list`. A format change of the kind spec §9 says costs no version.

## Costs

A second name to keep in step with the title. Worth it only if more folders
than poptop want it.

## Acceptance criteria

- [ ] A second adopted folder wants it, or this is dropped

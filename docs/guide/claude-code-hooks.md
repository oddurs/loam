---
order: 6
summary: Two hooks that bring a file's pages to Claude Code as it edits, and send it back to the pages it made stale before it stops.
covers:
  - docs/cookbook
reviewed:
  commit: f6c73a6296c30e690671aa74fe29b0b9155d56a8
  date: 2026-09-23
---

# Claude Code hooks

Two recipes, both scripts in [`docs/cookbook`](../cookbook/). Each needs `loam`
and `jq` on `PATH`.

## Context after an edit

[`loam-context.sh`](../cookbook/loam-context.sh) runs after Claude edits a
file, and hands Claude the output of `loam context` for it: the pages that
describe the file, and whether each is still true. Once per file per session,
so the same pages do not arrive twice.

It is a `PostToolUse` hook, which fires *after* the edit. A `PreToolUse` hook
would be earlier and better, but it can only allow or refuse an edit, not add
to what Claude knows — so the first edit to a file goes in without the pages,
and they arrive in time for the rest. (That is how Claude Code's documentation
describes the two events as this is written; if `PreToolUse` gains context, move
the script there.)

## Back to the pages before stopping

[`loam-stop.sh`](../cookbook/loam-stop.sh) runs when Claude is about to finish.
If its uncommitted changes touch code a page covers, it sends Claude back with
`loam stale --working-tree`'s list: update each page, or `loam review` it. A page
updated or reviewed counts as changed alongside the code, so the next stop
goes through. It sends Claude back once per turn at most — when the hook input
says a stop hook is already active, it lets Claude stop.

## Registering them

Copy the two scripts to `.claude/hooks/` and make them executable, then add to
`.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          { "type": "command", "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/loam-context.sh", "timeout": 30 }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          { "type": "command", "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/loam-stop.sh", "timeout": 60 }
        ]
      }
    ]
  }
}
```

## What was tested, and what was not

Both scripts were run with the input Claude Code's documentation says each
event receives, in a scratch repository with a page covering a changed file:
the context hook printed the `additionalContext` object once and nothing the
second time; the stop hook printed a `block` decision, printed nothing when the
stop hook was already active, and nothing when no covered file had changed.
They have not been run inside a live Claude Code session; the settings above
are as its documentation gives them.

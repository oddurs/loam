#!/bin/sh
# A Claude Code Stop hook: before Claude says it is done, name the pages its
# uncommitted changes have made stale, and send it back to them.
#
# Register it in .claude/settings.json; see claude-code-hooks.md beside this.
# Needs loam and jq on PATH.
input=$(cat)
# Already sent back once this turn: let it stop, rather than loop.
[ "$(printf '%s' "$input" | jq -r '.stop_hook_active // false')" = "true" ] && exit 0
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0

report=$(loam stale --working-tree 2>/dev/null)
# loam exits 1 when there are pages to reread, 0 when there are none.
[ $? -eq 1 ] || exit 0
jq -n --arg r "$report" '{decision: "block", reason: ("These pages describe code you changed. Update each one, or if it is still true, run `loam review` on it, then finish.\n\n" + $r)}'

#!/bin/sh
# A Claude Code PostToolUse hook: after Claude edits a file, tell it which pages
# describe that file — once per file per session, within a budget.
#
# Register it in .claude/settings.json; see claude-code-hooks.md beside this.
# Needs loam and jq on PATH.
input=$(cat)
file=$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')
session=$(printf '%s' "$input" | jq -r '.session_id // "none"')
[ -n "$file" ] || exit 0
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0

# Once per file per session: the same pages, again, are noise.
seen="${TMPDIR:-/tmp}/loam-context-$session"
grep -qxF "$file" "$seen" 2>/dev/null && exit 0
printf '%s\n' "$file" >> "$seen"

context=$(loam context "$file" --budget 6k 2>/dev/null) || exit 0
case "$context" in *"No page covers"*) exit 0 ;; esac
jq -n --arg c "$context" '{hookSpecificOutput: {hookEventName: "PostToolUse", additionalContext: $c}}'

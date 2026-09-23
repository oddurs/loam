---
order: 4
summary: Every code `loam check` reports, how severe it is by default, and what to do about it.
---

# Findings

The format defines most of these ([spec §6.5](../../spec/README.md#65-findings)),
and any reader of it reports them the same way. The last four are loam's own
judgements. Every severity can be changed in
[`[check.severity]`](loam-toml.md#checkseverity).

| Code | Default | Means | Do |
| --- | --- | --- | --- |
| `invalid-encoding` | error | The file is not UTF-8. Nothing in it can be read. | Save it as UTF-8. |
| `malformed-frontmatter` | error | Frontmatter never closes, is not YAML, or is not a mapping. | Close it with `---`, or fix the YAML. |
| `malformed-key` | warning | A key of the format has the wrong type, as `title: 2026` does. It is ignored. | Quote it: `title: "2026"`. |
| `unknown-status` | warning | A status other than `draft`, `current`, `superseded`. | Use one of those. `done` is for work, not pages. |
| `unknown-kind` | warning | A `kind` that `loam.toml` does not declare. | Declare it, or correct the page. |
| `unclaimed` | warning | No kind's directory holds the page, and it names no kind. | Add a kind at `.`, or move the page. |
| `untitled` | warning | No `title` and no level-1 heading. | Start the page with `# Its title`. |
| `broken-link` | warning | A link to a file or directory that does not exist, spelled exactly. | Fix it; the message says when only the case differs. |
| `broken-anchor` | warning | The file exists and has no such heading or anchor. | Link to the heading's slug, as GitHub makes it. |
| `escapes-repository` | warning | A relative link climbs out of the repository. | Link to it by URL instead. |
| `stale-cairn-link` | warning | A link to a cairn item under an old file name. | Point it at the current name, which the message gives. |
| `broken-cairn-link` | warning | A link to a cairn item that does not exist. | Fix the number. |
| `broken-supersession` | warning | `supersedes` or `superseded_by` names something that is not a page. | Fix the path. |
| `one-sided-supersession` | warning | One page says it replaces another, and the other does not say so. | `loam supersede` writes both sides. |
| `superseded-without-successor` | warning | `status: superseded`, with nothing named to read instead. | Add `superseded_by`. |
| `successor-without-superseded` | warning | `superseded_by` names a successor, but the status says otherwise. | Make it `superseded`, or remove the successor. |
| `link-to-superseded` | warning | A link to a superseded page, from anything but its successor. | Link to what replaced it. |
| `stale-index` | error | The index is not what `loam render` would write (with `--render`). | Run `loam render`. |

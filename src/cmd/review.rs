// loam review — record that a page was read against the code as it is now.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Clearing a stale page by hand means writing a commit's name into YAML, and
// nobody will. This writes `reviewed` — the commit at HEAD and today's date —
// and nothing else, so the review is one command and one line of a diff.

use super::Ctx;
use crate::covers::Covers;
use crate::git::Git;
use crate::write::{Lines, body_start, set_mapping, write_atomic};
use anyhow::{Result, bail};

#[derive(clap::Args)]
pub struct Args {
    /// The pages that were read against the code
    #[arg(required = true)]
    pub pages: Vec<String>,
    /// Why it is still true, kept in a `## Review log` at the end of the page
    #[arg(long, value_name = "TEXT")]
    pub note: Option<String>,
}

const LOG: &str = "## Review log";

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let (lock, tree) = ctx.locked_tree()?;
    let git = Git::open(&tree.config.root)?;
    let Some(head) = git.head() else {
        bail!("the repository has no commits yet; commit the code, then review against it")
    };
    let today = crate::fresh::today();
    let dirty = git.uncommitted()?;

    let mut writes = Vec::new();
    for typed in &args.pages {
        let path = ctx.repo_path(&tree.config, typed)?;
        let page = super::page(&tree, &path)?;
        let Ok(text) = std::str::from_utf8(&page.bytes) else {
            bail!("{path} is not UTF-8, so loam will not rewrite it")
        };
        if !page.is_readable() {
            bail!("{path} cannot be read; run `loam check` to see why");
        }
        // A review against code no commit holds yet is a review of nothing
        // anybody else can check out. Said, not refused: the author may be
        // about to commit the two together.
        let covers = Covers::new(&page.covers);
        let unsaved: Vec<&String> = dirty.iter().filter(|p| covers.covers(p)).collect();
        if !unsaved.is_empty() {
            eprintln!(
                "{} {path} covers {} with uncommitted changes; this review is of code no commit holds yet:",
                crate::style::yellow("warning:"),
                if unsaved.len() == 1 {
                    "a file"
                } else {
                    "files"
                }
            );
            for p in unsaved.iter().take(5) {
                eprintln!("  {p}");
            }
        }
        let mut lines = Lines::parse(text);
        set_mapping(
            &mut lines,
            "reviewed",
            &[("commit", head.clone()), ("date", today.clone())],
        );
        if let Some(note) = &args.note {
            let entry = format!("- {today}, at {}: {}", &head[..7], note.trim());
            let start = body_start(&lines);
            match (start..lines.lines.len()).find(|&i| lines.lines[i].0.trim_end() == LOG) {
                // After the log's last entry: the end of its section, which is
                // the next heading of its level or above, or the end of the page.
                Some(log) => {
                    let next = (log + 1..lines.lines.len())
                        .find(|&i| {
                            let l = &lines.lines[i].0;
                            l.starts_with("# ") || l.starts_with("## ")
                        })
                        .unwrap_or(lines.lines.len());
                    let mut at = next;
                    while at > log + 1 && lines.lines[at - 1].0.trim().is_empty() {
                        at -= 1;
                    }
                    if at < next {
                        lines.insert(at, &[entry]);
                    } else {
                        // Nothing between the entry and what follows: keep a
                        // blank line before the next heading.
                        lines.insert(at, &[entry, String::new()]);
                    }
                }
                None => {
                    let mut at = lines.lines.len();
                    while at > start && lines.lines[at - 1].0.trim().is_empty() {
                        at -= 1;
                    }
                    lines.insert(at, &[String::new(), LOG.to_string(), String::new(), entry]);
                }
            }
        }
        writes.push((path, lines.render()));
    }

    {
        for (path, text) in &writes {
            write_atomic(&tree.config.abs(path), text.as_bytes())?;
        }
    }
    for (path, _) in &writes {
        println!("reviewed {path} against {} ({today})", &head[..7]);
    }
    drop(lock);
    ctx.after_change(&tree.config);
    Ok(0)
}

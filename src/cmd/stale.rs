// loam stale — the pages whose code has moved on without them.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Sorted by how much changed, not by how long ago: a page three months old
// whose code changed by one line matters less than one a week old whose code
// was rewritten. A page with no `covers` is listed as unknown, apart, and never
// as fresh — silence would be a claim.

use super::Ctx;
use crate::fresh::{self, Freshness, How, Options, State};
use crate::git::Git;
use anyhow::Result;
use serde_json::{Value, json};

#[derive(clap::Args)]
pub struct Args {
    /// Print JSON on standard output
    #[arg(long)]
    pub json: bool,
    /// Judge as the repository was at this commit, with the pages as they are
    /// now — for replaying history
    #[arg(long, value_name = "REV")]
    pub at: Option<String>,
    /// Also list the fresh and unknown pages by name
    #[arg(short, long)]
    pub all: bool,
    /// Instead: the pages covering files with uncommitted changes — what to
    /// reread before stopping
    #[arg(long, conflicts_with = "at")]
    pub working_tree: bool,
    /// Instead: the pages covering files changed since REV
    #[arg(long, value_name = "REV", conflicts_with = "at")]
    pub since: Option<String>,
}

/// `--working-tree` and `--since`: not what history says, but what a change
/// just made — for an agent's stop hook, before it says it is done.
fn touched(ctx: &Ctx, args: &Args) -> Result<u8> {
    let tree = ctx.tree()?;
    let git = Git::open(&tree.config.root)?;
    let changed = match &args.since {
        Some(rev) => {
            if git.resolve(rev).is_none() {
                anyhow::bail!("no commit called {rev}");
            }
            let mut c = git.changed_since(rev)?;
            if args.working_tree {
                c.extend(git.uncommitted()?);
            }
            c
        }
        None => git.uncommitted()?,
    };
    let pages = fresh::touched(&tree, &changed);
    let reread: Vec<_> = pages.iter().filter(|t| !t.page_changed).collect();
    if args.json {
        let v = json!({
            "changed": changed,
            "pages": pages.iter().map(|t| json!({
                "path": t.path,
                "state": if t.page_changed { "updating" } else { "reread" },
                "patterns": t.patterns.iter().map(|(p, files)| json!({"pattern": p, "files": files})).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(u8::from(!reread.is_empty()));
    }
    let what = if args.since.is_some() {
        "since then"
    } else {
        "and not yet committed"
    };
    if pages.is_empty() {
        println!("no page covers what changed {what}");
        return Ok(0);
    }
    for t in &pages {
        let files: Vec<String> = t.patterns.iter().flat_map(|(_, f)| f.clone()).collect();
        let shown = if files.len() > 3 {
            format!("{} and {} more", files[..3].join(", "), files.len() - 3)
        } else {
            files.join(", ")
        };
        if t.page_changed {
            println!(
                "{}   covers {shown}, and changed with it",
                crate::style::dim(&t.path)
            );
        } else {
            println!(
                "{}   covers {shown}, changed {what}",
                crate::style::bold(&t.path)
            );
            println!(
                "  update it, or if it is still true: loam review {}",
                t.path
            );
        }
    }
    println!();
    println!(
        "{} page(s) to reread, {} changed alongside",
        reread.len(),
        pages.len() - reread.len()
    );
    Ok(u8::from(!reread.is_empty()))
}

fn ago(days: Option<i64>) -> String {
    match days {
        None => "at an unknown time".into(),
        Some(0) => "today".into(),
        Some(1) => "yesterday".into(),
        Some(d) => format!("{d} days ago"),
    }
}

pub fn describe(f: &Freshness) -> String {
    let Some(b) = &f.baseline else {
        return String::new();
    };
    let short = |c: &Option<String>| {
        c.as_deref()
            .map(|c| c[..c.len().min(7)].to_string())
            .unwrap_or_default()
    };
    match b.how {
        How::Reviewed => format!("reviewed {} at {}", ago(f.age), short(&b.commit)),
        How::Introduced => format!(
            "reviewed {}, on a commit squashed or rebased since",
            ago(f.age)
        ),
        How::Dated => format!(
            "reviewed {}, on a commit this repository does not have",
            ago(f.age)
        ),
        How::LastEdit => format!(
            "never reviewed; last edited {} at {}",
            ago(f.age),
            short(&b.commit)
        ),
        How::New => "not yet committed".into(),
    }
}

pub fn freshness_json(f: &Freshness) -> Value {
    let state = match f.state {
        State::Generated => "generated",
        State::Unknown => "unknown",
        State::Fresh => "fresh",
        State::Stale => "stale",
        State::Updating => "updating",
    };
    json!({
        "path": f.path,
        "state": state,
        "baseline": f.baseline.as_ref().map(|b| json!({
            "how": match b.how {
                How::Reviewed => "reviewed",
                How::Introduced => "introduced",
                How::Dated => "dated",
                How::LastEdit => "last-edit",
                How::New => "new",
            },
            "commit": b.commit,
            "date": b.date,
        })),
        "age": f.age,
        "stale_after": f.aged,
        "added": f.added,
        "removed": f.removed,
        "patterns": f.patterns.iter().map(|p| json!({
            "pattern": p.pattern, "commits": p.commits, "added": p.added, "removed": p.removed, "latest": p.latest,
        })).collect::<Vec<_>>(),
        "commits": f.commits.iter().map(|c| json!({
            "commit": c.sha, "date": c.date, "subject": c.subject,
        })).collect::<Vec<_>>(),
    })
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    if args.working_tree || args.since.is_some() {
        return touched(ctx, &args);
    }
    let tree = ctx.tree()?;
    let git = Git::open(&tree.config.root)?;
    let today = match &args.at {
        // Replaying: ages as of that commit, not as of today.
        Some(rev) => Some(git_date(&git, rev)?),
        None => None,
    };
    let report = fresh::assess(
        &tree,
        &git,
        &Options {
            at: args.at.as_deref(),
            today,
        },
    )?;
    let count = |s: State| report.pages.iter().filter(|f| f.state == s).count();
    let stale = count(State::Stale);

    if args.json {
        let v = json!({
            "notes": report.notes,
            "pages": report.pages.iter().map(freshness_json).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(u8::from(stale > 0));
    }

    for note in &report.notes {
        eprintln!("{} {note}", crate::style::yellow("note:"));
    }
    let bold = crate::style::bold;
    let show = |f: &Freshness| {
        println!("{}   {}", bold(&f.path), describe(f));
        let width = f
            .patterns
            .iter()
            .map(|p| p.pattern.chars().count())
            .max()
            .unwrap_or(0);
        for p in &f.patterns {
            println!(
                "  {:width$}  {} commit(s), +{} −{}   \"{}\"",
                p.pattern, p.commits, p.added, p.removed, p.latest
            );
        }
        if let Some(limit) = f.aged {
            println!("  older than its kind's {limit} days");
        }
    };
    for f in report.pages.iter().filter(|f| f.state == State::Stale) {
        show(f);
    }
    let updating: Vec<_> = report
        .pages
        .iter()
        .filter(|f| f.state == State::Updating)
        .collect();
    if !updating.is_empty() {
        println!();
        println!(
            "{}",
            crate::style::dim("changed alongside its code, so probably being updated:")
        );
        for f in updating {
            show(f);
        }
    }
    let unknown: Vec<_> = report
        .pages
        .iter()
        .filter(|f| f.state == State::Unknown)
        .collect();
    if args.all {
        for (label, state) in [("fresh", State::Fresh), ("generated", State::Generated)] {
            for f in report.pages.iter().filter(|f| f.state == state) {
                println!("{}  {label}", crate::style::dim(&f.path));
            }
        }
        for f in &unknown {
            println!("{}  unknown: no covers", crate::style::dim(&f.path));
        }
    }
    if stale > 0 || !args.all {
        println!();
    }
    println!(
        "{stale} stale, {} being updated, {} fresh, {} unknown (no covers), {} generated",
        count(State::Updating),
        count(State::Fresh),
        unknown.len(),
        count(State::Generated)
    );
    Ok(u8::from(stale > 0))
}

fn git_date(git: &Git, rev: &str) -> Result<String> {
    git.date_of(rev)
        .ok_or_else(|| anyhow::anyhow!("no commit called {rev}"))
}

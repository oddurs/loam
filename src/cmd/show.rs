// loam show — one page: what loam read from it, and the page itself.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use super::Ctx;
use crate::fresh::Freshness;
use crate::tree::Tree;
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};

#[derive(clap::Args)]
pub struct Args {
    /// The page, relative to here, to the repository, or to the docs root
    pub path: String,
    /// Print JSON on standard output: the page's reading and its body
    #[arg(long)]
    pub json: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    let path = ctx.repo_path(&tree.config, &args.path)?;
    let Some(page) = tree.pages.get(&path) else {
        bail!(
            "{path} is not a page (a page is a .md file under {}/)",
            tree.config.docs
        );
    };
    // Whether it is still true, when there is a history to ask.
    let (mut freshness, trouble) = freshness(&tree, &HashSet::from([path.clone()]));
    let freshness = freshness.remove(&path);
    if args.json {
        let mut v = crate::reading::page_json(page);
        v["path"] = serde_json::json!(page.path);
        v["body"] = serde_json::json!(page.body);
        v["freshness"] = freshness
            .as_ref()
            .map_or(serde_json::Value::Null, super::stale::freshness_json);
        v["freshness_error"] = serde_json::json!(trouble);
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(0);
    }
    let from =
        |f: Option<&str>| f.map_or_else(String::new, |f| crate::style::dim(&format!("  ({f})")));
    println!(
        "{}",
        crate::style::bold(page.title.as_deref().unwrap_or("(untitled)"))
    );
    println!("     path  {}", page.path);
    println!(
        "     kind  {}{}",
        page.kind.as_deref().unwrap_or("?"),
        from(page.kind_from)
    );
    println!("   status  {}{}", page.status, from(Some(page.status_from)));
    if let Some(s) = &page.summary {
        println!("  summary  {s}{}", from(page.summary_from));
    }
    for (label, list) in [
        ("supersedes", &page.supersedes),
        ("replaced by", &page.superseded_by),
    ] {
        if !list.is_empty() {
            println!("{label:>9}  {}", list.join(", "));
        }
    }
    if let Some(f) = &freshness {
        use crate::fresh::State;
        let state = match f.state {
            State::Generated => "generated, so kept true by its generator".to_string(),
            State::Unknown => "unknown: the page says nothing about what it covers".to_string(),
            State::Fresh => format!("fresh; {}", super::stale::describe(f)),
            State::Stale | State::Updating => {
                let n = f.commits.len();
                let what = if f.aged.is_some() && n == 0 {
                    format!("older than its kind's {} days", f.aged.unwrap_or(0))
                } else {
                    format!(
                        "{n} commit(s) changed what it covers, +{} −{}",
                        f.added, f.removed
                    )
                };
                let prefix = if f.state == State::Stale {
                    "stale"
                } else {
                    "being updated"
                };
                format!("{prefix}: {what}; {}", super::stale::describe(f))
            }
        };
        println!("    fresh  {state}");
    }
    if let Some(e) = &trouble {
        println!("    fresh  could not be read: {e}");
    }
    if !page.covers.is_empty() {
        println!("   covers  {}", page.covers.join(", "));
    }
    // A research page's sources, and when each was read (0043): a loam
    // convention, not a key of the format.
    if let Some(sources) = page
        .frontmatter
        .as_ref()
        .and_then(|m| m.iter().find(|(k, _)| k == "sources"))
        .map(|(_, v)| v)
    {
        let list = match sources {
            crate::yaml::Yaml::Seq(items) => items.clone(),
            other => vec![other.clone()],
        };
        for (i, source) in list.iter().enumerate() {
            let label = if i == 0 { "  sources" } else { "         " };
            let text = match source {
                crate::yaml::Yaml::Str(s) => s.clone(),
                crate::yaml::Yaml::Map(m) => {
                    let get = |k: &str| m.iter().find(|(key, _)| key == k).map(|(_, v)| v);
                    let s = |v: Option<&crate::yaml::Yaml>| match v {
                        Some(crate::yaml::Yaml::Str(s)) => Some(s.clone()),
                        _ => None,
                    };
                    let name = s(get("title"))
                        .or(s(get("url")))
                        .unwrap_or_else(|| "(a source)".into());
                    match s(get("read")) {
                        Some(read) => format!("{name}, read {read}"),
                        None => name,
                    }
                }
                _ => continue,
            };
            println!("{label}  {text}");
        }
    }
    let broken = page
        .findings
        .iter()
        .filter(|f| f.code.contains("link") || f.code == "broken-anchor")
        .count();
    println!("    links  {} out, {broken} broken", page.links.len());
    for f in &page.findings {
        println!(
            "  {} line {}: {}",
            crate::style::yellow("finding"),
            f.line,
            crate::cmd::check::message(f)
        );
    }
    println!();
    print!("{}", page.body.trim_start_matches('\n'));
    if !page.body.ends_with('\n') {
        println!();
    }
    Ok(0)
}

/// Whether each of `only` is still true, and what went wrong if that could not
/// be read. Not being in a git repository is not something going wrong: there
/// is just no history to ask.
pub fn freshness(
    tree: &Tree,
    only: &HashSet<String>,
) -> (HashMap<String, Freshness>, Option<String>) {
    let Ok(git) = crate::git::Git::open(&tree.config.root) else {
        return (HashMap::new(), None);
    };
    let opts = crate::fresh::Options {
        at: None,
        today: None,
        only: Some(only),
    };
    match crate::fresh::assess(tree, &git, &opts) {
        Ok(report) => (
            report
                .pages
                .into_iter()
                .map(|f| (f.path.clone(), f))
                .collect(),
            None,
        ),
        Err(e) => (HashMap::new(), Some(format!("{e:#}"))),
    }
}

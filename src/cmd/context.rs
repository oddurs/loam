// loam context — the pages that matter for a file, within a budget.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// An agent about to change `src/engine/scheduler.rs` does not know that a page
// argues against exactly the change it is about to make. This tells it, before
// the change, and within a budget it names — because context an agent cannot
// afford to read is context it will not read.
//
// Three rings, closest first: pages whose `covers` match the paths; pages that
// name a path in their text; pages the covering pages link to. Each carries its
// freshness. A stale page is flagged, never left out: being told a page is
// stale is itself context.
//
// Budgets are bytes, not tokens. A token count depends on a tokenizer loam has
// no business shipping; English runs about four bytes to a token.

use super::Ctx;
use crate::covers::Covers;
use crate::fresh::{Freshness, State};
use crate::tree::{LinkClass, Page, Tree};
use anyhow::{Result, bail};
use serde_json::json;
use std::collections::HashMap;

#[derive(clap::Args)]
pub struct Args {
    /// Files you are about to read or change
    #[arg(required = true, value_name = "PATH")]
    pub paths: Vec<String>,
    /// The most to print, in bytes: `8k` is 8192. About four bytes a token
    #[arg(long, default_value = "8k")]
    pub budget: String,
    /// Print JSON on standard output
    #[arg(long)]
    pub json: bool,
}

pub fn parse_budget(text: &str) -> Result<usize> {
    let t = text.trim().to_lowercase();
    let (n, mult) = match t.strip_suffix('k') {
        Some(n) => (n, 1024),
        None => (t.as_str(), 1),
    };
    match n.parse::<usize>() {
        Ok(v) if v > 0 => Ok(v * mult),
        _ => bail!("a budget is a number of bytes, as 8000 or 8k, not `{text}`"),
    }
}

pub struct Hit<'a> {
    pub page: &'a Page,
    pub why: String,
    rank: (u8, i64),
}

/// The pages that matter for these paths, closest first.
pub fn gather<'a>(tree: &'a Tree, paths: &[String]) -> Vec<Hit<'a>> {
    let mut hits: Vec<Hit> = Vec::new();
    for page in tree.pages.values() {
        if page.path == tree.config.index {
            continue;
        }
        let covers = Covers::new(&page.covers);
        let matched: Vec<(&String, &str)> = paths
            .iter()
            .filter_map(|p| covers.which(p).map(|w| (p, w)))
            .collect();
        if !matched.is_empty() {
            let longest = matched.iter().map(|(_, w)| w.len()).max().unwrap_or(0) as i64;
            let patterns: Vec<&str> = matched.iter().map(|(_, w)| *w).collect();
            hits.push(Hit {
                page,
                why: format!("covers {}", patterns.join(", ")),
                // More paths matched first, then the more specific pattern.
                rank: (0, -(matched.len() as i64 * 1000 + longest)),
            });
            continue;
        }
        let text = page.text.as_deref().unwrap_or("");
        let named: Vec<&String> = paths
            .iter()
            .filter(|p| !p.is_empty() && text.contains(p.as_str()))
            .collect();
        if !named.is_empty() {
            hits.push(Hit {
                page,
                why: format!(
                    "mentions {}",
                    named
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                rank: (1, -(named.len() as i64)),
            });
        }
    }
    // Pages the covering pages link to, most-linked first.
    let mut linked: HashMap<&str, Vec<&str>> = HashMap::new();
    for h in hits.iter().filter(|h| h.rank.0 == 0) {
        for l in h.page.links.iter().filter(|l| l.class == LinkClass::Page) {
            if let Some(t) = l.target.as_deref()
                && t != h.page.path
                && t != tree.config.index
            {
                let from = linked.entry(t).or_default();
                if !from.contains(&h.page.path.as_str()) {
                    from.push(&h.page.path);
                }
            }
        }
    }
    for (target, from) in linked {
        if hits.iter().any(|h| h.page.path == target) {
            continue;
        }
        if let Some(page) = tree.pages.get(target) {
            hits.push(Hit {
                page,
                why: format!("linked from {}", from.join(", ")),
                rank: (2, -(from.len() as i64)),
            });
        }
    }
    hits.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.page.path.cmp(&b.page.path)));
    hits
}

fn freshness_note(f: Option<&Freshness>) -> String {
    match f {
        None => String::new(),
        Some(f) => match f.state {
            State::Stale | State::Updating => {
                let what = if f.commits.is_empty() {
                    "older than its kind allows".to_string()
                } else {
                    format!(
                        "{} commit(s) changed what it covers since it was last read",
                        f.commits.len()
                    )
                };
                format!("; STALE: {what} — trust it less, and check it against the code")
            }
            State::Fresh => "; fresh".into(),
            State::Unknown => "; freshness unknown (no covers)".into(),
            State::Generated => "; generated".into(),
        },
    }
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    let budget = parse_budget(&args.budget)?;
    let paths: Vec<String> = args
        .paths
        .iter()
        .map(|p| ctx.repo_path(&tree.config, p))
        .collect::<Result<_>>()?;
    let hits = gather(&tree, &paths);
    let freshness: HashMap<String, Freshness> = crate::git::Git::open(&tree.config.root)
        .ok()
        .and_then(|git| {
            crate::fresh::assess(
                &tree,
                &git,
                &crate::fresh::Options {
                    at: None,
                    today: None,
                },
            )
            .ok()
        })
        .map(|r| r.pages.into_iter().map(|f| (f.path.clone(), f)).collect())
        .unwrap_or_default();

    // Room kept for saying what was left out.
    const TRAILER: usize = 160;
    let mut out = format!(
        "# loam context for {} (within {budget} bytes)\n",
        paths.join(", ")
    );
    let mut included: Vec<(&Hit, &'static str)> = Vec::new();
    let mut left: Vec<&Hit> = Vec::new();
    for h in &hits {
        let p = h.page;
        let title = p.title.as_deref().unwrap_or("(untitled)");
        let head = format!(
            "\n## {} — {}\n({}{})\n",
            p.path,
            title,
            h.why,
            freshness_note(freshness.get(&p.path))
        );
        let body = format!("{head}\n{}\n", p.body.trim());
        let short = format!(
            "{head}{}\n",
            p.summary
                .as_deref()
                .map_or(String::new(), |s| format!("{s}\n"))
        );
        if out.len() + body.len() + TRAILER <= budget {
            out.push_str(&body);
            included.push((h, "full"));
        } else if out.len() + short.len() + TRAILER <= budget {
            out.push_str(&short);
            included.push((h, "summary"));
        } else {
            left.push(h);
        }
    }
    if hits.is_empty() {
        out.push_str("\nNo page covers, mentions or is linked from a page covering these paths.\n");
        out.push_str("If one should, give it `covers` (`loam set <PAGE> covers+=<PATH>`).\n");
    } else if !left.is_empty() {
        let mut trailer = format!("\nleft out, over the budget ({}):", left.len());
        for h in &left {
            let next = format!(" {}", h.page.path);
            if out.len() + trailer.len() + next.len() + 5 > budget {
                trailer.push_str(" …");
                break;
            }
            trailer.push_str(&next);
        }
        trailer.push('\n');
        out.push_str(&trailer);
    }
    // Never past the budget, whatever the pages were.
    if out.len() > budget {
        let mut cut = budget;
        while !out.is_char_boundary(cut) {
            cut -= 1;
        }
        out.truncate(cut);
    }

    if args.json {
        let pages: Vec<_> = hits
            .iter()
            .map(|h| {
                let how = included
                    .iter()
                    .find(|(i, _)| std::ptr::eq(*i, h))
                    .map_or("left out", |(_, how)| *how);
                json!({
                    "path": h.page.path,
                    "title": h.page.title,
                    "summary": h.page.summary,
                    "why": h.why,
                    "freshness": freshness.get(&h.page.path).map(super::stale::freshness_json),
                    "included": how,
                    "body": if how == "full" { Some(h.page.body.trim()) } else { None },
                })
            })
            .collect();
        let v = json!({ "paths": paths, "budget": budget, "used": out.len(), "pages": pages });
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(0);
    }
    print!("{out}");
    Ok(0)
}

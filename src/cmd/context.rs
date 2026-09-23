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
use std::collections::{HashMap, HashSet};

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
        Ok(v) if v * mult >= MIN_BUDGET => Ok(v * mult),
        Ok(_) => bail!(
            "a budget under {MIN_BUDGET} bytes cannot hold a page's heading and the names of those left out"
        ),
        _ => bail!("a budget is a number of bytes, as 8000 or 8k, not `{text}`"),
    }
}

/// The smallest budget worth printing: a header, a page's heading and
/// summary, or a note naming what was left out.
pub const MIN_BUDGET: usize = 200;

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
    // Whether each is still true, asked only of the pages found. Not being in
    // a git repository is no error; failing to read one is said, since a page
    // shown without its staleness would read as fresh.
    let only: HashSet<String> = hits.iter().map(|h| h.page.path.clone()).collect();
    let (freshness, trouble) = super::show::freshness(&tree, &only);

    let mut out = format!(
        "# loam context for {} (within {budget} bytes)\n",
        paths.join(", ")
    );
    if let Some(e) = &trouble {
        out.push_str(&format!(
            "(whether these pages are still true could not be read: {e})\n"
        ));
    }
    if out.len() + 40 > budget {
        bail!(
            "the paths are too long to name within {budget} bytes; pass fewer, or a larger --budget"
        );
    }
    let parts: Vec<(String, String)> = hits
        .iter()
        .map(|h| {
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
            (body, short)
        })
        .collect();
    // Fill the budget, keeping back `reserve` bytes for the note of what was
    // left out: nothing when everything fits, and when something does not,
    // room for the note however few names it then has space for.
    let fill = |reserve: usize| {
        let mut used = out.len();
        let mut how: Vec<Option<&'static str>> = Vec::new();
        for (body, short) in &parts {
            if used + body.len() + reserve <= budget {
                used += body.len();
                how.push(Some("full"));
            } else if used + short.len() + reserve <= budget {
                used += short.len();
                how.push(Some("summary"));
            } else {
                how.push(None);
            }
        }
        how
    };
    let mut how = fill(0);
    if how.iter().any(Option::is_none) {
        how = fill(format!("\nleft out, over the budget ({}): …\n", hits.len()).len());
    }
    let mut left: Vec<&Hit> = Vec::new();
    for ((h, (body, short)), how) in hits.iter().zip(&parts).zip(&how) {
        match how {
            Some("full") => out.push_str(body),
            Some(_) => out.push_str(short),
            None => left.push(h),
        }
    }
    if hits.is_empty() {
        out.push_str("\nNo page covers, mentions or is linked from a page covering these paths.\n");
        out.push_str("If one should, give it `covers` (`loam set <PAGE> covers+=<PATH>`).\n");
    } else if !left.is_empty() {
        let mut trailer = format!("\nleft out, over the budget ({}):", left.len());
        for (i, h) in left.iter().enumerate() {
            let next = format!(" {}", h.page.path);
            let more = if i + 1 < left.len() { " …".len() } else { 0 };
            if out.len() + trailer.len() + next.len() + more + 1 > budget {
                trailer.push_str(" …");
                break;
            }
            trailer.push_str(&next);
        }
        trailer.push('\n');
        out.push_str(&trailer);
    }
    debug_assert!(out.len() <= budget || hits.is_empty());

    if args.json {
        let pages: Vec<_> = hits
            .iter()
            .map(|h| {
                let i = hits
                    .iter()
                    .position(|o| std::ptr::eq(o, h))
                    .expect("one of them");
                let how = how[i].unwrap_or("left out");
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
        let v = json!({
            "paths": paths,
            "budget": budget,
            "used": out.len(),
            "freshness_error": trouble,
            "pages": pages,
        });
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(0);
    }
    print!("{out}");
    Ok(0)
}

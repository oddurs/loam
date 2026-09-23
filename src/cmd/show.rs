// loam show — one page: what loam read from it, and the page itself.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use super::Ctx;
use anyhow::{Result, bail};

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
    if args.json {
        let mut v = crate::reading::page_json(page);
        v["path"] = serde_json::json!(page.path);
        v["body"] = serde_json::json!(page.body);
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

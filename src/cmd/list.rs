// loam list — the pages, by kind and status.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use super::Ctx;
use crate::tree::Page;
use anyhow::Result;
use serde_json::{Value, json};

#[derive(clap::Args)]
pub struct Args {
    /// Only pages of this kind
    #[arg(long)]
    pub kind: Option<String>,
    /// Only pages with this status: draft, current or superseded
    #[arg(long)]
    pub status: Option<String>,
    /// Print JSON on standard output
    #[arg(long)]
    pub json: bool,
}

/// What a listing says about a page, in `list --json` and `search --json`.
pub fn summary_json(page: &Page) -> Value {
    json!({
        "path": page.path,
        "title": page.title,
        "kind": page.kind,
        "status": page.status,
        "summary": page.summary,
    })
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    if let Some(k) = &args.kind {
        super::kind(&tree.config, k)?;
    }
    let pages: Vec<&Page> = tree
        .pages
        .values()
        .filter(|p| {
            args.kind
                .as_ref()
                .is_none_or(|k| p.kind.as_ref() == Some(k))
        })
        .filter(|p| args.status.as_ref().is_none_or(|s| &p.status == s))
        .collect();
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &pages.iter().map(|p| summary_json(p)).collect::<Vec<_>>()
            )?
        );
        return Ok(0);
    }
    let width = |f: &dyn Fn(&Page) -> usize, head: usize| {
        pages.iter().map(|p| f(p)).max().unwrap_or(0).max(head)
    };
    let wp = width(&|p| p.path.chars().count(), 4);
    let wk = width(&|p| p.kind.as_deref().unwrap_or("?").chars().count(), 4);
    let ws = width(&|p| p.status.chars().count(), 6);
    println!(
        "{}",
        crate::style::bold(&format!(
            "{:wp$}  {:wk$}  {:ws$}  TITLE",
            "PATH", "KIND", "STATUS"
        ))
    );
    for p in &pages {
        println!(
            "{:wp$}  {:wk$}  {:ws$}  {}",
            p.path,
            p.kind.as_deref().unwrap_or("?"),
            p.status,
            p.title.as_deref().unwrap_or("(untitled)")
        );
    }
    Ok(0)
}

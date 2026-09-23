// loam search — what is already written about something.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// A scan of the files, every time: no index is kept on disk (the boundary,
// cairn 0019). Matches in a title rank above matches in a summary, which rank
// above matches in the body, because a page named for the thing is the page to
// read first.

use super::Ctx;
use crate::tree::Page;
use anyhow::Result;
use serde_json::json;

#[derive(clap::Args)]
pub struct Args {
    /// Words to look for; a page matches when it contains every one, in any case
    #[arg(required = true)]
    pub words: Vec<String>,
    /// Only pages of this kind
    #[arg(long)]
    pub kind: Option<String>,
    /// Print JSON on standard output
    #[arg(long)]
    pub json: bool,
}

pub struct Hit<'a> {
    pub page: &'a Page,
    pub rank: &'static str,
    pub line: Option<(usize, String)>,
}

pub fn search<'a>(pages: impl Iterator<Item = &'a Page>, words: &[String]) -> Vec<Hit<'a>> {
    let words: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let all = |s: &str| {
        let s = s.to_lowercase();
        words.iter().all(|w| s.contains(w.as_str()))
    };
    let any = |s: &str| {
        let s = s.to_lowercase();
        words.iter().any(|w| s.contains(w.as_str()))
    };
    let mut hits: Vec<Hit> = Vec::new();
    for page in pages {
        let title = page.title.as_deref().unwrap_or("");
        let summary = page.summary.as_deref().unwrap_or("");
        let text = page.text.as_deref().unwrap_or("");
        let whole = format!("{title}\n{summary}\n{}\n{text}", page.path);
        if !all(&whole) {
            continue;
        }
        let rank = if all(title) {
            "title"
        } else if all(&format!("{title} {summary}")) {
            "summary"
        } else {
            "body"
        };
        let line = text
            .split('\n')
            .enumerate()
            .skip(page.body_line.saturating_sub(1))
            .find(|(_, l)| any(l) && !l.trim_start().starts_with('#'))
            .map(|(i, l)| (i + 1, l.trim().to_string()));
        hits.push(Hit { page, rank, line });
    }
    let order = |r: &str| {
        ["title", "summary", "body"]
            .iter()
            .position(|x| *x == r)
            .unwrap_or(3)
    };
    hits.sort_by(|a, b| (order(a.rank), &a.page.path).cmp(&(order(b.rank), &b.page.path)));
    hits
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    let pages = tree.pages.values().filter(|p| {
        args.kind
            .as_ref()
            .is_none_or(|k| p.kind.as_ref() == Some(k))
    });
    let hits = search(pages, &args.words);
    if args.json {
        let list: Vec<_> = hits
            .iter()
            .map(|h| {
                let mut v = super::list::summary_json(h.page);
                v["rank"] = json!(h.rank);
                v["line"] = json!(h.line.as_ref().map(|l| l.0));
                v["text"] = json!(h.line.as_ref().map(|l| l.1.clone()));
                v
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(u8::from(hits.is_empty()));
    }
    if hits.is_empty() {
        eprintln!("loam: nothing written matches");
        return Ok(1);
    }
    for h in &hits {
        println!(
            "{}  {}",
            crate::style::bold(&h.page.path),
            h.page.title.as_deref().unwrap_or("")
        );
        if let Some((n, text)) = &h.line {
            let text: String = text.chars().take(100).collect();
            println!("  {}", crate::style::dim(&format!("{n}: {text}")));
        }
    }
    Ok(0)
}

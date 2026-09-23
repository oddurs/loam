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
    /// How well it matches: a word in the title counts most, then in the
    /// summary or the path, then each time it appears in the body.
    pub score: usize,
    /// How many of the words it contains, for the closest pages when none
    /// contains all of them.
    pub matched: usize,
}

/// The words to look for. A quoted phrase is still words: `"docker cache"`
/// finds a page about caching in Docker however it says so.
pub fn words(args: &[String]) -> Vec<String> {
    args.iter()
        .flat_map(|a| a.split_whitespace())
        .map(str::to_lowercase)
        .collect()
}

/// Every page containing every word, best first; or, when none does and
/// `closest` is set, those containing the most of them.
pub fn search<'a>(
    pages: impl Iterator<Item = &'a Page>,
    words: &[String],
    closest: bool,
) -> Vec<Hit<'a>> {
    let mut hits: Vec<Hit> = Vec::new();
    for page in pages {
        let title = page.title.as_deref().unwrap_or("").to_lowercase();
        let summary = page.summary.as_deref().unwrap_or("").to_lowercase();
        let path = page.path.to_lowercase();
        let text = page.text.as_deref().unwrap_or("");
        let body = text.to_lowercase();
        let mut score = 0;
        let mut matched = 0;
        for w in words {
            let in_body = body.matches(w.as_str()).count();
            let found = title.contains(w.as_str())
                || summary.contains(w.as_str())
                || path.contains(w.as_str())
                || in_body > 0;
            if !found {
                continue;
            }
            matched += 1;
            score += 20 * usize::from(title.contains(w.as_str()))
                + 6 * usize::from(summary.contains(w.as_str()))
                + 4 * usize::from(path.contains(w.as_str()))
                + in_body.min(10);
        }
        if matched == 0 {
            continue;
        }
        let all = |s: &str| words.iter().all(|w| s.contains(w.as_str()));
        let rank = if all(&title) {
            "title"
        } else if all(&format!("{title} {summary}")) {
            "summary"
        } else {
            "body"
        };
        // The line with the most of the words on it, headings apart.
        let line = text
            .split('\n')
            .enumerate()
            .skip(page.body_line.saturating_sub(1))
            .filter(|(_, l)| !l.trim_start().starts_with('#'))
            .map(|(i, l)| {
                let lower = l.to_lowercase();
                let n = words.iter().filter(|w| lower.contains(w.as_str())).count();
                (n, i, l)
            })
            .filter(|(n, ..)| *n > 0)
            .max_by_key(|(n, i, _)| (*n, std::cmp::Reverse(*i)))
            .map(|(_, i, l)| (i + 1, l.trim().to_string()));
        hits.push(Hit {
            page,
            rank,
            line,
            score,
            matched,
        });
    }
    let best = hits.iter().map(|h| h.matched).max().unwrap_or(0);
    if best < words.len() {
        if !closest {
            return Vec::new();
        }
        // None has every word: those with the most, and only if that is more
        // than one of several, since one common word matches everything.
        hits.retain(|h| h.matched == best && (best > 1 || words.len() == 1));
    } else {
        hits.retain(|h| h.matched == best);
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.page.path.cmp(&b.page.path)));
    hits
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    if let Some(k) = &args.kind {
        super::kind(&tree.config, k)?;
    }
    // The index repeats every title, so it would match whatever the pages do.
    let pages = tree.pages.values().filter(|p| {
        p.path != tree.config.index
            && args
                .kind
                .as_ref()
                .is_none_or(|k| p.kind.as_ref() == Some(k))
    });
    let words = words(&args.words);
    let hits = search(pages.clone(), &words, false);
    if args.json {
        let list: Vec<_> = hits
            .iter()
            .map(|h| {
                let mut v = super::list::summary_json(h.page);
                v["rank"] = json!(h.rank);
                v["score"] = json!(h.score);
                v["line"] = json!(h.line.as_ref().map(|l| l.0));
                v["text"] = json!(h.line.as_ref().map(|l| l.1.clone()));
                v
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(u8::from(hits.is_empty()));
    }
    if hits.is_empty() {
        let near = search(pages, &words, true);
        if near.is_empty() {
            eprintln!("loam: nothing written matches");
        } else {
            eprintln!("loam: no page has every word; these have the most:");
            for h in near.iter().take(5) {
                eprintln!(
                    "  {}  {}",
                    h.page.path,
                    h.page.title.as_deref().unwrap_or("")
                );
            }
        }
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

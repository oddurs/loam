// loam new — put a page where its kind lives, seeded from the kind's template.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use super::Ctx;
use crate::config::join;
use crate::write::{Lock, write_atomic};
use anyhow::{Result, bail};

#[derive(clap::Args)]
pub struct Args {
    /// The kind of page, as loam.toml declares it
    pub kind: String,
    /// The page's title; the file name is made from it
    pub title: String,
    /// Mark the page a draft: written, not yet to be trusted
    #[arg(long)]
    pub draft: bool,
    /// Write as this agent; also read from LOAM_AGENT (see `loam agent`)
    #[arg(long, value_name = "NAME")]
    pub agent: Option<String>,
    /// Write it even though a page of the same kind looks like it
    #[arg(long)]
    pub anyway: bool,
}

/// A file name from a title, by cairn's rule: lowercase letters and digits,
/// runs of anything else as one hyphen, cut on a character boundary.
pub fn slug(title: &str, max_bytes: usize) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in title.chars() {
        if c.is_alphanumeric() {
            // Filtered after lowercasing: `İ` lowercases to `i` and a
            // combining dot, which must not reach a file name (cairn 0119).
            let lowered: String = c.to_lowercase().filter(|c| c.is_alphanumeric()).collect();
            if lowered.is_empty() {
                continue;
            }
            out.push_str(&lowered);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let mut s = out.trim_end_matches('-').to_string();
    if s.len() > max_bytes {
        let mut cut = max_bytes;
        while !s.is_char_boundary(cut) {
            cut -= 1;
        }
        s.truncate(cut);
        s = s.trim_end_matches('-').to_string();
    }
    if s.is_empty() { "page".into() } else { s }
}

/// The page `new` writes: frontmatter from the template and `--draft`, the
/// title as the level-1 heading (spec §11), then the template's body.
pub fn content(title: &str, template: Option<&str>, draft: bool) -> String {
    let template = template.unwrap_or("");
    let (mut front, body) = match crate::tree::split(template) {
        Ok((Some(f), b, _)) => (f.lines().map(str::to_string).collect::<Vec<_>>(), b),
        _ => (Vec::new(), template.to_string()),
    };
    if draft && !front.iter().any(|l| l.starts_with("status:")) {
        front.push("status: draft".into());
    }
    let mut out = String::new();
    if !front.is_empty() {
        out.push_str("---\n");
        for l in front {
            out.push_str(&l);
            out.push('\n');
        }
        out.push_str("---\n\n");
    }
    out.push_str(&format!("# {title}\n"));
    let body = body.trim_start_matches('\n');
    if !body.is_empty() {
        out.push('\n');
        out.push_str(body);
        if !body.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let config = ctx.config()?;
    let Some(kind) = config.kind(&args.kind) else {
        let names: Vec<&str> = config.kinds.iter().map(|k| k.name.as_str()).collect();
        bail!(
            "no kind called `{}`; loam.toml declares: {}",
            args.kind,
            names.join(", ")
        );
    };
    if args.title.trim().is_empty() {
        bail!("a page needs a title");
    }
    let path = join(
        &join(&config.docs, &kind.dir),
        &format!("{}.md", slug(&args.title, 80)),
    );
    let tree = crate::tree::Tree::read(config.clone())?;
    // Exactly, and also as the filesystem sees it: on a disk that ignores case,
    // `readme.md` would overwrite `README.md`.
    if tree.fs.exists(&path) || config.abs(&path).exists() {
        let there = tree
            .pages
            .iter()
            .find(|(p, _)| p.eq_ignore_ascii_case(&path))
            .and_then(|(_, page)| page.title.clone())
            .map_or_else(String::new, |t| format!(" (\"{t}\")"));
        bail!("{path} already exists{there}; loam new never overwrites a page");
    }
    // Before writing a second page on the same thing (0048).
    if !args.anyway {
        let same_kind: Vec<(String, String)> = tree
            .pages
            .iter()
            .filter(|(p, page)| {
                page.kind.as_deref() == Some(kind.name.as_str()) && **p != tree.config.index
            })
            .map(|(_, page)| {
                (
                    page.title.clone().unwrap_or_default(),
                    page.summary.clone().unwrap_or_default(),
                )
            })
            .collect();
        let common = crate::agent::common(&same_kind);
        let alike: Vec<(&String, &crate::tree::Page, Vec<String>)> = tree
            .pages
            .iter()
            .filter(|(p, page)| {
                page.kind.as_deref() == Some(kind.name.as_str()) && **p != tree.config.index
            })
            .filter_map(|(p, page)| {
                let shared = crate::agent::likeness(
                    &args.title,
                    page.title.as_deref().unwrap_or(""),
                    page.summary.as_deref().unwrap_or(""),
                    &common,
                )?;
                Some((p, page, shared))
            })
            .collect();
        if !alike.is_empty() {
            eprintln!("{} already has {} page(s) like it:", kind.name, alike.len());
            for (p, page, shared) in &alike {
                eprintln!(
                    "  {p}  \"{}\"  (shares: {})",
                    page.title.as_deref().unwrap_or(""),
                    shared.join(", ")
                );
            }
            let agent = crate::agent::acting(args.agent.as_deref());
            use std::io::IsTerminal;
            let ask = agent.is_none()
                && std::io::stdin().is_terminal()
                && std::io::stderr().is_terminal();
            let yes = ask && {
                eprint!(
                    "update one of those instead, or write a new page anyway? [y = write it / N] "
                );
                let mut answer = String::new();
                std::io::stdin().read_line(&mut answer).is_ok()
                    && matches!(answer.trim(), "y" | "Y" | "yes")
            };
            if !yes {
                eprintln!(
                    "nothing written. Update the page above, or `loam new … --anyway` if this is different."
                );
                return Ok(1);
            }
        }
    }
    let agent = crate::agent::acting(args.agent.as_deref());
    // A page an agent writes starts as a draft, when the project says so
    // (0047): a guard rail, not a boundary.
    let draft =
        args.draft || (agent.is_some() && tree.config.agent_status.as_deref() == Some("draft"));
    let text = content(&args.title, kind.template.as_deref(), draft);
    {
        let _lock = Lock::acquire(&config)?;
        write_atomic(&config.abs(&path), text.as_bytes())?;
    }
    println!("{path}");
    if draft && !args.draft {
        eprintln!(
            "a draft, because {} wrote it; a person makes it current with `loam set {path} status=current`",
            agent.as_deref().unwrap_or("an agent")
        );
    }
    ctx.after_change(&config);
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert_eq!(
            slug("Markdown rendering in a forty-column pane", 80),
            "markdown-rendering-in-a-forty-column-pane"
        );
        assert_eq!(slug("Þór's hammer!", 80), "þór-s-hammer");
        assert_eq!(slug("İ", 80), "i");
        assert_eq!(slug("???", 80), "page");
    }

    #[test]
    fn content_puts_frontmatter_first() {
        assert_eq!(
            content("T", Some("---\nkind: x\n---\n\n## Findings\n"), true),
            "---\nkind: x\nstatus: draft\n---\n\n# T\n\n## Findings\n"
        );
        assert_eq!(content("T", None, false), "# T\n");
    }
}

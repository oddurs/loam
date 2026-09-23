// loam supersede — say that one page replaces another, on both pages.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Agents in particular write `architecture-v2.md` beside `architecture.md`,
// and from then on nobody can tell which is true. This records it the way
// spec §4.2 asks — on both pages — and puts a line at the top of the old one
// for whoever lands there from a link.

use super::Ctx;
use crate::tree::{Page, Tree, resolve};
use crate::write::{
    Lines, Lock, Value, body_start, dir_of, encode_destination, relative, set_key, write_atomic,
};
use anyhow::{Result, bail};

#[derive(clap::Args)]
pub struct Args {
    /// The page being replaced
    pub old: String,
    /// The page that replaces it
    pub new: String,
}

pub const NOTICE: &str = "> **Superseded** by ";

fn lines_of(page: &Page) -> Result<Lines> {
    match std::str::from_utf8(&page.bytes) {
        Ok(t) => Ok(Lines::parse(t)),
        Err(_) => bail!("{} is not UTF-8, so loam will not rewrite it", page.path),
    }
}

/// Add `value` to a supersession key unless it already names `target`.
fn add(lines: &mut Lines, page: &Page, key: &str, current: &[String], value: String, target: &str) {
    if current
        .iter()
        .any(|v| resolve(&page.path, v).0.as_deref() == Some(target))
    {
        return;
    }
    let mut all = current.to_vec();
    all.push(value);
    let v = if all.len() == 1 {
        Value::Str(all.remove(0))
    } else {
        Value::List(all)
    };
    set_key(lines, key, &v);
}

/// The two rewritten files, (path, contents), leaving out any unchanged.
pub fn plan(tree: &Tree, old: &str, new: &str) -> Result<Vec<(String, String)>> {
    let (Some(o), Some(n)) = (tree.pages.get(old), tree.pages.get(new)) else {
        let missing = if tree.pages.contains_key(old) {
            new
        } else {
            old
        };
        bail!("{missing} is not a page");
    };
    if old == new {
        bail!("a page cannot supersede itself");
    }
    if !o.is_readable() || !n.is_readable() {
        bail!("both pages must be readable; run `loam check` to see why one is not");
    }

    let mut new_lines = lines_of(n)?;
    add(
        &mut new_lines,
        n,
        "supersedes",
        &n.supersedes,
        relative(dir_of(new), old),
        old,
    );

    let mut old_lines = lines_of(o)?;
    add(
        &mut old_lines,
        o,
        "superseded_by",
        &o.superseded_by,
        relative(dir_of(old), new),
        new,
    );
    if o.status != "superseded" || o.status_from == "frontmatter" {
        set_key(&mut old_lines, "status", &Value::Str("superseded".into()));
    }
    let start = body_start(&old_lines);
    if !old_lines.lines[start..]
        .iter()
        .any(|(l, _)| l.starts_with(NOTICE))
    {
        let title = n.title.clone().unwrap_or_else(|| new.to_string());
        let notice = format!(
            "{NOTICE}[{title}]({}).",
            encode_destination(&relative(dir_of(old), new))
        );
        // Before a blank line and the title, so the title and the summary are
        // still what §5 reads (the notice is not the paragraph under the title).
        let at = (start..old_lines.lines.len())
            .find(|&i| !old_lines.lines[i].0.trim().is_empty())
            .unwrap_or(start);
        old_lines.insert(at, &[notice, String::new()]);
    }

    let mut out = Vec::new();
    for (path, page, lines) in [(new, n, new_lines), (old, o, old_lines)] {
        let text = lines.render();
        if text.as_bytes() != page.bytes {
            out.push((path.to_string(), text));
        }
    }
    Ok(out)
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    let old = ctx.repo_path(&tree.config, &args.old)?;
    let new = ctx.repo_path(&tree.config, &args.new)?;
    let writes = plan(&tree, &old, &new)?;
    if writes.is_empty() {
        println!("{old} is already superseded by {new}");
        return Ok(0);
    }
    {
        let _lock = Lock::acquire(&tree.config)?;
        for (path, text) in &writes {
            write_atomic(&tree.config.abs(path), text.as_bytes())?;
        }
    }
    println!("{old} is superseded by {new}");
    for (path, _) in &writes {
        println!("  updated {path}");
    }
    ctx.after_change(&tree.config);
    Ok(0)
}

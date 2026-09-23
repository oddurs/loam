// loam mv — move a page, or a directory of them, and rewrite every link.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Pages are addressed by their paths (cairn 0016), so renaming one breaks
// every link to it; this is the command that makes a path identity affordable,
// as `cairn set 1 key=v2.0` is for a milestone key. It rewrites links *to*
// what moved, and links *from* what moved, whose relative paths change with
// it — in every Markdown file in the repository, not only pages, because a
// README one directory up links into the docs more than anything does.
//
// Each file is written atomically, so an interrupted move leaves every file
// either as it was or as it will be. A link that already read the same after
// the move is left exactly as written.

use super::Ctx;
use crate::config::{Config, join};
use crate::tree::{Tree, read_page, resolve};
use crate::write::{Lines, Value, dir_of, encode_destination, relative, set_key, write_atomic};
use anyhow::{Context, Result, bail};

#[derive(clap::Args)]
pub struct Args {
    /// The page, file or directory to move
    pub old: String,
    /// Where it goes; into it, if it is an existing directory
    pub new: String,
    /// Print what would change, and change nothing
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

pub struct Edit {
    pub line: usize,
    pub from: String,
    pub to: String,
}

pub struct Plan {
    /// Files to rewrite in place, before the move: (path now, text, edits).
    pub rewrites: Vec<(String, String, Vec<Edit>)>,
}

/// Every Markdown file a move might need to rewrite: what git knows about,
/// tracked or not ignored; without git, every one under the repository.
fn documents(config: &Config) -> Vec<String> {
    let git = std::process::Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.md",
        ])
        .current_dir(&config.root)
        .output();
    if let Ok(out) = git
        && out.status.success()
    {
        let mut v: Vec<String> = out
            .stdout
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .filter(|p| config.abs(p).is_file())
            .collect();
        v.sort();
        v.dedup();
        return v;
    }
    let mut out = Vec::new();
    let mut stack = vec![String::new()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(config.abs(&dir)) else {
            continue;
        };
        for e in entries.filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            let path = join(&dir, &name);
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(path);
            } else if name.ends_with(".md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn mapped(old: &str, new: &str, p: &str) -> String {
    if p == old {
        new.to_string()
    } else if let Some(rest) = p.strip_prefix(old).and_then(|r| r.strip_prefix('/')) {
        join(new, rest)
    } else {
        p.to_string()
    }
}

/// A destination written again for its new place: the same target, reached
/// from `from` — root-relative if it was, starting `./` if it did, with its
/// query and fragment as written.
fn rewrite(raw: &str, angle: bool, from: &str, target: &str) -> String {
    let cut = raw.find(['?', '#']).unwrap_or(raw.len());
    let (path, suffix) = raw.split_at(cut);
    let mut new_path = if path.starts_with('/') {
        format!("/{target}")
    } else {
        relative(dir_of(from), target)
    };
    if path.ends_with('/') && !new_path.ends_with('/') {
        new_path.push('/');
    }
    // Written the author's way: `./x.md` stays `./x.md`.
    if path.starts_with("./") && !new_path.starts_with("../") && !new_path.starts_with('/') {
        new_path.insert_str(0, "./");
    }
    let new_path = if angle {
        new_path
    } else {
        encode_destination(&new_path)
    };
    format!("{new_path}{suffix}")
}

pub fn plan(tree: &Tree, old: &str, new: &str) -> Result<Plan> {
    let config = &tree.config;
    let mut rewrites = Vec::new();
    for doc in documents(config) {
        let Ok(bytes) = std::fs::read(config.abs(&doc)) else {
            continue;
        };
        let Ok(text) = std::str::from_utf8(&bytes) else {
            continue;
        };
        let page = read_page(config, &doc, bytes.clone());
        if !page.is_readable() {
            continue;
        }
        let here = mapped(old, new, &doc);
        let mut lines = Lines::parse(text);
        let mut edits = Vec::new();

        // Right to left within a line, so earlier offsets stay true.
        let mut links = page.links.clone();
        links.sort_by(|a, b| {
            (a.raw.line, std::cmp::Reverse(a.raw.start))
                .cmp(&(b.raw.line, std::cmp::Reverse(b.raw.start)))
        });
        for link in &links {
            let (target, _, how) = resolve(&doc, &link.raw.destination);
            let Some(target) = target.filter(|_| how == "path") else {
                continue;
            };
            let there = mapped(old, new, &target);
            if resolve(&here, &link.raw.destination).0.as_deref() == Some(there.as_str()) {
                continue;
            }
            let to = rewrite(&link.raw.destination, link.raw.angle, &here, &there);
            let line = &mut lines.lines[link.raw.line - 1].0;
            line.replace_range(link.raw.start..link.raw.end, &to);
            edits.push(Edit {
                line: link.raw.line,
                from: link.raw.destination.clone(),
                to,
            });
        }

        if tree.pages.contains_key(&doc) {
            for (key, values) in [
                ("supersedes", &page.supersedes),
                ("superseded_by", &page.superseded_by),
            ] {
                let mut changed = false;
                let updated: Vec<String> = values
                    .iter()
                    .map(|v| match resolve(&doc, v) {
                        (Some(t), _, "path") => {
                            let there = mapped(old, new, &t);
                            if resolve(&here, v).0.as_deref() == Some(there.as_str()) {
                                v.clone()
                            } else {
                                changed = true;
                                let to = rewrite(v, true, &here, &there);
                                edits.push(Edit {
                                    line: 1,
                                    from: v.clone(),
                                    to: to.clone(),
                                });
                                to
                            }
                        }
                        _ => v.clone(),
                    })
                    .collect();
                if changed {
                    let value = match page
                        .frontmatter
                        .as_ref()
                        .and_then(|m| m.iter().find(|(k, _)| k == key))
                    {
                        Some((_, crate::yaml::Yaml::Str(_))) if updated.len() == 1 => {
                            Value::Str(updated[0].clone())
                        }
                        _ => Value::List(updated),
                    };
                    set_key(&mut lines, key, &value);
                }
            }
        }

        if !edits.is_empty() {
            edits.sort_by_key(|e| e.line);
            rewrites.push((doc, lines.render(), edits));
        }
    }
    Ok(Plan { rewrites })
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let (lock, tree) = ctx.locked_tree()?;
    let config = &tree.config;
    let old = ctx.repo_path(config, &args.old)?;
    if old.is_empty() || !config.abs(&old).exists() {
        bail!("{} does not exist", args.old);
    }
    let mut new = ctx.repo_path(config, &args.new)?;
    if config.abs(&new).is_dir() && new != old {
        let name = old.rsplit('/').next().unwrap_or(&old);
        new = join(&new, name);
    }
    // Moving the configuration would leave the repository unreadable; the
    // docs root is named in it, not found by where it is.
    if old == ".git" || old.starts_with(".git/") {
        bail!("{old} is git's own; loam moves pages and the files they link to");
    }
    if old == crate::config::CONFIG_FILE {
        bail!(
            "{old} is loam's own configuration; move the docs with `loam mv`, and leave it where it is"
        );
    }
    if new == old {
        bail!("{old} is already there");
    }
    if new.starts_with(&format!("{old}/")) {
        bail!("cannot move {old} into itself");
    }
    if tree.fs.exists(&new) || config.abs(&new).exists() {
        bail!("{new} already exists; loam mv never overwrites");
    }

    let plan = plan(&tree, &old, &new)?;
    let links: usize = plan.rewrites.iter().map(|r| r.2.len()).sum();

    if args.dry_run {
        println!("would move {old} → {new}");
        for (path, _, edits) in &plan.rewrites {
            for e in edits {
                println!("  {path}:{}: {} → {}", e.line, e.from, e.to);
            }
        }
        println!(
            "{links} link(s) in {} file(s); nothing changed",
            plan.rewrites.len()
        );
        return Ok(0);
    }

    {
        for (path, text, _) in &plan.rewrites {
            write_atomic(&config.abs(path), text.as_bytes())?;
        }
        let to = config.abs(&new);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::rename(config.abs(&old), &to).with_context(|| format!("moving {old} to {new}"))?;
    }

    println!("moved {old} → {new}");
    for (path, _, edits) in &plan.rewrites {
        println!(
            "  rewrote {} link(s) in {}",
            edits.len(),
            mapped(&old, &new, path)
        );
    }
    drop(lock);
    ctx.after_change(config);
    Ok(0)
}

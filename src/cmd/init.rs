// loam init — adopt a docs folder that already exists.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Every repository this is for already has a docs folder, so `init` starts
// from what is there: it proposes a kind for each directory that holds pages,
// catches the rest with a kind at the root, writes loam.toml, and touches no
// page. Then it prints what it made of every page, guesses included, because a
// guess shown is one file to correct and a guess hidden is a surprise later.

use super::Ctx;
use crate::config::{CONFIG_FILE, Config, FORMAT, join};
use crate::index::{BEGIN, END};
use crate::tree::Tree;
use crate::write::write_atomic;
use anyhow::{Result, bail};
use std::path::Path;

#[derive(clap::Args)]
pub struct Args {
    /// Use a preset's kinds instead of the folder's directories:
    /// diataxis, standard or minimal
    #[arg(long)]
    pub preset: Option<String>,
    /// The docs folder; found as docs/ or doc/ when not given
    #[arg(long)]
    pub docs: Option<String>,
    /// Replace an existing loam.toml
    #[arg(long)]
    pub force: bool,
}

pub struct KindSpec {
    pub name: String,
    pub dir: String,
    pub description: String,
    pub template: Option<&'static str>,
}

const RESEARCH_TEMPLATE: &str = "## Question\n\n## What we found\n\n## Sources\n";
const DESIGN_TEMPLATE: &str = "## What it is\n\n## Why it is this way\n\n## What it rules out\n";

fn known(name: &str) -> (&'static str, Option<&'static str>) {
    match name {
        "guide" => (
            "How do I do this? A task, start to finish, with real output.",
            None,
        ),
        "reference" => (
            "What does this mean? Facts, tables and settings, checked against the code where they can be.",
            None,
        ),
        "design" => (
            "Why is it like this? The shape of the thing, and the argument for it.",
            Some(DESIGN_TEMPLATE),
        ),
        "research" => (
            "What did we find out? Evidence, with its sources and its date.",
            Some(RESEARCH_TEMPLATE),
        ),
        // The catch-all: its description would head a section of the index,
        // where "has not found its kind" reads as a complaint about the pages.
        "page" => ("", None),
        _ => ("", None),
    }
}

fn spec(name: &str, dir: &str) -> KindSpec {
    let (description, template) = known(name);
    KindSpec {
        name: name.into(),
        dir: dir.into(),
        description: description.into(),
        template,
    }
}

/// The presets of cairn item 0017, taken from real folders rather than invented.
pub fn preset(name: &str) -> Result<Vec<KindSpec>> {
    let names: &[&str] = match name {
        // poptop's: Diátaxis without tutorials.
        "diataxis" => &["guide", "reference", "design"],
        "standard" => &["guide", "reference", "design", "research"],
        // For a folder that has not decided.
        "minimal" => &[],
        other => bail!("no preset called `{other}`; there are diataxis, standard and minimal"),
    };
    let mut kinds: Vec<KindSpec> = names.iter().map(|n| spec(n, n)).collect();
    kinds.push(spec("page", "."));
    Ok(kinds)
}

fn singular(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.strip_suffix('s') {
        Some(stem) if stem.len() >= 3 && !lower.ends_with("ss") => stem.to_string(),
        _ => lower,
    }
}

fn holds_pages(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .any(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with(['.', '_']) {
                return false;
            }
            let path = e.path();
            if path.is_dir() {
                holds_pages(&path)
            } else {
                name.ends_with(".md")
            }
        })
}

/// A kind for each directory directly under the docs root that holds pages,
/// and a kind at the root for everything else.
pub fn infer(docs: &Path) -> Vec<KindSpec> {
    let mut dirs: Vec<String> = std::fs::read_dir(docs)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with(['.', '_']) && holds_pages(&docs.join(n)))
        .collect();
    // The kinds a reader will recognise first, in the presets' order — how
    // before what before why — and any others after them, alphabetically.
    let rank = |d: &String| {
        ["guide", "reference", "design", "research"]
            .iter()
            .position(|k| *k == singular(d))
            .unwrap_or(4)
    };
    dirs.sort_by(|a, b| (rank(a), a).cmp(&(rank(b), b)));
    let mut kinds: Vec<KindSpec> = Vec::new();
    for dir in dirs {
        let mut name = singular(&dir);
        if name == "page" || kinds.iter().any(|k| k.name == name) {
            name = dir.to_lowercase();
        }
        kinds.push(spec(&name, &dir));
    }
    kinds.push(spec("page", "."));
    kinds
}

fn toml_string(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

pub fn config_text(
    docs: &str,
    cairn: Option<&str>,
    kinds: &[KindSpec],
    hook: bool,
    index_note: &str,
) -> String {
    let mut t = String::new();
    t.push_str("# loam.toml — how loam reads this repository's docs.\n#\n");
    t.push_str("#   loam check     what is broken: links, anchors, frontmatter\n");
    t.push_str("#   loam render    regenerate the index between its markers\n");
    t.push_str("#   loam list      every page, its kind and its status\n#\n");
    t.push_str("# The page format is https://github.com/oddurs/loam/blob/main/spec/README.md\n\n");
    t.push_str(&format!(
        "format = {FORMAT}\n\n[docs]\nroot = {}\n",
        toml_string(docs)
    ));
    if let Some(c) = cairn {
        t.push_str("\n# A link into this directory is a reference to a cairn item, checked by its number.\n");
        t.push_str(&format!("[links]\ncairn = {}\n", toml_string(c)));
    }
    t.push_str(
        "\n[index]\n# Relative to the docs root. loam writes only between the two marker lines.\n",
    );
    t.push_str("path = \"README.md\"\n");
    if !index_note.is_empty() {
        t.push_str(&format!("# {index_note}\n"));
    }
    t.push_str("\n[hooks]\n");
    if hook {
        t.push_str("after-change = \"loam render --quiet\"\n");
    } else {
        t.push_str("# after-change = \"loam render --quiet\"   # once the index has its markers\n");
    }
    t.push_str("\n# Kinds, in the order the index presents them. A page's kind is the one whose\n");
    t.push_str("# directory holds it; a kind at \".\" takes whatever no other kind claims.\n");
    let only = kinds.len() == 1;
    for k in kinds {
        if k.dir == "." {
            t.push_str("\n# Whatever no other kind claims.");
        }
        t.push_str(&format!(
            "\n[[kind]]\nname = {}\ndir = {}\n",
            toml_string(&k.name),
            toml_string(&k.dir)
        ));
        if k.dir == "." {
            t.push_str(if only {
                "title = \"Pages\"\n"
            } else {
                "title = \"Other pages\"\n"
            });
        }
        if !k.description.is_empty() {
            t.push_str(&format!("description = {}\n", toml_string(&k.description)));
        }
        if let Some(tpl) = k.template {
            t.push_str(&format!("template = \"\"\"\n{tpl}\"\"\"\n"));
        }
    }
    t
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let root = ctx.cwd.canonicalize()?;
    let config_path = root.join(CONFIG_FILE);
    if config_path.exists() && !args.force {
        bail!("{CONFIG_FILE} already exists here; `loam init --force` replaces it");
    }
    let docs = match args.docs {
        Some(d) => d.trim_matches('/').to_string(),
        None => ["docs", "doc"]
            .into_iter()
            .find(|d| root.join(d).is_dir())
            .map(str::to_string)
            .ok_or_else(|| anyhow::anyhow!("no docs/ or doc/ here; name the folder with --docs"))?,
    };
    if !root.join(&docs).is_dir() {
        bail!("{docs}/ is not a directory");
    }
    let kinds = match &args.preset {
        Some(p) => preset(p)?,
        None => infer(&root.join(&docs)),
    };

    // cairn's own default when its config does not say.
    let cairn = std::fs::read_to_string(root.join("cairn.toml"))
        .ok()
        .map(|text| {
            text.parse::<toml::Table>()
                .ok()
                .and_then(|t| t.get("project")?.get("dir")?.as_str().map(str::to_string))
                .unwrap_or_else(|| "cairn/items".into())
        });

    let index = root.join(&docs).join("README.md");
    let (hook, note) = match std::fs::read_to_string(&index) {
        Err(_) => (true, String::new()),
        Ok(text) if text.contains(BEGIN) && text.contains(END) => (true, String::new()),
        Ok(_) => (
            false,
            format!(
                "{docs}/README.md has no markers yet: add `{BEGIN}` and `{END}` where the index goes."
            ),
        ),
    };

    let text = config_text(&docs, cairn.as_deref(), &kinds, hook, &note);
    let config = Config::parse(&root, &text)?;
    write_atomic(&config_path, text.as_bytes())?;

    let tree = Tree::read(config)?;
    let width = tree
        .pages
        .keys()
        .map(|p| p.chars().count())
        .max()
        .unwrap_or(0);
    let kw = tree
        .config
        .kinds
        .iter()
        .map(|k| k.name.len())
        .max()
        .unwrap_or(1)
        .max(1);
    for (path, page) in &tree.pages {
        let kind = page.kind.as_deref().unwrap_or("?");
        let title = page
            .title
            .as_deref()
            .map_or_else(|| "(no title)".to_string(), |t| format!("\"{t}\""));
        let mut notes = Vec::new();
        if page.kind_from == Some("directory")
            && tree.config.kind(kind).is_some_and(|k| k.dir.is_empty())
            && !dir_of_docs(&tree.config, path).is_empty()
        {
            notes.push("no kind for its directory → the root kind".to_string());
        }
        if page.kind.is_none() {
            notes.push("no kind claims its directory".to_string());
        }
        if page.title.is_none() {
            notes.push("no level-1 heading to take a title from".to_string());
        }
        let n = page
            .findings
            .iter()
            .filter(|f| f.code != "untitled" && f.code != "unclaimed")
            .count();
        if n > 0 {
            notes.push(format!("{n} finding(s); see `loam check`"));
        }
        let notes = if notes.is_empty() {
            String::new()
        } else {
            crate::style::dim(&format!("  ({})", notes.join("; ")))
        };
        println!("{path:width$}  {kind:kw$}  {title}{notes}");
    }
    println!();
    let kinds_list: Vec<String> = tree
        .config
        .kinds
        .iter()
        .map(|k| {
            format!(
                "{} ({})",
                k.name,
                if k.dir.is_empty() {
                    format!("{docs}/")
                } else {
                    join(&docs, &k.dir) + "/"
                }
            )
        })
        .collect();
    println!(
        "wrote {CONFIG_FILE}: {} page(s), kinds {}",
        tree.pages.len(),
        kinds_list.join(", ")
    );
    println!("no page was changed.");
    println!();
    println!("next: correct any guess above in {CONFIG_FILE}, then `loam check`.");
    if !note.is_empty() {
        println!("      {note}");
    } else {
        println!("      `loam render` writes the index to {docs}/README.md.");
    }
    Ok(0)
}

fn dir_of_docs<'a>(config: &Config, path: &'a str) -> &'a str {
    let inside = config.inside_docs(path).unwrap_or(path);
    inside.rsplit_once('/').map_or("", |(d, _)| d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_kinds_come_first() {
        let dir = tempfile::tempdir().unwrap();
        for d in ["design", "roadmaps", "guide", "reference", "archive"] {
            std::fs::create_dir_all(dir.path().join(d)).unwrap();
            std::fs::write(dir.path().join(d).join("a.md"), "# A\n").unwrap();
        }
        let names: Vec<String> = infer(dir.path()).into_iter().map(|k| k.name).collect();
        assert_eq!(
            names,
            ["guide", "reference", "design", "archive", "roadmap", "page"]
        );
    }

    #[test]
    fn singulars() {
        assert_eq!(singular("roadmaps"), "roadmap");
        assert_eq!(singular("decisions"), "decision");
        assert_eq!(singular("guide"), "guide");
        assert_eq!(singular("process"), "process");
        assert_eq!(singular("ops"), "ops");
    }

    #[test]
    fn every_preset_writes_a_config_loam_reads() {
        for p in ["diataxis", "standard", "minimal"] {
            let text = config_text("docs", Some("cairn/items"), &preset(p).unwrap(), true, "");
            let c = Config::parse(Path::new("/"), &text).unwrap();
            assert!(c.warnings.is_empty(), "{:?}", c.warnings);
            assert_eq!(c.kinds.last().unwrap().dir, "");
        }
    }
}

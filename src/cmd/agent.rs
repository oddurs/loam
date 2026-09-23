// loam agent — the instructions an agent needs, generated from loam.toml.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Told to "research this in docs", an agent invents a filename and a format,
// and the next agent invents another. cairn fixed this for items with a block
// generated from its schema, which cannot describe a workflow the project does
// not have. This is that, for pages: the kinds and where they live come from
// the configuration, and nothing here names a kind the project did not declare.

use super::Ctx;
use crate::config::{Config, join};
use crate::write::write_atomic;
use anyhow::{Context, Result};

#[derive(clap::Args)]
pub struct Args {
    /// Insert or update the block in this file, as AGENTS.md or CLAUDE.md,
    /// instead of printing it
    #[arg(short, long, value_name = "FILE")]
    pub write: Option<std::path::PathBuf>,
}

pub const BEGIN: &str = "<!-- loam:begin -->";
pub const END: &str = "<!-- loam:end -->";

pub fn block(config: &Config) -> String {
    let docs = if config.docs.is_empty() {
        ".".to_string()
    } else {
        format!("{}/", config.docs)
    };
    let mut b = String::new();
    b.push_str(BEGIN);
    b.push_str("\n## Written context\n\n");
    b.push_str(&format!(
        "This project keeps its written context — research, design, reference, guides — as Markdown pages under `{docs}`, read and checked by `loam`. The index at `{}` is generated: never edit between its markers.\n\n",
        config.index
    ));
    b.push_str("**Search before you write.** Another session has probably written about this already. Update that page rather than start another; a second page on the same thing is how a docs folder decays.\n\n");

    b.push_str("### Where pages go\n\n");
    for k in &config.kinds {
        let dir = if k.dir.is_empty() {
            docs.clone()
        } else {
            format!("{}/", join(&config.docs, &k.dir))
        };
        let what = k
            .description
            .as_deref()
            .map(|d| d.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|d| !d.is_empty())
            .map_or(String::new(), |d| format!(" — {d}"));
        let age = k.stale_after.as_deref().map_or(String::new(), |a| {
            format!(" Goes stale {a} after its last review.")
        });
        b.push_str(&format!("- **`{}`** in `{dir}`{what}{age}\n", k.name));
    }
    b.push('\n');

    b.push_str("### The loop\n\n");
    b.push_str("1. `loam search <WORDS>` for what is already written, and `loam context <PATH>` for the pages about a file you are about to change. Read them.\n");
    b.push_str("2. To add a page: `loam new <KIND> \"<Title>\"` — never a path you made up. It refuses when a page like it exists: update that page instead, or pass `--anyway` if yours is really about something else.\n");
    if config.agent_status.as_deref() == Some("draft") {
        b.push_str("3. A page you create starts as `status: draft`. A person makes it `current`; do not do that yourself. loam recognises Claude Code; any other agent identifies itself with `LOAM_AGENT=<name>` in the environment, or `--agent <name>`.\n");
    } else {
        b.push_str("3. Identify yourself with `LOAM_AGENT=<name>` in the environment, or `--agent <name>`. loam recognises Claude Code without it.\n");
    }
    b.push_str("4. Write the page: its title as the `# heading`, then a first paragraph saying what the page is for — that paragraph is its summary in the index. Say which files it describes, so loam can tell when they change: `loam set <PAGE> covers+=<PATH>`. A page drawn from outside sources lists them in its frontmatter, each with the day it was read:\n\n   ```yaml\n   sources:\n     - title: <what it is>\n       url: <where it is>\n       read: <YYYY-MM-DD>\n   ```\n\n");
    b.push_str("5. A page that replaces another: `loam supersede <OLD> <NEW>` — never a `-v2.md` beside the old one. Renaming: `loam mv <OLD> <NEW>`, which rewrites every link.\n");
    b.push_str("6. After changing code, `loam stale --working-tree` names the pages covering what you changed. Update each, or if it is still true, `loam review <PAGE>`.\n");
    b.push_str("7. `loam check` must pass before you finish.\n\n");

    b.push_str("### Pages and backlog items\n\n");
    if config.cairn.is_some() {
        b.push_str("A cairn item records why something changed; a page says how it is now. A question is an item; its answer, if it outlives the question, is a page, and the item links to it. A plan with steps is a milestone and its items, not a page. A page cites the items behind it by a relative link to the item's file.\n\n");
    } else {
        b.push_str("A page says how things are now. A plan with steps, a task, a question still open, belong wherever the project tracks work — not in these pages.\n\n");
    }

    b.push_str("### Commands\n\n```sh\n");
    for (cmd, what) in [
        (
            "loam search <WORDS> --json",
            "what is written, titles first",
        ),
        (
            "loam context <PATH> --budget 8k",
            "the pages about a file, within a budget",
        ),
        ("loam list --kind <KIND> --json", "every page of a kind"),
        ("loam show <PAGE>", "one page, and whether it is still true"),
        ("loam new <KIND> \"<TITLE>\"", "a page where its kind lives"),
        (
            "loam set <PAGE> <KEY>=<VALUE>",
            "frontmatter; covers+=<PATH> adds to a list",
        ),
        ("loam supersede <OLD> <NEW>", "one page replaces another"),
        ("loam mv <OLD> <NEW>", "rename, rewriting every link"),
        (
            "loam stale --working-tree",
            "pages your uncommitted changes affect",
        ),
        (
            "loam review <PAGE>",
            "record that a page was read against the code",
        ),
        ("loam check", "validate; run before finishing"),
    ] {
        b.push_str(&format!("{cmd:<34}# {what}\n"));
    }
    b.push_str("```\n");
    b.push_str(END);
    b.push('\n');
    b
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let config = ctx.config()?;
    let block = block(&config);
    let Some(path) = args.write else {
        print!("{block}");
        return Ok(0);
    };
    let full = if path.is_absolute() {
        path.clone()
    } else {
        config.root.join(&path)
    };
    // A file that cannot be read as text is not written over — it would be
    // lost. A file that does not exist yet is simply empty.
    let existing = match std::fs::read(&full) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(t) => t,
            Err(_) => anyhow::bail!(
                "{} is not UTF-8, so loam will not write into it",
                path.display()
            ),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    let block_lines: Vec<String> = block
        .trim_end_matches('\n')
        .split('\n')
        .map(str::to_string)
        .collect();
    let mut lines = crate::write::Lines::parse(&existing);
    let updated = match crate::write::markers(&lines, BEGIN, END) {
        // Replace the block where it is, so the file can be edited around it.
        Some((b, e)) => {
            crate::write::replace_lines(&mut lines, b, e, &block_lines);
            lines.render()
        }
        None if existing.is_empty() => block.clone(),
        // After the rest, a blank line between, in the file's own endings.
        None => {
            let eol = lines.eol();
            let mut text = existing.clone();
            if !text.ends_with('\n') {
                text.push_str(eol);
            }
            if !text.ends_with(&format!("{eol}{eol}")) {
                text.push_str(eol);
            }
            text.push_str(&block_lines.join(eol));
            text.push_str(eol);
            text
        }
    };
    if updated == existing {
        println!("{} is already current", path.display());
        return Ok(0);
    }
    write_atomic(&full, updated.as_bytes())?;
    println!("wrote the loam block to {}", path.display());
    Ok(0)
}

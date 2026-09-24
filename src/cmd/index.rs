// loam index — the index, printed rather than written; or, with --json, the
// manifest a site can be built from.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use super::Ctx;
use crate::manifest::{History, manifest};
use anyhow::Result;
use std::collections::HashMap;

#[derive(clap::Args)]
pub struct Args {
    /// Print the manifest: every page, the index's sections, backlinks,
    /// headings and freshness, as JSON (spec/manifest.md)
    #[arg(long)]
    pub json: bool,
    /// Leave out freshness, which reads the git history
    #[arg(long)]
    pub no_history: bool,
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    if !args.json {
        for line in crate::index::generate(&tree) {
            println!("{line}");
        }
        return Ok(0);
    }
    let history = if args.no_history {
        History {
            pages: HashMap::new(),
            notes: Vec::new(),
            error: None,
        }
    } else {
        history(&tree)
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest(&tree, &history))?
    );
    Ok(0)
}

/// Every page's freshness. Outside git there is no history, which is not an
/// error; failing to read one that exists is.
fn history(tree: &crate::tree::Tree) -> History {
    let Ok(git) = crate::git::Git::open(&tree.config.root) else {
        return History {
            pages: HashMap::new(),
            notes: vec!["not a git repository, so there is no history to judge pages by".into()],
            error: None,
        };
    };
    let opts = crate::fresh::Options {
        at: None,
        today: None,
        only: None,
    };
    match crate::fresh::assess(tree, &git, &opts) {
        Ok(r) => History {
            pages: r.pages.into_iter().map(|f| (f.path.clone(), f)).collect(),
            notes: r.notes,
            error: None,
        },
        Err(e) => History {
            pages: HashMap::new(),
            notes: Vec::new(),
            error: Some(format!("{e:#}")),
        },
    }
}

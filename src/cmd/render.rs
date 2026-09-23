// loam render — write the docs index, or say whether it is current.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use super::Ctx;
use crate::index::{self, Target};
use crate::write::write_atomic;
use anyhow::Result;

#[derive(clap::Args)]
pub struct Args {
    /// Change nothing; exit 1 if the index is not what rendering would write
    #[arg(long)]
    pub check: bool,

    /// Print nothing on success
    #[arg(short, long)]
    pub quiet: bool,
}

/// Whether the index is current, for `render --check` and `check --render`.
pub fn is_current(tree: &crate::tree::Tree) -> Result<bool> {
    Ok(match index::render(tree)? {
        Target::New(_) => false,
        Target::Existing { current, rendered } => current == rendered,
    })
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    // `--check` only reads, and runs in CI: it takes no lock.
    let (_lock, tree) = if args.check {
        (None, ctx.tree()?)
    } else {
        let (l, t) = ctx.locked_tree()?;
        (Some(l), t)
    };
    let path = tree.config.index.clone();
    let target = index::render(&tree)?;
    if args.check {
        let current =
            matches!(&target, Target::Existing { current, rendered } if current == rendered);
        if current {
            if !args.quiet {
                println!("{} {path} is current", crate::style::green("ok:"));
            }
            return Ok(0);
        }
        eprintln!("loam: {path} is out of date; run `loam render`");
        return Ok(1);
    }
    let text = match target {
        Target::New(text) => text,
        Target::Existing { current, rendered } if current == rendered => {
            if !args.quiet {
                println!("{path} is already current");
            }
            return Ok(0);
        }
        Target::Existing { rendered, .. } => rendered,
    };
    write_atomic(&tree.config.abs(&path), text.as_bytes())?;
    if !args.quiet {
        println!("rendered {path}");
    }
    Ok(0)
}

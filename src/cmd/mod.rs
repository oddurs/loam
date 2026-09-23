// loam — the commands, and what they share.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

pub mod agent;
pub mod check;
pub mod context;
pub mod init;
pub mod list;
pub mod mv;
pub mod new;
pub mod render;
pub mod review;
pub mod search;
pub mod set;
pub mod show;
pub mod stale;
pub mod supersede;

use crate::config::Config;
use crate::tree::Tree;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Where a command runs, and whether it may run hooks.
pub struct Ctx {
    pub cwd: PathBuf,
    pub no_hooks: bool,
}

impl Ctx {
    pub fn config(&self) -> Result<Config> {
        let config = Config::discover(&self.cwd)?;
        for w in &config.warnings {
            eprintln!("{}: {w}", crate::style::yellow("loam"));
        }
        Ok(config)
    }

    pub fn tree(&self) -> Result<Tree> {
        Tree::read(self.config()?)
    }

    /// The tree, read while holding the lock, for a command that will write:
    /// reading first and locking after lets two writers each read the page as
    /// it was, and the second write undo the first. Release it before running
    /// a hook, which may be loam wanting the lock itself.
    pub fn locked_tree(&self) -> Result<(crate::write::Lock, Tree)> {
        let config = self.config()?;
        let lock = crate::write::Lock::acquire(&config)?;
        Ok((lock, Tree::read(config)?))
    }

    /// A path the user typed, as a repository path. Tried as given from the
    /// working directory, then from the repository, then from the docs root.
    pub fn repo_path(&self, config: &Config, typed: &str) -> Result<String> {
        let root = config
            .root
            .canonicalize()
            .context("finding the repository")?;
        let candidates = [
            self.cwd.join(typed),
            config.root.join(typed),
            config.abs(&config.docs).join(typed),
        ];
        let from = |p: &Path| -> Option<String> {
            let parent = p.parent()?.canonicalize().ok()?;
            let full = parent.join(p.file_name()?);
            let rel = full.strip_prefix(&root).ok()?;
            Some(rel.to_string_lossy().replace('\\', "/"))
        };
        for c in &candidates {
            if c.exists()
                && let Some(r) = from(c)
            {
                return Ok(r);
            }
        }
        // Nothing there yet — a destination. Resolve it from the working
        // directory without asking the filesystem about what does not exist.
        let base = self
            .cwd
            .canonicalize()
            .context("finding the working directory")?;
        let rel = base
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .ok();
        let joined = match (typed.starts_with('/'), rel) {
            (true, _) => return Err(anyhow::anyhow!("{typed} is not inside the repository")),
            (false, Some(rel)) => crate::config::join(&rel, typed),
            (false, None) => anyhow::bail!("the working directory is not inside the repository"),
        };
        crate::tree::normalise(&joined)
            .with_context(|| format!("{typed} is not inside the repository"))
    }

    /// Run the configured after-change hook, as cairn runs its own.
    pub fn after_change(&self, config: &Config) {
        if self.no_hooks {
            return;
        }
        let Some(cmd) = &config.after_change else {
            return;
        };
        // `loam …` means this loam, not whichever one is first on $PATH: a
        // hook written by `loam init` must not depend on how loam was installed.
        let run = match (cmd.strip_prefix("loam "), std::env::current_exe()) {
            (Some(rest), Ok(exe)) => format!(
                "'{}' {rest}",
                exe.display().to_string().replace('\'', "'\\''")
            ),
            _ => cmd.clone(),
        };
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&run)
            .current_dir(&config.root)
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => eprintln!(
                "{}: the after-change hook `{cmd}` exited {s}",
                crate::style::yellow("loam")
            ),
            Err(e) => eprintln!(
                "{}: could not run the after-change hook `{cmd}`: {e}",
                crate::style::yellow("loam")
            ),
        }
    }
}

/// The page at `path`, or an error naming the page probably meant: one with
/// the same file name, or a path a letter or two away.
pub fn page<'a>(tree: &'a Tree, path: &str) -> Result<&'a crate::tree::Page> {
    if let Some(page) = tree.pages.get(path) {
        return Ok(page);
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    let mut near: Vec<&str> = tree
        .pages
        .keys()
        .map(String::as_str)
        .filter(|p| p.rsplit('/').next() == Some(name))
        .collect();
    if near.is_empty() {
        near = tree
            .pages
            .keys()
            .map(String::as_str)
            .filter(|p| distance(p, path) <= 2)
            .collect();
    }
    near.sort();
    match near.as_slice() {
        [] => anyhow::bail!(
            "{path} is not a page (a page is a .md file under {}/)",
            tree.config.docs
        ),
        [one] => anyhow::bail!("{path} is not a page; did you mean {one}?"),
        many => anyhow::bail!(
            "{path} is not a page; did you mean one of: {}?",
            many.iter().take(5).copied().collect::<Vec<_>>().join(", ")
        ),
    }
}

/// The kind called `name`, or an error listing the kinds there are.
pub fn kind<'a>(config: &'a Config, name: &str) -> Result<&'a crate::config::Kind> {
    config.kind(name).ok_or_else(|| {
        let names: Vec<&str> = config.kinds.iter().map(|k| k.name.as_str()).collect();
        anyhow::anyhow!(
            "no kind called `{name}`; loam.toml declares: {}",
            names.join(", ")
        )
    })
}

/// Edits, in characters, to turn one string into the other.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let here = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != *cb))
                .min(row[j] + 1)
                .min(here + 1);
            prev = here;
        }
    }
    row[b.len()]
}

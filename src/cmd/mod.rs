// loam — the commands, and what they share.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

pub mod check;
pub mod init;
pub mod list;
pub mod mv;
pub mod new;
pub mod render;
pub mod search;
pub mod show;
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

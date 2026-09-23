// loam — what the repository's history says, asked of git.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// git is an ordinary program loam runs, as cairn runs it for `cairn log`: no
// library is linked, so loam reads history exactly as the person's own git
// does, with their configuration, and needs nothing git does not already have.

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Git {
    root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FileChange {
    pub path: String,
    pub added: u64,
    pub removed: u64,
}

#[derive(Clone, Debug)]
pub struct Commit {
    pub sha: String,
    /// The committer's date, `YYYY-MM-DD`.
    pub date: String,
    pub subject: String,
    pub files: Vec<FileChange>,
}

impl Git {
    /// The repository at `root`, if it is one and git is installed.
    pub fn open(root: &Path) -> Result<Git> {
        let out = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(root)
            .output();
        match out {
            Err(_) => bail!(
                "git is not installed, and whether a page is still true is read from git history"
            ),
            Ok(o) if !o.status.success() => bail!(
                "{} is not in a git repository, so there is no history to read",
                root.display()
            ),
            Ok(_) => Ok(Git {
                root: root.to_path_buf(),
            }),
        }
    }

    fn command(&self) -> Command {
        let mut c = Command::new("git");
        // Paths exactly as they are, not quoted and escaped for a terminal.
        c.args(["-c", "core.quotePath=false"])
            .current_dir(&self.root);
        c
    }

    fn run(&self, args: &[&str]) -> Result<String> {
        let out = self.command().args(args).output().context("running git")?;
        if !out.status.success() {
            bail!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    fn succeeds(&self, args: &[&str]) -> bool {
        self.command()
            .args(args)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// The full name of a commit, if it exists here.
    pub fn resolve(&self, rev: &str) -> Option<String> {
        let spec = format!("{rev}^{{commit}}");
        self.run(&["rev-parse", "--verify", "--quiet", &spec])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn is_ancestor(&self, ancestor: &str, of: &str) -> bool {
        self.succeeds(&["merge-base", "--is-ancestor", ancestor, of])
    }

    /// For every path under `dir`, the last commit at or before `at` that
    /// changed it, with that commit's date.
    pub fn last_changes(&self, at: &str, dir: &str) -> Result<HashMap<String, (String, String)>> {
        let pathspec = if dir.is_empty() {
            ".".to_string()
        } else {
            dir.to_string()
        };
        let text = self.run(&[
            "log",
            "--format=@@%H %cs",
            "--name-only",
            "--no-renames",
            at,
            "--",
            &pathspec,
        ])?;
        let mut out = HashMap::new();
        let mut current: Option<(String, String)> = None;
        for line in text.lines() {
            if let Some(head) = line.strip_prefix("@@") {
                let (sha, date) = head.split_once(' ').unwrap_or((head, ""));
                current = Some((sha.to_string(), date.to_string()));
            } else if !line.is_empty()
                && let Some(c) = &current
            {
                out.entry(line.to_string()).or_insert_with(|| c.clone());
            }
        }
        Ok(out)
    }

    /// Commits in `from..to` (all of `to`'s history when `from` is `None`),
    /// newest first, merges left out, with the lines each changed per file.
    /// Changes to whitespace alone are not counted: reindenting code does not
    /// make a page describing it untrue.
    ///
    /// `paths` narrows the log to what could matter, as git pathspecs; every
    /// file is still checked against the page's own patterns afterwards.
    pub fn commits(&self, from: Option<&str>, to: &str, paths: &[String]) -> Result<Vec<Commit>> {
        let range = match from {
            Some(f) => format!("{f}..{to}"),
            None => to.to_string(),
        };
        let mut args: Vec<&str> = vec![
            "log",
            "--no-merges",
            "--no-renames",
            "--ignore-all-space",
            "--ignore-blank-lines",
            "--numstat",
            "--format=@@%H%x09%cs%x09%s",
            &range,
            "--",
        ];
        args.extend(paths.iter().map(String::as_str));
        let text = self.run(&args)?;
        let mut out: Vec<Commit> = Vec::new();
        for line in text.lines() {
            if let Some(head) = line.strip_prefix("@@") {
                let mut parts = head.splitn(3, '\t');
                out.push(Commit {
                    sha: parts.next().unwrap_or("").to_string(),
                    date: parts.next().unwrap_or("").to_string(),
                    subject: parts.next().unwrap_or("").to_string(),
                    files: Vec::new(),
                });
            } else if let Some(c) = out.last_mut() {
                let mut parts = line.splitn(3, '\t');
                if let (Some(a), Some(r), Some(path)) = (parts.next(), parts.next(), parts.next()) {
                    // `-` for a binary file: it changed, by an unknown amount.
                    let (added, removed) = (a.parse().unwrap_or(1), r.parse().unwrap_or(0));
                    if added + removed > 0 {
                        c.files.push(FileChange {
                            path: path.to_string(),
                            added,
                            removed,
                        });
                    }
                }
            }
        }
        Ok(out)
    }

    /// The oldest commit reachable from `at` whose change to `path` added or
    /// removed `needle` — for a review, the commit that brought it here.
    pub fn introduced(&self, needle: &str, path: &str, at: &str) -> Option<String> {
        let pickaxe = format!("-S{needle}");
        let text = self
            .run(&["log", "--format=%H", &pickaxe, at, "--", path])
            .ok()?;
        text.lines().last().map(str::to_string)
    }

    /// The last commit on `at`'s first-parent line made on or before `date`.
    pub fn commit_on_or_before(&self, date: &str, at: &str) -> Option<String> {
        let before = format!("--before={date} 23:59:59");
        let text = self
            .run(&["rev-list", "-1", "--first-parent", &before, at])
            .ok()?;
        Some(text.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// Paths with changes not yet committed, staged or not, and new files.
    pub fn uncommitted(&self) -> Result<Vec<String>> {
        let text = self.run(&[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--no-renames",
        ])?;
        Ok(text
            .split('\0')
            .filter(|e| e.len() > 3)
            .map(|e| e[3..].to_string())
            .collect())
    }

    /// Every file git knows about, or would: tracked, and untracked but not ignored.
    pub fn files(&self) -> Result<Vec<String>> {
        let text = self.run(&[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])?;
        let mut v: Vec<String> = text
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        v.sort();
        v.dedup();
        Ok(v)
    }

    /// The committer's date of `rev`, `YYYY-MM-DD`.
    pub fn date_of(&self, rev: &str) -> Option<String> {
        let text = self.run(&["log", "-1", "--format=%cs", rev, "--"]).ok()?;
        Some(text.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// Commits at or before `at` that changed `path`, newest first, with dates.
    pub fn path_history(&self, path: &str, at: &str) -> Vec<(String, String)> {
        self.run(&["log", "--format=%H %cs", "--no-renames", at, "--", path])
            .map(|t| {
                t.lines()
                    .filter_map(|l| {
                        l.split_once(' ')
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// A file's contents at a commit, if it existed there.
    pub fn show(&self, rev: &str, path: &str) -> Option<Vec<u8>> {
        let spec = format!("{rev}:{path}");
        let out = self.command().args(["show", &spec]).output().ok()?;
        out.status.success().then_some(out.stdout)
    }

    /// Whether history was cut short by a shallow clone.
    pub fn is_shallow(&self) -> bool {
        self.run(&["rev-parse", "--is-shallow-repository"])
            .is_ok_and(|s| s.trim() == "true")
    }

    /// Files changed between `rev` and HEAD.
    pub fn changed_since(&self, rev: &str) -> Result<Vec<String>> {
        let range = format!("{rev}..HEAD");
        let text = self.run(&["diff", "--name-only", "--no-renames", &range])?;
        Ok(text
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    pub fn head(&self) -> Option<String> {
        self.resolve("HEAD")
    }
}

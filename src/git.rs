// loam — what the repository's history says, asked of git.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// git is an ordinary program loam runs, as cairn runs it for `cairn log`: no
// library is linked, so loam reads history exactly as the person's own git
// does, with their configuration, and needs nothing git does not already have.

use anyhow::{Context, Result, bail};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Git {
    root: PathBuf,
    /// Where `root` is below the top of the work tree, `sub/dir/` or empty.
    /// git reports some paths from the top whatever directory it runs in.
    prefix: String,
    /// One `git cat-file --batch`, started when first needed, for every blob
    /// read in a run rather than a process for each.
    blobs: RefCell<Option<Batch>>,
}

struct Batch {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Drop for Batch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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

/// One file in one commit of a directory's history, with renames found.
#[derive(Clone, Debug)]
pub struct Entry {
    /// `A`dded, `M`odified, `D`eleted, `R`enamed, and so on.
    pub status: char,
    pub old_blob: String,
    pub new_blob: String,
    /// The path before, which differs from `path` only for a rename or copy.
    pub old_path: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct Change {
    pub sha: String,
    pub date: String,
    pub entries: Vec<Entry>,
}

/// The pieces of `git log -z` output: NUL-separated, with the line break git
/// puts between a commit's header and its changes left off.
fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split('\0')
        .map(|t| t.strip_prefix('\n').unwrap_or(t))
        .filter(|t| !t.is_empty())
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
            Ok(_) => {
                let mut git = Git {
                    root: root.to_path_buf(),
                    prefix: String::new(),
                    blobs: RefCell::new(None),
                };
                git.prefix = git.run(&["rev-parse", "--show-prefix"])?.trim().to_string();
                Ok(git)
            }
        }
    }

    /// A path git gave from the top of the work tree, made relative to the
    /// root, or `None` when it is outside it.
    fn relative(&self, path: &str) -> Option<String> {
        path.strip_prefix(self.prefix.as_str()).map(str::to_string)
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

    /// Commits reachable from the first of `revs` and not from any `^rev`
    /// after it, newest first, with the lines each changed per file. A merge
    /// counts for what it changed beyond merging, which is nothing unless a
    /// conflict was resolved or something was added in the merge; what the
    /// merged branch changed is counted in the branch's own commits. Changes to
    /// whitespace alone are not counted: reindenting code does not make a page
    /// describing it untrue.
    ///
    /// `paths` narrows the log to what could matter, as git pathspecs.
    pub fn commits(&self, revs: &[String], paths: &[String]) -> Result<Vec<Commit>> {
        let mut args: Vec<&str> = vec![
            "log",
            "-z",
            "--relative",
            "--diff-merges=remerge",
            "--no-renames",
            "--ignore-all-space",
            "--ignore-blank-lines",
            "--numstat",
            "--format=@@%H%x09%cs%x09%s",
        ];
        args.extend(revs.iter().map(String::as_str));
        args.push("--");
        args.extend(paths.iter().map(String::as_str));
        let text = self.run(&args)?;
        let mut out: Vec<Commit> = Vec::new();
        for token in tokens(&text) {
            if let Some(head) = token.strip_prefix("@@") {
                let mut parts = head.splitn(3, '\t');
                out.push(Commit {
                    sha: parts.next().unwrap_or("").to_string(),
                    date: parts.next().unwrap_or("").to_string(),
                    subject: parts.next().unwrap_or("").to_string(),
                    files: Vec::new(),
                });
            } else if let Some(c) = out.last_mut() {
                let mut parts = token.splitn(3, '\t');
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

    /// Every commit reachable from the first of `revs` and not from any `^rev`
    /// after it, with its parents.
    pub fn parents(&self, revs: &[String]) -> Result<HashMap<String, Vec<String>>> {
        let mut args = vec!["rev-list", "--parents"];
        args.extend(revs.iter().map(String::as_str));
        let text = self.run(&args)?;
        Ok(text
            .lines()
            .filter_map(|l| {
                let mut shas = l.split(' ').map(str::to_string);
                Some((shas.next()?, shas.collect()))
            })
            .collect())
    }

    /// The commit every one of `revs` descends from, if they share one.
    pub fn merge_base(&self, revs: &[String]) -> Option<String> {
        let mut args = vec!["merge-base", "--octopus"];
        args.extend(revs.iter().map(String::as_str));
        let text = self.run(&args).ok()?;
        Some(text.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// The history of every file under `dir` up to `at`, newest first, renames
    /// found, with the blobs before and after each change.
    pub fn history(&self, at: &str, dir: &str) -> Result<Vec<Change>> {
        let pathspec = if dir.is_empty() { "." } else { dir };
        let text = self.run(&[
            "log",
            "-z",
            "--relative",
            "--raw",
            "-M",
            "--no-abbrev",
            "--format=@@%H%x09%cs",
            at,
            "--",
            pathspec,
        ])?;
        let mut out: Vec<Change> = Vec::new();
        let mut tokens = tokens(&text);
        while let Some(token) = tokens.next() {
            if let Some(head) = token.strip_prefix("@@") {
                let (sha, date) = head.split_once('\t').unwrap_or((head, ""));
                out.push(Change {
                    sha: sha.to_string(),
                    date: date.to_string(),
                    entries: Vec::new(),
                });
            } else if let Some(meta) = token.strip_prefix(':') {
                // `:100644 100644 OLD NEW M`, then the path, or two for a
                // rename or a copy.
                let fields: Vec<&str> = meta.split(' ').collect();
                let status = fields.get(4).and_then(|s| s.chars().next()).unwrap_or('M');
                let first = tokens.next().unwrap_or("").to_string();
                let second = if matches!(status, 'R' | 'C') {
                    tokens.next().unwrap_or("").to_string()
                } else {
                    first.clone()
                };
                if let Some(c) = out.last_mut() {
                    c.entries.push(Entry {
                        status,
                        old_blob: fields.get(2).unwrap_or(&"").to_string(),
                        new_blob: fields.get(3).unwrap_or(&"").to_string(),
                        old_path: first,
                        path: second,
                    });
                }
            }
        }
        Ok(out)
    }

    /// A blob's contents, by its name.
    pub fn blob(&self, sha: &str) -> Option<Vec<u8>> {
        if sha.is_empty() || sha.bytes().all(|b| b == b'0') {
            return None;
        }
        let mut slot = self.blobs.borrow_mut();
        if slot.is_none() {
            let mut child = self
                .command()
                .args(["cat-file", "--batch"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let input = child.stdin.take()?;
            let output = BufReader::new(child.stdout.take()?);
            *slot = Some(Batch {
                child,
                input,
                output,
            });
        }
        let batch = slot.as_mut()?;
        writeln!(batch.input, "{sha}").ok()?;
        batch.input.flush().ok()?;
        // `SHA TYPE SIZE`, the contents, and a line break; or `SHA missing`.
        let mut header = String::new();
        batch.output.read_line(&mut header).ok()?;
        let size: usize = header.split(' ').nth(2)?.trim().parse().ok()?;
        let mut contents = vec![0; size + 1];
        batch.output.read_exact(&mut contents).ok()?;
        contents.pop();
        Some(contents)
    }

    /// The oldest commit reachable from `at` whose change to `path` added or
    /// removed `needle` — for a review, the commit that brought it here.
    pub fn introduced(&self, needle: &str, path: &str, at: &str) -> Option<String> {
        let pickaxe = format!("-S{needle}");
        let path = format!(":(literal){path}");
        let text = self
            .run(&["log", "--format=%H", &pickaxe, at, "--", &path])
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
        // Paths from the top of the work tree, whatever directory git ran in.
        Ok(text
            .split('\0')
            .filter(|e| e.len() > 3)
            .filter_map(|e| self.relative(&e[3..]))
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

    /// Whether history was cut short by a shallow clone.
    pub fn is_shallow(&self) -> bool {
        self.run(&["rev-parse", "--is-shallow-repository"])
            .is_ok_and(|s| s.trim() == "true")
    }

    /// Files changed on this branch since it left `rev`: from where the two
    /// last met, so what changed on `rev` since is not counted.
    pub fn changed_since(&self, rev: &str) -> Result<Vec<String>> {
        let range = format!("{rev}...HEAD");
        let text = self.run(&[
            "diff",
            "-z",
            "--relative",
            "--name-only",
            "--no-renames",
            &range,
        ])?;
        Ok(text
            .split('\0')
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    pub fn head(&self) -> Option<String> {
        self.resolve("HEAD")
    }
}

// loam — whether a page is still true.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// A page says what it covers (spec §4.3). git says what changed. A page is
// stale when a file it covers changed after the page was last read against
// the code — its `reviewed` commit, or, when it has never been reviewed, the
// last commit that changed the page itself, since whoever wrote that commit
// had the code in front of them. Nothing is written into the page; staleness
// is derived, every time, from the history as it is.
//
// Three things make that hard, and each is handled here rather than left to
// surprise somebody:
//
// - A reviewed commit that is not on this branch (0042). Squashing or
//   rebasing the branch it was made on leaves a name that means nothing here.
//   But the review itself — the `reviewed:` line — came here in some commit,
//   and that commit carried the code the page was read against. So the
//   baseline is the commit that brought the review onto this branch, found by
//   git's pickaxe; the review's date is the last resort.
// - Research, which goes stale because the world moved, not the code (0043):
//   a kind may say how long its pages stay true.
// - A page being updated in the same change as its code (0045) is not stale;
//   its author is plausibly on it.

use crate::covers::Covers;
use crate::git::{Commit, Git};
use crate::tree::{Page, Tree};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum How {
    /// The `reviewed` commit, which is on this branch.
    Reviewed,
    /// The commit that brought the review onto this branch.
    Introduced,
    /// The last commit on or before the review's date.
    Dated,
    /// Never reviewed: the last commit that changed the page.
    LastEdit,
    /// Never committed: the page is as new as the code.
    New,
}

#[derive(Clone, Debug)]
pub struct Baseline {
    pub how: How,
    pub commit: Option<String>,
    /// The day the page was last known to be true.
    pub date: Option<String>,
}

/// One pattern's share of what changed.
#[derive(Clone, Debug)]
pub struct PatternChange {
    pub pattern: String,
    pub commits: usize,
    pub added: u64,
    pub removed: u64,
    /// The newest commit's subject.
    pub latest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// A program writes it; it is never stale (spec §4.3).
    Generated,
    /// No `covers`, and no age limit: nothing to judge by. Not the same as fresh.
    Unknown,
    Fresh,
    Stale,
    /// Stale, but the page itself changed alongside: probably being updated.
    Updating,
}

#[derive(Clone, Debug)]
pub struct Freshness {
    pub path: String,
    pub state: State,
    pub baseline: Option<Baseline>,
    pub commits: Vec<Commit>,
    pub added: u64,
    pub removed: u64,
    pub patterns: Vec<PatternChange>,
    /// Days since the baseline date, when there is one.
    pub age: Option<i64>,
    /// The kind's limit, when the page has outlived it.
    pub aged: Option<i64>,
}

impl Freshness {
    pub fn lines(&self) -> u64 {
        self.added + self.removed
    }
}

pub struct Report {
    pub pages: Vec<Freshness>,
    /// Said once per run: how many pages were compared from somewhere other
    /// than their reviewed commit, and why.
    pub notes: Vec<String>,
}

/// `180d`, `26w`, `6m`, `1y` as a number of days.
pub fn parse_age(text: &str) -> Option<i64> {
    let text = text.trim();
    let (n, unit) = text.split_at(text.find(|c: char| !c.is_ascii_digit())?);
    let n: i64 = n.parse().ok()?;
    Some(match unit {
        "d" => n,
        "w" => n * 7,
        "m" => n * 30,
        "y" => n * 365,
        _ => return None,
    })
}

pub fn days_between(from: &str, to: &str) -> Option<i64> {
    let parse = |s: &str| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok();
    Some((parse(to)? - parse(from)?).num_days())
}

pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// A page's body: the text after its frontmatter. Frontmatter is where loam
/// itself writes — a review, a summary, an order — and a commit that changes
/// only that has not changed what the page says.
fn body_of(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_string();
    let body = match crate::tree::split(&text) {
        Ok((_, body, _)) => body,
        Err(()) => text,
    };
    // Adding frontmatter to a page that had none leaves a blank line where the
    // page began; that is not a change to what it says.
    body.trim_matches(|c: char| c == '\n' || c == ' ' || c == '\t')
        .to_string()
}

/// What the docs directory's history says about each page: every commit
/// that touched it, followed back through moves, and whether each changed
/// what the page says. Read with one `git log` and one `git cat-file`.
struct PageHistory<'a> {
    git: &'a Git,
    changes: Vec<crate::git::Change>,
    bodies: RefCell<HashMap<String, Option<String>>>,
}

/// One commit that touched a page, under the name it had there.
struct Touch {
    sha: String,
    date: String,
    old_blob: String,
    new_blob: String,
}

impl PageHistory<'_> {
    fn body(&self, blob: &str) -> Option<String> {
        if let Some(b) = self.bodies.borrow().get(blob) {
            return b.clone();
        }
        let body = self.git.blob(blob).map(|b| body_of(&b));
        self.bodies
            .borrow_mut()
            .insert(blob.to_string(), body.clone());
        body
    }

    /// The commits that touched `path`, newest first, following it back
    /// through every move, and stopping where it was created.
    fn touches(&self, path: &str) -> Vec<Touch> {
        let mut name = path.to_string();
        let mut out = Vec::new();
        for change in &self.changes {
            let Some(e) = change.entries.iter().find(|e| e.path == name) else {
                continue;
            };
            out.push(Touch {
                sha: change.sha.clone(),
                date: change.date.clone(),
                old_blob: e.old_blob.clone(),
                new_blob: e.new_blob.clone(),
            });
            match e.status {
                'A' | 'D' => break,
                'R' => name = e.old_path.clone(),
                _ => {}
            }
        }
        out
    }

    fn changed_body(&self, t: &Touch) -> bool {
        self.body(&t.old_blob) != self.body(&t.new_blob)
    }
}

pub struct Options<'a> {
    /// Judge as of this revision; `None` is HEAD, with the working tree.
    pub at: Option<&'a str>,
    /// Today, for ages; the revision's date when replaying.
    pub today: Option<String>,
    /// Judge only these pages; `None` is every page.
    pub only: Option<&'a HashSet<String>>,
}

/// Every commit in the range `graph` holds that is `from` or an ancestor of
/// it. Outside the range, everything is an ancestor of `from` already.
fn ancestors(graph: &HashMap<String, Vec<String>>, from: &str) -> HashSet<String> {
    let mut seen = HashSet::new();
    let mut stack = vec![from.to_string()];
    while let Some(sha) = stack.pop() {
        if let Some(parents) = graph.get(&sha)
            && seen.insert(sha)
        {
            stack.extend(parents.iter().cloned());
        }
    }
    seen
}

pub fn assess(tree: &Tree, git: &Git, opts: &Options) -> Result<Report> {
    let wanted = |path: &str| opts.only.is_none_or(|o| o.contains(path));
    let at = match opts.at {
        Some(rev) => git
            .resolve(rev)
            .ok_or_else(|| anyhow::anyhow!("no commit called {rev}"))?,
        None => match git.head() {
            Some(h) => h,
            None => return Ok(unborn(tree, &wanted)),
        },
    };
    let today = opts.today.clone().unwrap_or_else(today);
    let history = PageHistory {
        git,
        changes: git.history(&at, &tree.config.docs)?,
        bodies: RefCell::new(HashMap::new()),
    };
    let dirty: HashSet<String> = if opts.at.is_none() {
        git.uncommitted()?.into_iter().collect()
    } else {
        HashSet::new()
    };

    let mut notes = Notes::default();
    let mut pages = Vec::new();
    // Pages judged against the code: each with its covers and baseline.
    let mut judged: Vec<(Freshness, Covers, Vec<Touch>)> = Vec::new();

    for (path, page) in &tree.pages {
        if !wanted(path) {
            continue;
        }
        let covers = Covers::new(&page.covers);
        let limit = page
            .kind
            .as_deref()
            .and_then(|k| tree.config.kind(k))
            .and_then(|k| k.stale_after.as_deref())
            .and_then(parse_age);
        let mut f = Freshness {
            path: path.clone(),
            state: State::Fresh,
            baseline: None,
            commits: Vec::new(),
            added: 0,
            removed: 0,
            patterns: Vec::new(),
            age: None,
            aged: None,
        };
        if page.generated.is_some() {
            f.state = State::Generated;
            pages.push(f);
            continue;
        }
        if covers.is_empty() && limit.is_none() {
            f.state = State::Unknown;
            pages.push(f);
            continue;
        }

        let touches = history.touches(path);
        let baseline = baseline(git, page, opts, &at, &touches, &history, &mut notes);
        f.age = baseline
            .date
            .as_deref()
            .and_then(|d| days_between(d, &today));
        if let (Some(limit), Some(age)) = (limit, f.age)
            && age > limit
        {
            f.aged = Some(limit);
        }
        let new = baseline.how == How::New;
        f.baseline = Some(baseline);
        if covers.is_empty() || new {
            if f.aged.is_some() {
                f.state = if dirty.contains(path) {
                    State::Updating
                } else {
                    State::Stale
                };
            }
            pages.push(f);
        } else {
            judged.push((f, covers, touches));
        }
    }

    // One log for every judged page, over everything any of them covers, from
    // where all their baselines meet; each page then takes its own share.
    if !judged.is_empty() {
        let bases: Vec<String> = judged
            .iter()
            .filter_map(|(f, ..)| f.baseline.as_ref()?.commit.clone())
            .collect();
        let mut revs = vec![at.clone()];
        if bases.len() == judged.len()
            && let Some(base) = git.merge_base(&bases)
        {
            revs.push(format!("^{base}"));
        }
        let mut specs: Vec<String> = judged.iter().flat_map(|(_, c, _)| c.pathspecs()).collect();
        specs.sort();
        specs.dedup();
        // No pathspec at all would be every file: a page whose every pattern
        // leaves the repository covers nothing.
        let commits = if specs.is_empty() {
            Vec::new()
        } else {
            git.commits(&revs, &specs)?
        };
        let graph = git.parents(&revs)?;
        let mut before: HashMap<Option<String>, HashSet<String>> = HashMap::new();

        for (mut f, covers, touches) in judged {
            let base = f.baseline.as_ref().and_then(|b| b.commit.clone());
            let old = before.entry(base.clone()).or_insert_with(|| {
                base.as_deref()
                    .map_or_else(HashSet::new, |b| ancestors(&graph, b))
            });
            let in_range = |sha: &str| graph.contains_key(sha) && !old.contains(sha);
            // Newest first, so the first subject seen for a pattern is its latest.
            for commit in commits.iter().filter(|c| in_range(&c.sha)) {
                let mut per: Vec<(String, u64, u64)> = Vec::new();
                // A page that covers the directory it is in does not go
                // stale by being written.
                let files = commit
                    .files
                    .iter()
                    .filter(|fc| fc.path != f.path && covers.covers(&fc.path));
                for fc in files {
                    let pattern = covers.which(&fc.path).unwrap_or("").to_string();
                    match per.iter_mut().find(|(p, _, _)| *p == pattern) {
                        Some(e) => {
                            e.1 += fc.added;
                            e.2 += fc.removed;
                        }
                        None => per.push((pattern, fc.added, fc.removed)),
                    }
                }
                if per.is_empty() {
                    continue;
                }
                for (pattern, added, removed) in per {
                    f.added += added;
                    f.removed += removed;
                    match f.patterns.iter_mut().find(|p| p.pattern == pattern) {
                        Some(e) => {
                            e.commits += 1;
                            e.added += added;
                            e.removed += removed;
                        }
                        None => f.patterns.push(PatternChange {
                            pattern,
                            commits: 1,
                            added,
                            removed,
                            latest: commit.subject.clone(),
                        }),
                    }
                }
                f.commits.push(commit.clone());
            }
            f.patterns
                .sort_by_key(|p| std::cmp::Reverse(p.added + p.removed));
            // The page's author is plausibly updating it when what it says
            // changed in or after the newest change to what it covers. An
            // edit made before that change does not answer it.
            let updating = f.commits.first().is_some_and(|newest| {
                let earlier = ancestors(&graph, &newest.sha);
                touches.iter().any(|t| {
                    in_range(&t.sha)
                        && (t.sha == newest.sha || !earlier.contains(&t.sha))
                        && history.changed_body(t)
                })
            });
            if !f.commits.is_empty() || f.aged.is_some() {
                f.state = if updating || dirty.contains(&f.path) {
                    State::Updating
                } else {
                    State::Stale
                };
            }
            pages.push(f);
        }
    }

    // A shallow clone — CI's default — has no history before its first
    // commit, so every page looks as if it was written there, and nothing can
    // be stale. Silence would be a claim.
    let mut notes = notes.into_vec();
    if git.is_shallow() {
        notes.insert(
            0,
            "this is a shallow clone, so history stops short and pages read as fresher than they are; \
             fetch the whole history (in GitHub Actions, `fetch-depth: 0` on actions/checkout)"
                .to_string(),
        );
    }
    pages.sort_by(|a, b| {
        b.lines()
            .cmp(&a.lines())
            .then(b.aged.is_some().cmp(&a.aged.is_some()))
            .then(a.path.cmp(&b.path))
    });
    Ok(Report { pages, notes })
}

/// A repository with no commits: nothing has changed since anything.
fn unborn(tree: &Tree, wanted: &dyn Fn(&str) -> bool) -> Report {
    let pages = tree
        .pages
        .iter()
        .filter(|(path, _)| wanted(path))
        .map(|(path, page)| Freshness {
            path: path.clone(),
            state: if page.generated.is_some() {
                State::Generated
            } else if page.covers.is_empty() {
                State::Unknown
            } else {
                State::Fresh
            },
            baseline: None,
            commits: Vec::new(),
            added: 0,
            removed: 0,
            patterns: Vec::new(),
            age: None,
            aged: None,
        })
        .collect();
    Report {
        pages,
        notes: vec!["the repository has no commits yet, so nothing can have changed".into()],
    }
}

/// How many pages were compared from somewhere other than their reviewed
/// commit, said once per run rather than once per page.
#[derive(Default)]
struct Notes {
    moved: usize,
    dated: usize,
    later: usize,
}

impl Notes {
    fn into_vec(self) -> Vec<String> {
        let mut notes = Vec::new();
        if self.moved > 0 {
            notes.push(format!(
                "{} page(s) were reviewed at a commit that is not on this branch — squashed or rebased away — \
                 and were compared from the commit that brought the review here",
                self.moved
            ));
        }
        if self.dated > 0 {
            notes.push(format!(
                "{} page(s) were reviewed at a commit this repository does not have, \
                 and were compared from the last commit on or before the review's date",
                self.dated
            ));
        }
        if self.later > 0 {
            notes.push(format!(
                "{} page(s) were reviewed after the commit being judged, so the review was set aside",
                self.later
            ));
        }
        notes
    }
}

fn baseline(
    git: &Git,
    page: &Page,
    opts: &Options,
    at: &str,
    touches: &[Touch],
    history: &PageHistory,
    notes: &mut Notes,
) -> Baseline {
    if let Some(r) = &page.reviewed {
        let sha = git.resolve(&r.commit);
        match &sha {
            Some(sha) if git.is_ancestor(sha, at) => {
                return Baseline {
                    how: How::Reviewed,
                    commit: Some(sha.clone()),
                    date: Some(r.date.clone()),
                };
            }
            // Judging an earlier commit than the review: as of then, there
            // was no review.
            Some(sha) if opts.at.is_some() && git.is_ancestor(at, sha) => notes.later += 1,
            _ => {
                if let Some(sha) = git.introduced(&r.commit, &page.path, at) {
                    notes.moved += 1;
                    return Baseline {
                        how: How::Introduced,
                        commit: Some(sha),
                        date: Some(r.date.clone()),
                    };
                }
                notes.dated += 1;
                return Baseline {
                    how: How::Dated,
                    commit: git.commit_on_or_before(&r.date, at),
                    date: Some(r.date.clone()),
                };
            }
        }
    }
    match touches.iter().find(|t| history.changed_body(t)) {
        Some(t) => Baseline {
            how: How::LastEdit,
            commit: Some(t.sha.clone()),
            date: Some(t.date.clone()),
        },
        None => Baseline {
            how: How::New,
            commit: None,
            date: None,
        },
    }
}

/// A page whose covered files are among a set just changed (0050).
pub struct Touched {
    pub path: String,
    /// Each covering pattern, and the changed files under it.
    pub patterns: Vec<(String, Vec<String>)>,
    /// The page itself changed too: plausibly already being updated.
    pub page_changed: bool,
}

/// Which pages cover any of `changed`: what an agent should reread before it
/// stops, having just changed those files.
pub fn touched(tree: &Tree, changed: &[String]) -> Vec<Touched> {
    let mut out = Vec::new();
    for (path, page) in &tree.pages {
        if page.generated.is_some() || page.covers.is_empty() {
            continue;
        }
        let covers = Covers::new(&page.covers);
        let mut patterns: Vec<(String, Vec<String>)> = Vec::new();
        for f in changed {
            if let Some(p) = covers.which(f) {
                match patterns.iter_mut().find(|(q, _)| q == p) {
                    Some((_, files)) => files.push(f.clone()),
                    None => patterns.push((p.to_string(), vec![f.clone()])),
                }
            }
        }
        if !patterns.is_empty() {
            out.push(Touched {
                path: path.clone(),
                patterns,
                page_changed: changed.contains(path),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages() {
        assert_eq!(parse_age("180d"), Some(180));
        assert_eq!(parse_age("2w"), Some(14));
        assert_eq!(parse_age("6m"), Some(180));
        assert_eq!(parse_age("1y"), Some(365));
        assert_eq!(parse_age("soon"), None);
        assert_eq!(parse_age("12"), None);
        assert_eq!(days_between("2026-01-01", "2026-03-01"), Some(59));
    }
}

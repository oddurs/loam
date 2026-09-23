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

/// A page's body at a commit: the text after its frontmatter. Frontmatter is
/// where loam itself writes — a review, a summary, an order — and a commit that
/// changes only that has not changed what the page says.
fn body_at(git: &Git, rev: &str, path: &str) -> Option<String> {
    let bytes = git.show(rev, path)?;
    let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_string();
    let body = match crate::tree::split(&text) {
        Ok((_, body, _)) => body,
        Err(()) => text,
    };
    // Adding frontmatter to a page that had none leaves a blank line where the
    // page began; that is not a change to what it says.
    Some(
        body.trim_matches(|c: char| c == '\n' || c == ' ' || c == '\t')
            .to_string(),
    )
}

/// Whether `commit` changed what the page says, not only its frontmatter.
fn changed_body(git: &Git, commit: &str, path: &str) -> bool {
    let parent = format!("{commit}^");
    body_at(git, commit, path) != body_at(git, &parent, path)
}

/// The last commit at or before `at` that changed the page's body.
fn last_body_change(git: &Git, path: &str, at: &str) -> Option<(String, String)> {
    git.path_history(path, at)
        .into_iter()
        .find(|(sha, _)| changed_body(git, sha, path))
}

pub struct Options<'a> {
    /// Judge as of this revision; `None` is HEAD, with the working tree.
    pub at: Option<&'a str>,
    /// Today, for ages; the revision's date when replaying.
    pub today: Option<String>,
}

pub fn assess(tree: &Tree, git: &Git, opts: &Options) -> Result<Report> {
    let at = match opts.at {
        Some(rev) => git
            .resolve(rev)
            .ok_or_else(|| anyhow::anyhow!("no commit called {rev}"))?,
        None => match git.head() {
            Some(h) => h,
            None => anyhow::bail!("the repository has no commits yet, so nothing can have changed"),
        },
    };
    let today = opts.today.clone().unwrap_or_else(today);
    let last = git.last_changes(&at, &tree.config.docs)?;
    let dirty: HashSet<String> = if opts.at.is_none() {
        git.uncommitted()?.into_iter().collect()
    } else {
        HashSet::new()
    };

    let mut ranges: HashMap<(Option<String>, String), Vec<Commit>> = HashMap::new();
    let mut moved = 0;
    let mut dated = 0;
    let mut pages = Vec::new();

    for (path, page) in &tree.pages {
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

        let baseline = baseline(git, page, &at, &last, &mut moved, &mut dated);
        f.age = baseline
            .date
            .as_deref()
            .and_then(|d| days_between(d, &today));
        if let (Some(limit), Some(age)) = (limit, f.age)
            && age > limit
        {
            f.aged = Some(limit);
        }

        let mut updating = false;
        if !covers.is_empty() && baseline.how != How::New {
            let specs = covers.pathspecs();
            let key = (baseline.commit.clone(), specs.join("\0"));
            if !ranges.contains_key(&key) {
                // The page itself too, to see whether it changed alongside.
                let mut paths = specs.clone();
                paths.push(format!(":(literal){path}"));
                ranges.insert(
                    key.clone(),
                    git.commits(baseline.commit.as_deref(), &at, &paths)?,
                );
            }
            let range = &ranges[&key];
            // Newest first, so the first subject seen for a pattern is its latest.
            for commit in range {
                let mut per: Vec<(String, u64, u64)> = Vec::new();
                for fc in commit.files.iter().filter(|fc| covers.covers(&fc.path)) {
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
            // The page changed after the first change to what it covers: its
            // author is plausibly updating it. A commit that only records a
            // review comes before that change, so it does not count.
            if let Some(oldest) = f.commits.last() {
                let upto = range.iter().position(|c| c.sha == oldest.sha).unwrap_or(0);
                updating = range[..=upto].iter().any(|c| {
                    c.files.iter().any(|fc| fc.path == *path) && changed_body(git, &c.sha, path)
                });
            }
        }

        if !f.commits.is_empty() || f.aged.is_some() {
            f.state = if updating || dirty.contains(path) {
                State::Updating
            } else {
                State::Stale
            };
        }
        f.baseline = Some(baseline);
        pages.push(f);
    }

    let mut notes = Vec::new();
    // A shallow clone — CI's default — has no history before its first
    // commit, so every page looks as if it was written there, and nothing can
    // be stale. Silence would be a claim.
    if git.is_shallow() {
        notes.push(
            "this is a shallow clone, so history stops short and pages read as fresher than they are; \
             fetch the whole history (in GitHub Actions, `fetch-depth: 0` on actions/checkout)"
                .to_string(),
        );
    }
    if moved > 0 {
        notes.push(format!(
            "{moved} page(s) were reviewed at a commit that is not on this branch — squashed or rebased away — \
             and were compared from the commit that brought the review here"
        ));
    }
    if dated > 0 {
        notes.push(format!(
            "{dated} page(s) were reviewed at a commit this repository does not have, \
             and were compared from the last commit on or before the review's date"
        ));
    }
    pages.sort_by(|a, b| {
        b.lines()
            .cmp(&a.lines())
            .then(b.aged.is_some().cmp(&a.aged.is_some()))
            .then(a.path.cmp(&b.path))
    });
    Ok(Report { pages, notes })
}

fn baseline(
    git: &Git,
    page: &Page,
    at: &str,
    last: &HashMap<String, (String, String)>,
    moved: &mut usize,
    dated: &mut usize,
) -> Baseline {
    if let Some(r) = &page.reviewed {
        if let Some(sha) = git.resolve(&r.commit)
            && git.is_ancestor(&sha, at)
        {
            return Baseline {
                how: How::Reviewed,
                commit: Some(sha),
                date: Some(r.date.clone()),
            };
        }
        if let Some(sha) = git.introduced(&r.commit, &page.path, at) {
            *moved += 1;
            return Baseline {
                how: How::Introduced,
                commit: Some(sha),
                date: Some(r.date.clone()),
            };
        }
        *dated += 1;
        return Baseline {
            how: How::Dated,
            commit: git.commit_on_or_before(&r.date, at),
            date: Some(r.date.clone()),
        };
    }
    if !last.contains_key(&page.path) {
        return Baseline {
            how: How::New,
            commit: None,
            date: None,
        };
    }
    match last_body_change(git, &page.path, at) {
        Some((sha, date)) => Baseline {
            how: How::LastEdit,
            commit: Some(sha),
            date: Some(date),
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

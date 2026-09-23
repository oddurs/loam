// loam — staleness, review, and the history they are read from.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Every test builds a real git repository, commits into it on fixed dates,
// and asks loam what it makes of the history.

use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Repo {
    dir: tempfile::TempDir,
    day: std::cell::Cell<u32>,
}

const CONFIG: &str = "format = 1\n\n[[kind]]\nname = \"research\"\ndir = \"research\"\nstale_after = \"30d\"\n\n[[kind]]\nname = \"page\"\n";

impl Repo {
    fn new() -> Repo {
        let repo = Repo {
            dir: tempfile::tempdir().unwrap(),
            day: std::cell::Cell::new(1),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.write("loam.toml", CONFIG);
        repo
    }

    fn path(&self, p: &str) -> PathBuf {
        self.dir.path().join(p)
    }

    fn write(&self, path: &str, text: &str) {
        let p = self.path(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.path(path)).unwrap()
    }

    fn git(&self, args: &[&str]) -> String {
        let date = format!("2026-01-{:02}T12:00:00", self.day.get());
        let out = Command::new("git")
            .args(args)
            .current_dir(self.dir.path())
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Commit everything, a day after the last commit.
    fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", message]);
        self.day.set(self.day.get() + 1);
        self.git(&["rev-parse", "HEAD"])
    }

    fn loam(&self, args: &[&str]) -> Output {
        self.loam_env(args, &[])
    }

    fn loam_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_loam"));
        c.args(args)
            .current_dir(self.dir.path())
            .env("NO_COLOR", "1")
            .env_remove("GITHUB_ACTIONS")
            // A person, unless a test says otherwise: these tests run inside agents.
            .env_remove("LOAM_AGENT")
            .env_remove("CAIRN_AGENT")
            .env_remove("AI_AGENT")
            .env_remove("CLAUDECODE");
        for (k, v) in env {
            c.env(k, v);
        }
        c.output().unwrap()
    }

    fn stale(&self) -> Value {
        let out = self.loam(&["stale", "--json"]);
        serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stderr)))
    }

    fn state(&self, page: &str) -> String {
        let v = self.stale();
        v["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == page)
            .unwrap()["state"]
            .as_str()
            .unwrap()
            .to_string()
    }
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

/// A repository with an engine and a page describing it, both committed.
fn engine() -> Repo {
    let repo = Repo::new();
    repo.write("src/engine/step.rs", "fn step() {}\n");
    repo.write("src/engine/tests/t.rs", "fn t() {}\n");
    repo.write("src/other.rs", "fn other() {}\n");
    repo.write(
        "docs/engine.md",
        "---\ncovers:\n  - src/engine\n  - \"!src/engine/tests\"\n---\n\n# The engine\n\nIt steps.\n",
    );
    repo.write("docs/notes.md", "# Notes\n");
    repo.commit("the engine, and a page about it");
    repo
}

#[test]
fn a_page_whose_code_changed_is_stale_and_says_how() {
    let repo = engine();
    assert_eq!(repo.state("docs/engine.md"), "fresh");
    repo.write("src/other.rs", "fn other() { 1 }\n");
    repo.write("src/engine/tests/t.rs", "fn t() { 2 }\n");
    repo.commit("uncovered changes");
    assert_eq!(
        repo.state("docs/engine.md"),
        "fresh",
        "neither file is covered"
    );

    repo.write("src/engine/step.rs", "fn step() {\n    go();\n}\n");
    repo.commit("split the scheduler out");
    let v = repo.stale();
    let page = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "docs/engine.md")
        .unwrap();
    assert_eq!(page["state"], "stale");
    assert_eq!(page["patterns"][0]["pattern"], "src/engine");
    assert_eq!(page["patterns"][0]["commits"], 1);
    assert_eq!(page["patterns"][0]["latest"], "split the scheduler out");
    assert_eq!(page["baseline"]["how"], "last-edit");

    let out = repo.loam(&["stale"]);
    assert_eq!(code(&out), 1, "stale pages found");
    let t = text(&out);
    assert!(
        t.contains("docs/engine.md   never reviewed; last edited"),
        "{t}"
    );
    assert!(
        t.contains("src/engine  1 commit(s), +3 −1   \"split the scheduler out\""),
        "{t}"
    );
    assert!(
        t.contains("1 stale, 0 being updated, 0 fresh, 1 unknown (no covers)"),
        "{t}"
    );
}

#[test]
fn pages_without_covers_are_unknown_not_fresh() {
    let repo = engine();
    let v = repo.stale();
    let notes = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "docs/notes.md")
        .unwrap();
    assert_eq!(notes["state"], "unknown");
    let out = repo.loam(&["stale", "--all"]);
    assert!(
        text(&out).contains("docs/notes.md  unknown: no covers"),
        "{}",
        text(&out)
    );
}

#[test]
fn changes_to_whitespace_alone_do_not_count() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn   step()   {}\n\n\n");
    repo.commit("reformat");
    assert_eq!(repo.state("docs/engine.md"), "fresh");
}

#[test]
fn review_clears_a_page_in_one_command() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 2 }\n");
    repo.commit("change");
    assert_eq!(repo.state("docs/engine.md"), "stale");
    let head = repo.git(&["rev-parse", "HEAD"]);
    let out = repo.loam(&["review", "docs/engine.md"]);
    assert_eq!(code(&out), 0, "{}", text(&out));
    assert!(
        repo.read("docs/engine.md")
            .contains(&format!("reviewed:\n  commit: {head}\n  date: "))
    );
    assert_eq!(repo.state("docs/engine.md"), "fresh");
    repo.commit("review");
    assert_eq!(
        repo.state("docs/engine.md"),
        "fresh",
        "a commit recording a review changes no code"
    );
    repo.write("src/engine/step.rs", "fn step() { 3 }\n");
    repo.commit("change again");
    let v = repo.stale();
    let page = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "docs/engine.md")
        .unwrap();
    assert_eq!(page["state"], "stale");
    assert_eq!(page["baseline"]["how"], "reviewed");
    assert_eq!(
        page["patterns"][0]["commits"], 1,
        "only what changed since the review"
    );
}

#[test]
fn review_warns_about_uncommitted_covered_changes() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 9 }\n");
    let out = repo.loam(&["review", "docs/engine.md"]);
    assert_eq!(code(&out), 0, "a warning, not a refusal");
    let t = text(&out);
    assert!(
        t.contains("warning: docs/engine.md covers a file with uncommitted changes"),
        "{t}"
    );
    assert!(t.contains("src/engine/step.rs"), "{t}");
    let clean = engine();
    assert!(!text(&clean.loam(&["review", "docs/engine.md"])).contains("warning"));
}

#[test]
fn review_keeps_a_log_when_asked() {
    let repo = engine();
    assert_eq!(
        code(&repo.loam(&["review", "docs/engine.md", "--note", "still steps"])),
        0
    );
    assert_eq!(
        code(&repo.loam(&["review", "docs/engine.md", "--note", "still steps, twice"])),
        0
    );
    let page = repo.read("docs/engine.md");
    assert!(page.contains("\n## Review log\n\n- "), "{page}");
    assert!(page.ends_with(": still steps, twice\n"), "{page}");
    assert_eq!(page.matches("## Review log").count(), 1);
    assert_eq!(page.matches("reviewed:").count(), 1);
}

#[test]
fn a_squashed_branch_does_not_make_every_page_stale() {
    let repo = engine();
    repo.write(
        "docs/other.md",
        "---\ncovers: src/other.rs\n---\n\n# Other\n",
    );
    repo.commit("another page");
    // On a branch: change the engine and the other file, update both pages
    // to match, and review them there.
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.write("src/engine/step.rs", "fn step() { new() }\n");
    repo.write("src/other.rs", "fn other() { new() }\n");
    repo.commit("rework");
    assert_eq!(
        code(&repo.loam(&["review", "docs/engine.md", "docs/other.md"])),
        0
    );
    repo.commit("review the pages against the rework");
    // Squash-merge it, a few days later, and lose the branch.
    repo.git(&["checkout", "-q", "main"]);
    repo.day.set(repo.day.get() + 3);
    repo.git(&["merge", "-q", "--squash", "feature"]);
    repo.commit("rework (#1)");
    repo.git(&["branch", "-q", "-D", "feature"]);
    repo.git(&["reflog", "expire", "--expire=now", "--all"]);
    repo.git(&["gc", "-q", "--prune=now"]);

    let out = repo.loam(&["stale"]);
    let t = text(&out);
    assert!(
        t.contains("0 stale"),
        "only real changes, and there are none: {t}"
    );
    assert_eq!(
        t.matches("note:").count(),
        1,
        "the fallback is announced once: {t}"
    );
    assert!(
        t.contains("2 page(s) were reviewed at a commit that is not on this branch"),
        "{t}"
    );

    // A real change after the merge is still found.
    repo.write("src/engine/step.rs", "fn step() { newer() }\n");
    repo.commit("after");
    let v = repo.stale();
    let page = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "docs/engine.md")
        .unwrap();
    assert_eq!(page["state"], "stale");
    assert_eq!(page["baseline"]["how"], "introduced");
    assert_eq!(page["commits"].as_array().unwrap().len(), 1);
}

#[test]
fn a_review_nothing_here_knows_of_falls_back_to_its_date() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 1 }\n");
    repo.commit("on the second");
    repo.write("src/engine/step.rs", "fn step() { 2 }\n");
    repo.commit("on the third");
    // A review made elsewhere, on a commit this clone never had, dated the
    // second — as a page arriving by copy, not by history, would be.
    let page = repo.read("docs/engine.md").replace(
        "---\ncovers:",
        "---\nreviewed:\n  commit: 0123456789abcdef0123456789abcdef01234567\n  date: 2026-01-02\ncovers:",
    );
    std::fs::write(repo.path("docs/engine.md"), page).unwrap();
    let v = repo.stale();
    let p = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["path"] == "docs/engine.md")
        .unwrap();
    assert_eq!(p["baseline"]["how"], "dated");
    assert_eq!(
        p["commits"].as_array().unwrap().len(),
        1,
        "only the change after the second"
    );
    assert!(v["notes"][0].as_str().unwrap().contains("does not have"));
}

#[test]
fn research_decays_by_age() {
    let repo = engine();
    repo.write(
        "docs/research/crates.md",
        "---\nreviewed:\n  commit: abcdef1\n  date: 2025-06-01\n---\n\n# Markdown crates\n",
    );
    repo.write(
        "docs/recent.md",
        "---\nreviewed:\n  commit: abcdef1\n  date: 2025-06-01\n---\n\n# Not research\n",
    );
    repo.commit("pages");
    let v = repo.stale();
    let find = |p: &str| {
        v["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["path"] == p)
            .unwrap()
            .clone()
    };
    let research = find("docs/research/crates.md");
    assert_eq!(research["state"], "stale");
    assert_eq!(research["stale_after"], 30);
    assert_eq!(
        find("docs/recent.md")["state"],
        "unknown",
        "a kind without a policy is unaffected"
    );
    let out = repo.loam(&["stale"]);
    assert!(
        text(&out).contains("older than its kind's 30 days"),
        "{}",
        text(&out)
    );

    // show says so, and lists what the research was read from, and when.
    repo.write(
        "docs/research/sourced.md",
        "---\nsources:\n  - title: A survey of crates\n    url: https://example.com/survey\n    read: 2025-06-01\n  - https://example.com/other\n---\n\n# Sourced\n",
    );
    repo.commit("sourced");
    let show = text(&repo.loam(&["show", "docs/research/crates.md"]));
    assert!(
        show.contains("fresh  stale: older than its kind's 30 days"),
        "{show}"
    );
    let show = text(&repo.loam(&["show", "docs/research/sourced.md"]));
    assert!(
        show.contains("sources  A survey of crates, read 2025-06-01"),
        "{show}"
    );
    assert!(
        show.contains("           https://example.com/other"),
        "{show}"
    );
}

#[test]
fn generated_pages_are_never_stale() {
    let repo = engine();
    repo.write(
        "docs/status.md",
        "---\ngenerated: make status\ncovers: src\n---\n\n# Status\n",
    );
    repo.commit("status");
    repo.write("src/engine/step.rs", "fn step() { 4 }\n");
    repo.commit("change");
    assert_eq!(repo.state("docs/status.md"), "generated");
}

#[test]
fn frontmatter_alone_is_not_an_edit() {
    // Adopting a folder adds a summary to every page; that commit must not
    // count as having read the page against the code.
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 5 }\n");
    repo.commit("change");
    let page = repo
        .read("docs/engine.md")
        .replace("---\ncovers:", "---\nsummary: What steps.\ncovers:");
    std::fs::write(repo.path("docs/engine.md"), page).unwrap();
    repo.commit("adopt: summaries");
    assert_eq!(repo.state("docs/engine.md"), "stale");
}

#[test]
fn giving_a_page_its_first_frontmatter_is_not_an_edit() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn a() {}\n");
    repo.write("docs/a.md", "# A\n\nAbout a.\n");
    repo.commit("a, and a page");
    repo.write("src/a.rs", "fn a() { 1 }\n");
    repo.commit("change a");
    repo.write(
        "docs/a.md",
        "---\ncovers: src/a.rs\n---\n\n# A\n\nAbout a.\n",
    );
    repo.commit("say what the page covers");
    assert_eq!(repo.state("docs/a.md"), "stale");
}

#[test]
fn covers_that_match_nothing_are_reported() {
    let repo = engine();
    assert!(!text(&repo.loam(&["check"])).contains("covers-nothing"));
    std::fs::remove_dir_all(repo.path("src/engine")).unwrap();
    repo.commit("delete the engine");
    let out = repo.loam(&["check"]);
    let t = text(&out);
    assert!(t.contains("loam: docs/engine.md:2: covers `src/engine`, which matches no file in the repository [covers-nothing]"), "{t}");
    // Deleting the code is a change, so the page is stale at first; once it is
    // reviewed, or merely edited, it reads as fresh while describing nothing.
    assert_eq!(repo.state("docs/engine.md"), "stale");
    assert_eq!(code(&repo.loam(&["review", "docs/engine.md"])), 0);
    repo.commit("review");
    assert_eq!(
        repo.state("docs/engine.md"),
        "fresh",
        "which is exactly why check says so"
    );
    assert!(text(&repo.loam(&["check"])).contains("[covers-nothing]"));
}

#[test]
fn check_stale_warns_and_annotates_without_failing() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 6 }\n");
    repo.commit("change");
    let plain = repo.loam(&["check"]);
    assert!(!text(&plain).contains("stale"), "only with --stale");
    let out = repo.loam(&["check", "--stale"]);
    assert_eq!(code(&out), 0, "a stale page is not a broken build");
    let t = text(&out);
    assert!(t.contains("loam: docs/engine.md:2: stale: src/engine changed in 1 commit(s), +1 −1; never reviewed"), "{t}");
    assert_eq!(
        code(&repo.loam(&["check", "--stale", "--strict"])),
        1,
        "unless the project wants the gate"
    );

    let actions = repo.loam_env(&["check", "--stale"], &[("GITHUB_ACTIONS", "true")]);
    let stdout = String::from_utf8_lossy(&actions.stdout);
    assert!(stdout.contains("::warning file=docs/engine.md,line=2,title=loam stale::stale: src/engine changed in 1 commit(s)"), "{stdout}");
}

#[test]
fn a_page_changed_alongside_its_code_is_not_warned_about() {
    let repo = engine();
    repo.git(&["checkout", "-q", "-b", "pr"]);
    repo.write("src/engine/step.rs", "fn step() { 7 }\n");
    repo.write(
        "docs/engine.md",
        &repo
            .read("docs/engine.md")
            .replace("It steps.", "It steps, now by sevens."),
    );
    repo.commit("change the engine and say so");
    assert_eq!(
        repo.state("docs/engine.md"),
        "fresh",
        "edited in the same commit: its body is newer"
    );

    // Reviewed first, then code and page changed together in a later diff.
    let repo = engine();
    assert_eq!(code(&repo.loam(&["review", "docs/engine.md"])), 0);
    repo.commit("review");
    repo.write("src/engine/step.rs", "fn step() { 8 }\n");
    repo.commit("change the engine");
    repo.write(
        "docs/engine.md",
        &repo
            .read("docs/engine.md")
            .replace("It steps.", "It steps by eights."),
    );
    repo.commit("and the page");
    assert_eq!(repo.state("docs/engine.md"), "updating");
    let out = repo.loam(&["check", "--stale", "--strict"]);
    assert_eq!(code(&out), 0, "{}", text(&out));
    // The working tree counts too.
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 9 }\n");
    repo.commit("change");
    repo.write(
        "docs/engine.md",
        &repo
            .read("docs/engine.md")
            .replace("It steps.", "It steps by nines."),
    );
    assert_eq!(repo.state("docs/engine.md"), "updating");
}

#[test]
fn replaying_history_with_at() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 10 }\n");
    let changed = repo.commit("change");
    let before = repo.git(&["rev-parse", "HEAD~1"]);
    let at = |rev: &str| {
        let out = repo.loam(&["stale", "--json", "--at", rev]);
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        v["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == "docs/engine.md")
            .unwrap()["state"]
            .clone()
    };
    assert_eq!(at(&before), "fresh");
    assert_eq!(at(&changed), "stale");
}

#[test]
fn a_shallow_clone_says_so() {
    let repo = engine();
    repo.write("src/engine/step.rs", "fn step() { 11 }\n");
    repo.commit("change");
    let clone = tempfile::tempdir().unwrap();
    let url = format!("file://{}", repo.dir.path().display());
    let out = Command::new("git")
        .args(["clone", "-q", "--depth", "1", &url])
        .arg(clone.path().join("c"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = Command::new(env!("CARGO_BIN_EXE_loam"))
        .args(["check", "--stale"])
        .current_dir(clone.path().join("c"))
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(
        text(&out).contains("note: this is a shallow clone"),
        "{}",
        text(&out)
    );
}

// What follows was found by an agent told to make staleness wrong, or slow.

fn page_covering(covers: &str) -> String {
    format!("---\ncovers:\n  - {covers}\n---\n\n# P\n\nBody.\n")
}

#[test]
fn a_project_in_a_subdirectory_of_its_repository() {
    let repo = Repo::new();
    std::fs::remove_file(repo.path("loam.toml")).unwrap();
    repo.write("proj/loam.toml", CONFIG);
    repo.write("proj/src/a.rs", "fn a() {}\n");
    repo.write("proj/docs/p.md", &page_covering("src/a.rs"));
    repo.commit("init");
    repo.write("proj/src/a.rs", "fn a() { 1 }\n");
    repo.commit("change");
    let out = Command::new(env!("CARGO_BIN_EXE_loam"))
        .args(["stale", "--json"])
        .current_dir(repo.path("proj"))
        .env_remove("CLAUDECODE")
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["pages"][0]["state"], "stale", "{v}");
    repo.write("proj/src/a.rs", "fn a() { 2 }\n");
    let out = Command::new(env!("CARGO_BIN_EXE_loam"))
        .args(["stale", "--working-tree"])
        .current_dir(repo.path("proj"))
        .env_remove("CLAUDECODE")
        .output()
        .unwrap();
    assert!(
        text(&out).contains("docs/p.md   covers src/a.rs"),
        "{}",
        text(&out)
    );
}

#[test]
fn a_page_covering_its_own_directory_is_not_stale_from_being_written() {
    let repo = Repo::new();
    repo.write("docs/p.md", &page_covering("docs"));
    repo.commit("init");
    let out = repo.loam(&["review", "docs/p.md"]);
    assert!(!text(&out).contains("lock"), "{}", text(&out));
    repo.commit("review");
    assert_eq!(repo.state("docs/p.md"), "fresh");
    repo.write("docs/other.md", "# Other\n");
    repo.commit("another page");
    assert_eq!(repo.state("docs/p.md"), "stale", "other pages still count");
}

#[test]
fn an_edit_before_the_latest_change_is_not_an_update() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn a() {}\n");
    repo.write("docs/p.md", &page_covering("src"));
    repo.commit("init");
    assert_eq!(code(&repo.loam(&["review", "docs/p.md"])), 0);
    repo.commit("review");
    repo.write("src/a.rs", "fn a() { 1 }\n");
    repo.commit("a small change");
    let page = repo.read("docs/p.md").replace("Body", "The body");
    repo.write("docs/p.md", &page);
    repo.commit("a typo");
    assert_eq!(
        repo.state("docs/p.md"),
        "updating",
        "edited after the change"
    );
    repo.write("src/b.rs", "fn b() {}\n".repeat(50).as_str());
    repo.commit("a large change");
    assert_eq!(
        repo.state("docs/p.md"),
        "stale",
        "the typo answers nothing newer"
    );
}

#[test]
fn paths_git_would_quote_are_read() {
    let repo = Repo::new();
    for (i, name) in ["q\"x.rs", "b\\s.rs", "tab\tx.rs"].iter().enumerate() {
        repo.write(&format!("src/{name}"), "x\n");
        let covers = format!(
            "\"src/{}\"",
            name.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\t', "\\t")
        );
        repo.write(&format!("docs/p{i}.md"), &page_covering(&covers));
    }
    repo.commit("init");
    for name in ["q\"x.rs", "b\\s.rs", "tab\tx.rs"] {
        repo.write(&format!("src/{name}"), "y\n");
    }
    repo.commit("change");
    for i in 0..3 {
        assert_eq!(repo.state(&format!("docs/p{i}.md")), "stale", "p{i}");
    }
}

#[test]
fn a_pattern_leaving_the_repository_spoils_nothing_else() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn a() {}\n");
    repo.write("docs/a.md", &page_covering("src"));
    repo.write("docs/bad.md", &page_covering("../outside"));
    repo.commit("init");
    repo.write("src/a.rs", "fn a() { 1 }\n");
    repo.commit("change");
    assert_eq!(repo.state("docs/a.md"), "stale");
    let out = repo.loam(&["check"]);
    assert!(
        text(&out).contains("which leaves the repository"),
        "{}",
        text(&out)
    );
    let out = repo.loam(&["context", "src/a.rs"]);
    assert!(text(&out).contains("STALE"), "{}", text(&out));
}

#[test]
fn moving_a_page_does_not_make_it_fresh() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn a() {}\n");
    repo.write("docs/p.md", &page_covering("src"));
    repo.commit("init");
    repo.write("src/a.rs", "fn a() { 1 }\n");
    repo.commit("change");
    assert_eq!(repo.state("docs/p.md"), "stale");
    repo.git(&["mv", "docs/p.md", "docs/q.md"]);
    repo.commit("move");
    assert_eq!(repo.state("docs/q.md"), "stale");
}

#[test]
fn what_a_merge_itself_changed_counts() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn a() {}\n");
    repo.write("other.txt", "one\n");
    repo.write("docs/p.md", &page_covering("src"));
    repo.commit("init");
    assert_eq!(code(&repo.loam(&["review", "docs/p.md"])), 0);
    repo.commit("review");
    repo.git(&["checkout", "-q", "-b", "side"]);
    repo.write("other.txt", "side\n");
    repo.commit("side");
    repo.git(&["checkout", "-q", "main"]);
    repo.write("other.txt", "main\n");
    repo.commit("main");
    let _ = Command::new("git")
        .args(["merge", "-q", "side"])
        .current_dir(repo.dir.path())
        .output();
    repo.write("other.txt", "resolved\n");
    repo.write("src/a.rs", "fn a() { rewritten() }\n");
    repo.commit("merge, rewriting src/a.rs as well");
    assert_eq!(repo.state("docs/p.md"), "stale");
}

#[test]
fn since_counts_from_where_the_branches_parted() {
    let repo = Repo::new();
    repo.write("src/a.rs", "a\n");
    repo.write("src/b.rs", "b\n");
    repo.write("docs/a.md", &page_covering("src/a.rs"));
    repo.write("docs/b.md", &page_covering("src/b.rs"));
    repo.commit("init");
    repo.git(&["checkout", "-q", "-b", "feat"]);
    repo.write("src/a.rs", "a2\n");
    repo.commit("feat");
    repo.git(&["checkout", "-q", "main"]);
    repo.write("src/b.rs", "b2\n");
    repo.commit("main moved");
    repo.git(&["checkout", "-q", "feat"]);
    let t = text(&repo.loam(&["stale", "--since", "main"]));
    assert!(t.contains("docs/a.md"), "{t}");
    assert!(
        !t.contains("docs/b.md"),
        "main's own change is not this branch's: {t}"
    );
}

#[test]
fn which_page_was_edited_decides_which_is_being_updated() {
    for edited in ["a", "b"] {
        let repo = Repo::new();
        repo.write("src/x.rs", "x\n");
        repo.write("docs/a.md", &page_covering("src"));
        repo.write("docs/b.md", &page_covering("src"));
        repo.commit("init");
        repo.write("src/x.rs", "y\n");
        repo.write(
            &format!("docs/{edited}.md"),
            &page_covering("src").replace("Body", "Updated"),
        );
        repo.commit("code and one page");
        let other = if edited == "a" { "b" } else { "a" };
        assert_eq!(
            repo.state(&format!("docs/{edited}.md")),
            "fresh",
            "{edited}"
        );
        repo.write("src/x.rs", "z\n");
        repo.write(
            &format!("docs/{edited}.md"),
            &page_covering("src").replace("Body", "Updated again"),
        );
        repo.commit("code and the same page");
        assert_eq!(
            repo.state(&format!("docs/{edited}.md")),
            "fresh",
            "{edited}"
        );
        assert_eq!(repo.state(&format!("docs/{other}.md")), "stale", "{other}");
    }
}

#[test]
fn a_small_budget_is_kept_and_still_names_what_was_left_out() {
    let repo = Repo::new();
    repo.write("src/a.rs", "a\n");
    for name in ["one", "two", "three"] {
        repo.write(&format!("docs/{name}.md"), &page_covering("src"));
    }
    repo.commit("init");
    for budget in [200, 230, 260, 300, 500] {
        let out = repo.loam(&["context", "src/a.rs", "--budget", &budget.to_string()]);
        assert_eq!(code(&out), 0);
        assert!(out.stdout.len() <= budget, "{budget}: {}", out.stdout.len());
        let t = String::from_utf8_lossy(&out.stdout);
        let shown = t.matches("\n## ").count();
        assert!(
            shown == 3 || t.contains("left out, over the budget"),
            "{budget}: {t}"
        );
    }
    assert_eq!(
        code(&repo.loam(&["context", "src/a.rs", "--budget", "60"])),
        2
    );
}

#[test]
fn an_empty_repository_has_nothing_stale() {
    let repo = Repo::new();
    repo.write("docs/p.md", &page_covering("src"));
    let out = repo.loam(&["stale"]);
    assert_eq!(code(&out), 0, "{}", text(&out));
    assert!(text(&out).contains("no commits yet"), "{}", text(&out));
    assert_eq!(code(&repo.loam(&["check", "--stale"])), 0);
}

#[test]
fn judging_a_commit_before_the_review_sets_the_review_aside() {
    let repo = Repo::new();
    repo.write("src/a.rs", "a\n");
    repo.write("docs/p.md", &page_covering("src"));
    repo.commit("init");
    repo.write("src/a.rs", "b\n");
    let before = repo.commit("change");
    repo.write("README", "later\n");
    repo.commit("later");
    assert_eq!(code(&repo.loam(&["review", "docs/p.md"])), 0);
    repo.commit("review");
    let t = text(&repo.loam(&["stale", "--at", &before]));
    assert!(t.contains("reviewed after the commit being judged"), "{t}");
    assert!(!t.contains("does not have"), "{t}");
    assert!(t.contains("1 stale"), "{t}");
}

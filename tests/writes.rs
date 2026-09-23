// loam — what a write must never do, each case found by a reviewing agent
// breaking it on purpose.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use std::path::PathBuf;
use std::process::{Command, Output};

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new(files: &[(&str, &str)]) -> Repo {
        let repo = Repo {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.write("loam.toml", "format = 1\n\n[[kind]]\nname = \"page\"\n");
        for (p, t) in files {
            repo.write(p, t);
        }
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
    fn loam(&self, args: &[&str]) -> Output {
        self.loam_in(".", args)
    }
    fn loam_in(&self, sub: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_loam"))
            .args(args)
            .current_dir(self.dir.path().join(sub))
            .env("NO_COLOR", "1")
            .env_remove("LOAM_AGENT")
            .env_remove("CAIRN_AGENT")
            .env_remove("AI_AGENT")
            .env_remove("CLAUDECODE")
            .output()
            .unwrap()
    }
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}
fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn agent_write_never_erases_a_file_it_cannot_read() {
    let repo = Repo::new(&[]);
    std::fs::write(
        repo.path("CLAUDE.md"),
        b"# Notes\n\nCaf\xe9 rules: never push on Friday.\n",
    )
    .unwrap();
    let out = repo.loam(&["agent", "--write", "CLAUDE.md"]);
    assert_eq!(code(&out), 2);
    assert!(text(&out).contains("not UTF-8"));
    assert_eq!(
        std::fs::read(repo.path("CLAUDE.md")).unwrap(),
        b"# Notes\n\nCaf\xe9 rules: never push on Friday.\n"
    );
}

#[test]
fn agent_write_uses_marker_lines_only_and_keeps_the_endings() {
    let mention = "# Agents\r\n\r\nThe block between `<!-- loam:begin -->` and `<!-- loam:end -->` is generated.\r\n\r\n```\r\n<!-- loam:begin -->\r\nan example\r\n<!-- loam:end -->\r\n```\r\n";
    let repo = Repo::new(&[("AGENTS.md", mention)]);
    assert_eq!(code(&repo.loam(&["agent", "--write", "AGENTS.md"])), 0);
    let after = repo.read("AGENTS.md");
    assert!(
        after.starts_with(mention),
        "the sentence and the example are untouched:\n{after}"
    );
    assert!(
        !after.replace("\r\n", "").contains('\n'),
        "every line still ends CRLF"
    );
    assert_eq!(after.matches("## Written context").count(), 1);
    // And a second run replaces the real block, not the example.
    assert_eq!(code(&repo.loam(&["agent", "--write", "AGENTS.md"])), 0);
    assert_eq!(repo.read("AGENTS.md"), after);
}

#[test]
fn render_writes_between_real_markers_not_an_example_of_them() {
    let readme = "# Docs\n\nPut these two lines where the index goes:\n\n```\n<!-- loam:index:begin -->\n<!-- loam:index:end -->\n```\n\n<!-- loam:index:begin -->\n<!-- loam:index:end -->\n";
    let repo = Repo::new(&[("docs/README.md", readme), ("docs/a.md", "# A\n")]);
    assert_eq!(code(&repo.loam(&["render"])), 0);
    let after = repo.read("docs/README.md");
    assert!(after.starts_with("# Docs\n\nPut these two lines where the index goes:\n\n```\n<!-- loam:index:begin -->\n<!-- loam:index:end -->\n```\n"), "{after}");
    assert!(after.contains("[A](a.md)"));
}

#[test]
fn set_refuses_to_rewrite_a_list_it_cannot_edit_safely() {
    let page = "---\nports: [80, 443]\nmeta:\n  owner: ann\nsources:\n  - title: Spec\n    read: 2026-01-01\n---\n\n# C\n";
    let repo = Repo::new(&[("docs/c.md", page)]);
    for arg in ["ports+=8080", "meta+=x", "sources-=nothing"] {
        let out = repo.loam(&["set", "docs/c.md", arg]);
        assert_eq!(code(&out), 2, "{arg}");
        assert!(
            text(&out).contains("edit it by hand"),
            "{arg}: {}",
            text(&out)
        );
    }
    assert_eq!(repo.read("docs/c.md"), page);
}

#[test]
fn two_writes_at_once_both_land() {
    let repo = Repo::new(&[("docs/a.md", "# A\n\ntext.\n")]);
    let bin = env!("CARGO_BIN_EXE_loam");
    let children: Vec<_> = (0..8)
        .map(|i| {
            Command::new(bin)
                .args(["set", "docs/a.md", &format!("k{i}=v{i}"), "--no-hooks"])
                .current_dir(repo.dir.path())
                .env_remove("CLAUDECODE")
                .spawn()
                .unwrap()
        })
        .collect();
    for mut c in children {
        assert!(c.wait().unwrap().success());
    }
    let after = repo.read("docs/a.md");
    for i in 0..8 {
        assert!(
            after.contains(&format!("k{i}: v{i}")),
            "k{i} was lost:\n{after}"
        );
    }
}

#[test]
fn two_news_at_once_write_one_page() {
    let repo = Repo::new(&[("docs/.keep", "")]);
    let bin = env!("CARGO_BIN_EXE_loam");
    let children: Vec<_> = (0..6)
        .map(|i| {
            Command::new(bin)
                .args([
                    "new",
                    "page",
                    "Same title",
                    "--anyway",
                    "--no-hooks",
                    "--draft",
                ])
                .current_dir(repo.dir.path())
                .env("MARK", i.to_string())
                .env_remove("CLAUDECODE")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap()
        })
        .collect();
    // All six started before any is waited for: they race.
    let written = children
        .into_iter()
        .map(|mut c| c.wait().unwrap())
        .filter(|s| s.success())
        .count();
    assert_eq!(
        written, 1,
        "exactly one wrote the page; the rest were refused"
    );
}

#[test]
fn mv_leaves_git_and_the_configuration_alone() {
    let repo = Repo::new(&[("docs/a.md", "# A\n")]);
    std::fs::create_dir_all(repo.path(".git")).unwrap();
    for (old, new) in [(".git", "gitx"), ("loam.toml", "proj")] {
        let out = repo.loam(&["mv", old, new]);
        assert_eq!(code(&out), 2, "{old}");
        assert!(repo.path(old).exists());
    }
}

#[test]
fn review_notes_go_in_the_review_log() {
    let page = "---\ncovers: src/a.rs\n---\n\n# A\n\n## Review log\n\n- 2026-01-01, at abcdef1: first\n\n## Appendix\n\nLast words.\n";
    let repo = Repo::new(&[("docs/a.md", page), ("src/a.rs", "fn a() {}\n")]);
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(repo.dir.path())
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@e")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@e")
            .output()
            .unwrap()
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&["commit", "-qm", "x"]);
    assert_eq!(
        code(&repo.loam(&["review", "docs/a.md", "--note", "second"])),
        0
    );
    let after = repo.read("docs/a.md");
    let log = after.find("## Review log").unwrap();
    let appendix = after.find("## Appendix").unwrap();
    let second = after.find(": second").unwrap();
    assert!(log < second && second < appendix, "{after}");
    assert!(after.ends_with("## Appendix\n\nLast words.\n"), "{after}");
    assert!(after.contains(": second\n\n## Appendix"), "{after}");
}

#[test]
fn commands_work_from_a_subdirectory() {
    let repo = Repo::new(&[("docs/a.md", "# A\n"), ("src/deep/x.rs", "")]);
    let out = repo.loam_in("src/deep", &["list"]);
    assert_eq!(code(&out), 0, "{}", text(&out));
    assert!(text(&out).contains("docs/a.md"));
}

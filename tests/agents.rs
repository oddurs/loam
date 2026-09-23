// loam — what an agent uses: the contract, drafts, the duplicate check,
// context, set, and what a change just made stale.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Repo {
    dir: tempfile::TempDir,
}

const CONFIG: &str = "format = 1\n\n[agents]\nstatus = \"draft\"\n\n[[kind]]\nname = \"research\"\ndir = \"research\"\ndescription = \"What did we find out? Evidence, with its sources.\"\n\n[[kind]]\nname = \"design\"\ndir = \"design\"\n\n[[kind]]\nname = \"page\"\n";

impl Repo {
    fn new(files: &[(&str, &str)]) -> Repo {
        let repo = Repo {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.write("loam.toml", CONFIG);
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
    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(self.dir.path())
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    fn commit(&self) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", "x"]);
    }
    /// As a person: no agent in the environment.
    fn loam(&self, args: &[&str]) -> Output {
        self.run(args, None)
    }
    fn as_agent(&self, args: &[&str]) -> Output {
        self.run(args, Some("tester"))
    }
    fn run(&self, args: &[&str], agent: Option<&str>) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_loam"));
        c.args(args)
            .current_dir(self.dir.path())
            .env("NO_COLOR", "1")
            .env_remove("LOAM_AGENT")
            .env_remove("CAIRN_AGENT")
            .env_remove("AI_AGENT")
            .env_remove("CLAUDECODE");
        if let Some(a) = agent {
            c.env("LOAM_AGENT", a);
        }
        c.output().unwrap()
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
fn json(o: &Output) -> Value {
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(o)))
}

// ─── loam set (0067) ─────────────────────────────────────────────────────────

#[test]
fn set_sets_replaces_and_removes_a_key_and_nothing_else() {
    let page = "---\r\nlayout: wide\r\nstatus: current\r\n---\r\n\r\n# A\r\n";
    let repo = Repo::new(&[("docs/a.md", page)]);
    assert_eq!(
        code(&repo.loam(&["set", "docs/a.md", "status=draft", "summary=What a is for."])),
        0
    );
    assert_eq!(
        repo.read("docs/a.md"),
        "---\r\nlayout: wide\r\nstatus: draft\r\nsummary: What a is for.\r\n---\r\n\r\n# A\r\n"
    );
    assert_eq!(
        code(&repo.loam(&["set", "docs/a.md", "covers+=src/a.rs", "covers+=src/b"])),
        0
    );
    assert!(
        repo.read("docs/a.md")
            .contains("covers:\r\n  - src/a.rs\r\n  - src/b\r\n")
    );
    assert_eq!(
        code(&repo.loam(&["set", "docs/a.md", "covers-=src/a.rs"])),
        0
    );
    assert!(repo.read("docs/a.md").contains("covers:\r\n  - src/b\r\n"));
    assert_eq!(
        code(&repo.loam(&[
            "set",
            "docs/a.md",
            "summary=",
            "covers=",
            "weight=3",
            "order=2"
        ])),
        0
    );
    assert_eq!(
        repo.read("docs/a.md"),
        "---\r\nlayout: wide\r\nstatus: draft\r\nweight: 3\r\norder: 2\r\n---\r\n\r\n# A\r\n"
    );
    let again = repo.loam(&["set", "docs/a.md", "order=2"]);
    assert!(text(&again).contains("already says so"));
    // Values that would read back as something else are quoted.
    assert_eq!(code(&repo.loam(&["set", "docs/a.md", "title=no"])), 0);
    let show = json(&repo.loam(&["show", "docs/a.md", "--json"]));
    assert_eq!(show["title"], "no");
}

#[test]
fn set_refuses_a_value_of_the_wrong_type_for_a_key_of_the_format() {
    let repo = Repo::new(&[("docs/a.md", "# A\n")]);
    for (arg, says) in [
        ("status=done", "draft, current or superseded"),
        ("order=first", "whole number"),
        ("kind=nonsense", "no kind called"),
        ("reviewed=today", "loam review"),
        ("title+=x", "holds one value"),
        ("bad key=x", "is not a key"),
        ("nothing", "is not KEY=VALUE"),
    ] {
        let out = repo.loam(&["set", "docs/a.md", arg]);
        assert_eq!(code(&out), 2, "{arg}");
        assert!(text(&out).contains(says), "{arg}: {}", text(&out));
    }
    assert_eq!(repo.read("docs/a.md"), "# A\n", "nothing written");
}

// ─── Drafts (0047) ───────────────────────────────────────────────────────────

#[test]
fn pages_an_agent_writes_are_drafts_and_a_persons_are_not() {
    let repo = Repo::new(&[(
        "docs/README.md",
        "# Docs\n\n<!-- loam:index:begin -->\n<!-- loam:index:end -->\n",
    )]);
    let by_agent = repo.as_agent(&["new", "research", "Markdown crates"]);
    assert_eq!(code(&by_agent), 0, "{}", text(&by_agent));
    assert!(
        repo.read("docs/research/markdown-crates.md")
            .starts_with("---\nstatus: draft\n---\n")
    );
    assert!(text(&by_agent).contains("a draft, because tester wrote it"));
    let by_person = repo.loam(&["new", "research", "Terminal widths"]);
    assert_eq!(code(&by_person), 0);
    assert!(
        repo.read("docs/research/terminal-widths.md")
            .starts_with("# Terminal widths")
    );
    // --agent names one too, and the index marks the draft.
    assert_eq!(
        code(&repo.loam(&["new", "design", "Layout", "--agent", "other"])),
        0
    );
    assert!(
        repo.read("docs/design/layout.md")
            .starts_with("---\nstatus: draft\n")
    );
    assert_eq!(code(&repo.loam(&["render"])), 0);
    assert!(
        repo.read("docs/README.md")
            .contains("[Markdown crates](research/markdown-crates.md) *(draft)*")
    );
    assert!(
        !repo
            .read("docs/README.md")
            .contains("[Terminal widths](research/terminal-widths.md) *(draft)*")
    );
}

#[test]
fn without_the_setting_agents_write_current_pages() {
    let repo = Repo::new(&[]);
    repo.write("loam.toml", "format = 1\n\n[[kind]]\nname = \"page\"\n");
    assert_eq!(code(&repo.as_agent(&["new", "page", "A"])), 0);
    assert_eq!(repo.read("docs/a.md"), "# A\n");
}

// ─── The duplicate check (0048) ──────────────────────────────────────────────

#[test]
fn a_second_page_on_the_same_thing_is_caught() {
    let repo = Repo::new(&[(
        "docs/research/markdown-crates.md",
        "# Markdown crates\n\nWhich Rust crates parse Markdown, and what each costs to embed.\n",
    )]);
    let out = repo.as_agent(&["new", "research", "Rust Markdown parsers"]);
    assert_eq!(code(&out), 1);
    let t = text(&out);
    assert!(t.contains("research already has 1 page(s) like it:"), "{t}");
    assert!(
        t.contains(
            "docs/research/markdown-crates.md  \"Markdown crates\"  (shares: markdown, parsers, rust)"
        ),
        "{t}"
    );
    assert!(!repo.path("docs/research/rust-markdown-parsers.md").exists());
    // A person without a terminal to answer on is refused the same way.
    assert_eq!(
        code(&repo.loam(&["new", "research", "Rust Markdown parsers"])),
        1
    );
    // --anyway writes it.
    assert_eq!(
        code(&repo.as_agent(&["new", "research", "Rust Markdown parsers", "--anyway"])),
        0
    );
    assert!(repo.path("docs/research/rust-markdown-parsers.md").exists());
    // Another kind is another question.
    assert_eq!(
        code(&repo.as_agent(&["new", "design", "Rust Markdown parsers"])),
        0
    );
}

// ─── The contract (0046) ─────────────────────────────────────────────────────

#[test]
fn the_contract_is_generated_from_the_config() {
    let repo = Repo::new(&[]);
    let block = String::from_utf8(repo.loam(&["agent"]).stdout).unwrap();
    assert!(block.starts_with("<!-- loam:begin -->\n## Written context\n"));
    assert!(block.ends_with("<!-- loam:end -->\n"));
    assert!(block.contains("- **`research`** in `docs/research/` — What did we find out? Evidence, with its sources."), "{block}");
    assert!(block.contains("starts as `status: draft`"));
    assert!(block.contains("loam search"));
    assert!(block.contains("loam supersede"));
    assert!(block.contains("loam stale --working-tree"));
    // A description written over several lines stays one list item.
    repo.write(
        "loam.toml",
        &CONFIG.replace(
            "description = \"What did we find out? Evidence, with its sources.\"",
            "description = \"\"\"\nWhat did we find out?\nEvidence.\n\"\"\"",
        ),
    );
    let wrapped = String::from_utf8(repo.loam(&["agent"]).stdout).unwrap();
    assert!(
        wrapped.contains("in `docs/research/` — What did we find out? Evidence.\n"),
        "{wrapped}"
    );
    assert!(
        wrapped.contains("sources:\n     - title:"),
        "the shape of sources is given"
    );
    // Changing a kind changes the block.
    repo.write(
        "loam.toml",
        &CONFIG.replace(
            "name = \"design\"\ndir = \"design\"",
            "name = \"decision\"\ndir = \"decisions\"",
        ),
    );
    let changed = String::from_utf8(repo.loam(&["agent"]).stdout).unwrap();
    assert!(changed.contains("**`decision`** in `docs/decisions/`"));
    assert!(!changed.contains("**`design`**"));
}

#[test]
fn write_is_idempotent_and_leaves_the_rest_of_the_file_alone() {
    let repo = Repo::new(&[(
        "AGENTS.md",
        "# Agents\n\nHand-written.\n\n<!-- cairn:begin -->\ncairn's\n<!-- cairn:end -->\n",
    )]);
    assert_eq!(code(&repo.loam(&["agent", "--write", "AGENTS.md"])), 0);
    let once = repo.read("AGENTS.md");
    assert!(once.starts_with("# Agents\n\nHand-written.\n\n<!-- cairn:begin -->\ncairn's\n<!-- cairn:end -->\n\n<!-- loam:begin -->"));
    let out = repo.loam(&["agent", "--write", "AGENTS.md"]);
    assert!(text(&out).contains("already current"));
    assert_eq!(repo.read("AGENTS.md"), once);
    // Edited around, and the block replaced in place.
    let edited = once.replace("Hand-written.", "Hand-written, and more.") + "\nAfter.\n";
    repo.write(
        "AGENTS.md",
        &edited.replace("### Commands", "### Commands, stale"),
    );
    assert_eq!(code(&repo.loam(&["agent", "--write", "AGENTS.md"])), 0);
    let now = repo.read("AGENTS.md");
    assert!(
        now.contains("Hand-written, and more.") && now.ends_with("<!-- loam:end -->\n\nAfter.\n"),
        "{now}"
    );
    assert!(!now.contains("### Commands, stale"));
}

// ─── Context (0049) ──────────────────────────────────────────────────────────

fn context_repo() -> Repo {
    let long = "It argues against splitting the queue. ".repeat(100);
    Repo::new(&[
        ("src/engine/scheduler.rs", "fn schedule() {}\n"),
        (
            "docs/design/scheduling.md",
            &format!(
                "---\ncovers: src/engine\n---\n\n# Scheduling\n\nOne queue, on purpose.\n\nSee [the queue](queue.md).\n\n{long}\n"
            ),
        ),
        (
            "docs/design/scheduler-file.md",
            "---\ncovers: src/engine/scheduler.rs\n---\n\n# The scheduler file\n\nWhat is in it.\n",
        ),
        ("docs/design/queue.md", "# The queue\n\nWhy there is one.\n"),
        (
            "docs/notes.md",
            "# Notes\n\nSee src/engine/scheduler.rs for the details.\n",
        ),
        (
            "docs/unrelated.md",
            "# Unrelated\n\nNothing to do with it.\n",
        ),
    ])
}

#[test]
fn context_returns_the_covering_pages_first() {
    let repo = context_repo();
    let v = json(&repo.loam(&[
        "context",
        "src/engine/scheduler.rs",
        "--json",
        "--budget",
        "64k",
    ]));
    let order: Vec<&str> = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        order,
        [
            "docs/design/scheduler-file.md",
            "docs/design/scheduling.md",
            "docs/notes.md",
            "docs/design/queue.md"
        ],
        "the most specific cover, then the other cover, then a mention, then a link"
    );
    assert_eq!(v["pages"][2]["why"], "mentions src/engine/scheduler.rs");
    assert_eq!(
        v["pages"][3]["why"],
        "linked from docs/design/scheduling.md"
    );
    assert!(
        v["pages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["included"] == "full")
    );
}

#[test]
fn context_never_exceeds_its_budget_and_says_what_it_left_out() {
    let repo = context_repo();
    for budget in ["300", "600", "1k", "5000"] {
        let out = repo.loam(&["context", "src/engine/scheduler.rs", "--budget", budget]);
        assert_eq!(code(&out), 0);
        let limit = if budget == "1k" {
            1024
        } else {
            budget.parse().unwrap()
        };
        assert!(
            out.stdout.len() <= limit,
            "{budget}: {} bytes",
            out.stdout.len()
        );
    }
    let out = String::from_utf8(
        repo.loam(&["context", "src/engine/scheduler.rs", "--budget", "1k"])
            .stdout,
    )
    .unwrap();
    assert!(
        out.contains("## docs/design/scheduler-file.md — The scheduler file"),
        "{out}"
    );
    assert!(
        out.contains("One queue, on purpose."),
        "a page too long for the budget is given by its summary: {out}"
    );
    assert!(!out.contains("It argues against splitting"), "{out}");
    let tight = String::from_utf8(
        repo.loam(&["context", "src/engine/scheduler.rs", "--budget", "400"])
            .stdout,
    )
    .unwrap();
    assert!(tight.contains("left out, over the budget"), "{tight}");
    assert_eq!(
        code(&repo.loam(&["context", "src/engine/scheduler.rs", "--budget", "lots"])),
        2
    );
}

#[test]
fn context_flags_a_stale_page_rather_than_leaving_it_out() {
    let repo = context_repo();
    repo.git(&["init", "-q", "-b", "main"]);
    repo.commit();
    repo.write("src/engine/scheduler.rs", "fn schedule() { split() }\n");
    repo.commit();
    let out = String::from_utf8(
        repo.loam(&["context", "src/engine/scheduler.rs", "--budget", "64k"])
            .stdout,
    )
    .unwrap();
    assert!(
        out.contains(
            "(covers src/engine; STALE: 1 commit(s) changed what it covers since it was last read"
        ),
        "{out}"
    );
    let none = String::from_utf8(repo.loam(&["context", "src/other.rs"]).stdout).unwrap();
    assert!(none.contains("No page covers"), "{none}");
}

// ─── What a change just made stale (0050) ────────────────────────────────────

#[test]
fn stale_working_tree_and_since() {
    let repo = context_repo();
    repo.git(&["init", "-q", "-b", "main"]);
    repo.commit();
    let clean = repo.loam(&["stale", "--working-tree"]);
    assert_eq!(code(&clean), 0);
    assert!(text(&clean).contains("no page covers what changed"));

    repo.write("src/engine/scheduler.rs", "fn schedule() { 1 }\n");
    let out = repo.loam(&["stale", "--working-tree"]);
    assert_eq!(code(&out), 1);
    let t = text(&out);
    assert!(t.contains("docs/design/scheduling.md   covers src/engine/scheduler.rs, changed and not yet committed"), "{t}");
    assert!(t.contains("update it, or if it is still true: loam review docs/design/scheduling.md"));
    assert!(
        t.contains("2 page(s) to reread, 0 changed alongside"),
        "{t}"
    );

    // A page changed with it is not to reread.
    repo.write(
        "docs/design/scheduler-file.md",
        &repo
            .read("docs/design/scheduler-file.md")
            .replace("What is in it.", "What is in it now."),
    );
    let v = json(&repo.loam(&["stale", "--working-tree", "--json"]));
    let states: Vec<(&str, &str)> = v["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["path"].as_str().unwrap(), p["state"].as_str().unwrap()))
        .collect();
    assert!(states.contains(&("docs/design/scheduler-file.md", "updating")));
    assert!(states.contains(&("docs/design/scheduling.md", "reread")));

    repo.git(&["checkout", "-q", "--", "docs"]);
    repo.git(&["tag", "before"]);
    repo.commit();
    let since = repo.loam(&["stale", "--since", "before"]);
    assert_eq!(code(&since), 1);
    assert!(
        text(&since).contains("changed since then"),
        "{}",
        text(&since)
    );
    assert_eq!(code(&repo.loam(&["stale", "--since", "HEAD"])), 0);
    assert_eq!(code(&repo.loam(&["stale", "--since", "nonsense"])), 2);
}

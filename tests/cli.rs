// loam — the commands, driven through the binary as a person or an agent would.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    fn new(files: &[(&str, &str)]) -> Repo {
        let dir = tempfile::tempdir().expect("temp dir");
        let repo = Repo { dir };
        for (path, text) in files {
            repo.write(path, text);
        }
        repo
    }

    fn with_config(files: &[(&str, &str)]) -> Repo {
        let repo = Repo::new(files);
        if !repo.path("loam.toml").exists() {
            repo.write(
                "loam.toml",
                "format = 1\n\n[[kind]]\nname = \"guide\"\ndir = \"guide\"\n\n[[kind]]\nname = \"design\"\ndir = \"design\"\n\n[[kind]]\nname = \"page\"\n",
            );
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
        std::fs::read_to_string(self.path(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    fn loam(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_loam"))
            .args(args)
            .current_dir(self.dir.path())
            .env("NO_COLOR", "1")
            // A person, unless a test says otherwise: these tests run inside agents.
            .env_remove("LOAM_AGENT")
            .env_remove("CAIRN_AGENT")
            .env_remove("AI_AGENT")
            .env_remove("CLAUDECODE")
            .output()
            .unwrap()
    }

    /// Every file under the repository, with its bytes, for before-and-after.
    fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
            for e in std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
                let p = e.path();
                if p.is_dir() {
                    walk(root, &p, out);
                } else {
                    let rel = p
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.push((rel, std::fs::read(&p).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(self.dir.path(), self.dir.path(), &mut out);
        out.sort();
        out
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}
fn code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

fn json(o: &Output) -> serde_json::Value {
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {}", stdout(o)))
}

// ─── init (0029) ─────────────────────────────────────────────────────────────

#[test]
fn init_adopts_a_folder_without_changing_a_page() {
    let repo = Repo::new(&[
        ("docs/README.md", "# Docs\n\nThe index.\n"),
        ("docs/guide/first.md", "# First\n\nStart here.\n"),
        ("docs/decisions/0001-x.md", "# X\n"),
        ("docs/concept.md", "# Concept\n"),
        ("cairn.toml", "[project]\ndir = \"backlog\"\n"),
    ]);
    let before = repo.snapshot();
    let out = repo.loam(&["init"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let after: Vec<_> = repo
        .snapshot()
        .into_iter()
        .filter(|(p, _)| p != "loam.toml")
        .collect();
    assert_eq!(before, after, "init writes loam.toml and nothing else");
    let text = stdout(&out);
    assert!(
        text.contains("docs/guide/first.md")
            && text.contains("guide")
            && text.contains("\"First\""),
        "{text}"
    );
    assert!(
        text.contains("docs/decisions/0001-x.md") && text.contains("decision"),
        "{text}"
    );
    assert!(text.contains("no page was changed"));
    let config = repo.read("loam.toml");
    assert!(config.contains("cairn = \"backlog\""), "{config}");
    assert!(
        config.contains("# after-change"),
        "no markers yet, so the hook waits: {config}"
    );
    // And what it wrote reads cleanly.
    let check = repo.loam(&["check"]);
    assert_eq!(code(&check), 0, "{}", stderr(&check));
}

#[test]
fn init_refuses_to_overwrite_without_force() {
    let repo = Repo::new(&[
        ("docs/a.md", "# A\n"),
        ("loam.toml", "format = 1\n# mine\n"),
    ]);
    let out = repo.loam(&["init"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("--force"));
    assert_eq!(repo.read("loam.toml"), "format = 1\n# mine\n");
    assert_eq!(code(&repo.loam(&["init", "--force"])), 0);
    assert!(repo.read("loam.toml").contains("[[kind]]"));
}

#[test]
fn init_presets() {
    for (preset, kinds) in [("diataxis", 4), ("standard", 5), ("minimal", 1)] {
        let repo = Repo::new(&[("docs/a.md", "# A\n")]);
        let out = repo.loam(&["init", "--preset", preset]);
        assert_eq!(code(&out), 0, "{}", stderr(&out));
        assert_eq!(
            repo.read("loam.toml").matches("[[kind]]").count(),
            kinds,
            "{preset}"
        );
    }
    let repo = Repo::new(&[("docs/a.md", "# A\n")]);
    assert_eq!(code(&repo.loam(&["init", "--preset", "nonsense"])), 2);
}

// ─── reading (0028) ──────────────────────────────────────────────────────────

#[test]
fn a_broken_page_is_reported_and_the_rest_still_read() {
    let repo = Repo::with_config(&[
        ("docs/broken.md", "---\ntitle: never closed\n\n# Broken\n"),
        ("docs/latin1.md", ""),
        ("docs/fine.md", "# Fine\n\nStill read.\n"),
    ]);
    std::fs::write(repo.path("docs/latin1.md"), b"# Caf\xe9\n").unwrap();
    let out = repo.loam(&["list", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let pages = json(&out);
    assert_eq!(pages.as_array().unwrap().len(), 3);
    let check = repo.loam(&["check"]);
    assert_eq!(code(&check), 1);
    let err = stderr(&check);
    assert!(
        err.contains("docs/broken.md:1: frontmatter is no closing delimiter"),
        "{err}"
    );
    assert!(err.contains("docs/latin1.md:1: not UTF-8"), "{err}");
}

#[test]
fn reading_changes_no_byte() {
    let repo = Repo::with_config(&[
        (
            "docs/crlf.md",
            "---\r\nlayout: post\r\n---\r\n\r\n# CRLF\r\n\r\n[link](other.md)\r\n",
        ),
        ("docs/bom.md", "\u{feff}# BOM\n"),
        ("docs/other.md", "# Other\n"),
    ]);
    let before = repo.snapshot();
    for args in [
        &["check"][..],
        &["list"],
        &["show", "docs/crlf.md"],
        &["search", "link"],
        &["render", "--check"],
    ] {
        repo.loam(args);
    }
    assert_eq!(repo.snapshot(), before);
}

// ─── check (0031) ────────────────────────────────────────────────────────────

#[test]
fn check_exit_codes() {
    let clean = Repo::with_config(&[("docs/a.md", "# A\n")]);
    let out = clean.loam(&["check"]);
    assert_eq!(code(&out), 0);
    assert!(stdout(&out).contains("ok: 1 page(s), 0 warning(s)"));
    assert!(clean.loam(&["check", "--quiet"]).stdout.is_empty());

    // Warnings pass, unless --strict.
    let warned = Repo::with_config(&[("docs/a.md", "# A\n\n[gone](gone.md)\n")]);
    assert_eq!(code(&warned.loam(&["check"])), 0);
    let strict = warned.loam(&["check", "--strict"]);
    assert_eq!(code(&strict), 1);
    assert!(
        stderr(&strict)
            .contains("loam: docs/a.md:3: link to `gone.md`, which does not exist [broken-link]")
    );

    // Errors fail without it.
    let broken = Repo::with_config(&[("docs/a.md", "---\n- a list\n---\n# A\n")]);
    assert_eq!(code(&broken.loam(&["check"])), 1);

    // A project loam cannot read at all.
    let refused = Repo::new(&[("loam.toml", "format = 99\n"), ("docs/a.md", "# A\n")]);
    let out = refused.loam(&["check"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("format 99") && stderr(&out).contains("format 1"));
    let nowhere = Repo::new(&[]);
    assert_eq!(code(&nowhere.loam(&["check"])), 2);
}

#[test]
fn check_reports_each_finding_in_json() {
    let repo = Repo::with_config(&[
        (
            "loam.toml",
            "format = 1\n\n[[kind]]\nname = \"guide\"\ndir = \"guide\"\n\n[check.severity]\nunclaimed = \"error\"\nuntitled = \"ignore\"\n",
        ),
        (
            "docs/guide/a.md",
            "# A\n\n[anchor](b.md#nowhere) [page](missing.md)\n",
        ),
        ("docs/guide/b.md", "---\nstatus: superseded\n---\n\n# B\n"),
        (
            "docs/guide/c.md",
            "---\nsuperseded_by: a.md\nstatus: current\n---\n\n# C\n\n[old](b.md)\n",
        ),
        ("docs/guide/d.md", "No title.\n"),
        ("docs/loose.md", "# Loose\n"),
    ]);
    let out = repo.loam(&["check", "--json"]);
    assert_eq!(code(&out), 1, "unclaimed is an error here");
    let found = json(&out);
    let codes: Vec<(String, String, String)> = found
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["path"].as_str().unwrap().into(),
                f["code"].as_str().unwrap().into(),
                f["severity"].as_str().unwrap().into(),
            )
        })
        .collect();
    let has = |p: &str, c: &str| codes.iter().any(|(pp, cc, _)| pp == p && cc == c);
    assert!(has("docs/guide/a.md", "broken-anchor"));
    assert!(has("docs/guide/a.md", "broken-link"));
    assert!(has("docs/guide/b.md", "superseded-without-successor"));
    assert!(has("docs/guide/c.md", "successor-without-superseded"));
    assert!(has("docs/guide/c.md", "link-to-superseded"));
    assert!(has("docs/guide/c.md", "one-sided-supersession"));
    assert!(
        !has("docs/guide/d.md", "untitled"),
        "ignored by the project"
    );
    assert!(
        codes
            .iter()
            .any(|(p, c, s)| p == "docs/loose.md" && c == "unclaimed" && s == "error")
    );
    for f in found.as_array().unwrap() {
        assert!(f["line"].as_u64().is_some() && f["message"].as_str().is_some());
    }
}

// ─── render (0030) ───────────────────────────────────────────────────────────

const MARKED: &str = "# Docs\n\nWritten by hand.\n\n<!-- loam:index:begin -->\n<!-- loam:index:end -->\n\n## Elsewhere\n\nAlso by hand.\n";

#[test]
fn render_writes_between_the_markers_and_nowhere_else() {
    let repo = Repo::with_config(&[
        (
            "loam.toml",
            "format = 1\n\n[[kind]]\nname = \"guide\"\ndir = \"guide\"\ndescription = \"How to do things.\"\n\n[[kind]]\nname = \"page\"\n",
        ),
        ("docs/README.md", MARKED),
        (
            "docs/guide/b.md",
            "---\norder: 1\nsummary: \"Second by title, first by order | with a pipe.\"\n---\n\n# B\n",
        ),
        (
            "docs/guide/a.md",
            "---\nstatus: draft\n---\n\n# A\n\nThe summary.\n",
        ),
        (
            "docs/old.md",
            "---\nsuperseded_by: guide/a.md\n---\n\n# Old\n",
        ),
    ]);
    let check = repo.loam(&["render", "--check"]);
    assert_eq!(code(&check), 1, "drift before rendering");
    assert_eq!(code(&repo.loam(&["render"])), 0);
    let text = repo.read("docs/README.md");
    assert!(
        text.starts_with("# Docs\n\nWritten by hand.\n\n<!-- loam:index:begin -->\n"),
        "{text}"
    );
    assert!(
        text.ends_with("<!-- loam:index:end -->\n\n## Elsewhere\n\nAlso by hand.\n"),
        "{text}"
    );
    let b = text.find("[B](guide/b.md)").expect("b listed");
    let a = text
        .find("[A](guide/a.md) *(draft)*")
        .expect("a listed, marked draft");
    assert!(b < a, "order before title:\n{text}");
    assert!(text.contains("How to do things."));
    assert!(
        text.contains("\\| with a pipe."),
        "a pipe in a cell is escaped"
    );
    assert!(
        text.contains("## Superseded") && text.contains("replaced by [A](guide/a.md)"),
        "{text}"
    );
    assert!(!text.contains("[Docs]"), "the index does not list itself");

    // Rendering twice changes nothing, and --check now passes.
    let once = repo.snapshot();
    assert_eq!(code(&repo.loam(&["render"])), 0);
    assert_eq!(repo.snapshot(), once);
    assert_eq!(code(&repo.loam(&["render", "--check"])), 0);
    assert_eq!(code(&repo.loam(&["check", "--render"])), 0);

    // Drift is caught by check --render too.
    repo.write("docs/guide/c.md", "# C\n");
    let out = repo.loam(&["check", "--render"]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("[stale-index]"));
}

#[test]
fn render_rewrites_a_summary_s_links_for_where_the_index_is() {
    let repo = Repo::with_config(&[
        ("docs/README.md", MARKED),
        (
            "docs/guide/a.md",
            "# A\n\nSee [why](../design/why.md#costs) and [b](b.md).\n",
        ),
        ("docs/guide/b.md", "# B\n"),
        ("docs/design/why.md", "# Why\n\n## Costs\n"),
    ]);
    assert_eq!(code(&repo.loam(&["render"])), 0);
    let text = repo.read("docs/README.md");
    assert!(
        text.contains("See [why](design/why.md#costs) and [b](guide/b.md)."),
        "{text}"
    );
    assert_eq!(code(&repo.loam(&["check", "--strict", "--render"])), 0);
}

#[test]
fn render_keeps_crlf_and_refuses_a_file_without_markers() {
    let repo = Repo::with_config(&[
        ("docs/README.md", &MARKED.replace('\n', "\r\n")),
        ("docs/a.md", "# A\n"),
    ]);
    assert_eq!(code(&repo.loam(&["render"])), 0);
    let text = repo.read("docs/README.md");
    assert!(
        !text.replace("\r\n", "").contains('\n'),
        "every line still ends CRLF"
    );
    assert!(text.contains("[A](a.md)"));

    let plain = Repo::with_config(&[("docs/README.md", "# Hand-kept\n"), ("docs/a.md", "# A\n")]);
    let out = plain.loam(&["render"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("<!-- loam:index:begin -->"));
    assert_eq!(plain.read("docs/README.md"), "# Hand-kept\n");

    let fresh = Repo::with_config(&[("docs/a.md", "# A\n")]);
    assert_eq!(code(&fresh.loam(&["render"])), 0);
    assert!(fresh.read("docs/README.md").contains("[A](a.md)"));
}

// ─── new (0032) ──────────────────────────────────────────────────────────────

#[test]
fn new_puts_each_kind_of_every_preset_where_it_lives() {
    for preset in ["diataxis", "standard", "minimal"] {
        let repo = Repo::new(&[("docs/.keep", "")]);
        assert_eq!(code(&repo.loam(&["init", "--preset", preset])), 0);
        let list = repo.loam(&["list", "--json"]);
        assert_eq!(json(&list).as_array().unwrap().len(), 0);
        for kind in ["guide", "reference", "design", "research", "page"] {
            let out = repo.loam(&[
                "new",
                kind,
                "Markdown rendering in a forty-column pane",
                "--no-hooks",
            ]);
            let known = repo
                .read("loam.toml")
                .contains(&format!("name = \"{kind}\""));
            if !known {
                assert_eq!(code(&out), 2, "{preset}/{kind}");
                continue;
            }
            assert_eq!(code(&out), 0, "{preset}/{kind}: {}", stderr(&out));
            let path = stdout(&out).trim().to_string();
            let dir = if kind == "page" {
                "docs".to_string()
            } else {
                format!("docs/{kind}")
            };
            assert_eq!(
                path,
                format!("{dir}/markdown-rendering-in-a-forty-column-pane.md")
            );
            let text = repo.read(&path);
            assert!(
                text.starts_with("# Markdown rendering in a forty-column pane\n"),
                "{text}"
            );
            if kind == "research" {
                assert!(text.contains("## Sources"), "the research template: {text}");
            }
            let show = json(&repo.loam(&["show", &path, "--json"]));
            assert_eq!(
                show["kind"], kind,
                "the page reads as the kind it was made as"
            );
        }
    }
}

#[test]
fn new_never_overwrites_and_says_what_is_there() {
    let repo = Repo::with_config(&[("docs/guide/first-run.md", "# The first run\n")]);
    let before = repo.snapshot();
    let out = repo.loam(&["new", "guide", "First run"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("docs/guide/first-run.md already exists (\"The first run\")"),
        "{}",
        stderr(&out)
    );
    assert_eq!(repo.snapshot(), before);

    let draft = repo.loam(&["new", "guide", "Second run", "--draft"]);
    assert_eq!(code(&draft), 0);
    assert!(
        repo.read("docs/guide/second-run.md")
            .starts_with("---\nstatus: draft\n---\n")
    );
    assert_eq!(code(&repo.loam(&["new", "nonsense", "X"])), 2);
}

// ─── list, show, search (0033) ───────────────────────────────────────────────

#[test]
fn list_show_and_search_speak_json() {
    let repo = Repo::with_config(&[
        ("docs/guide/colour.md", "# Colour\n\nPalettes and themes.\n"),
        (
            "docs/design/why.md",
            "---\nstatus: draft\n---\n\n# Why\n\nThe reason for the colour choices.\n",
        ),
        (
            "docs/notes.md",
            "# Notes\n\nNothing about it.\n\nExcept one line on colour, deep down.\n",
        ),
    ]);
    let list = json(&repo.loam(&["list", "--json", "--kind", "guide"]));
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["path"], "docs/guide/colour.md");
    for key in ["title", "kind", "status", "summary"] {
        assert!(list[0].get(key).is_some(), "{key}");
    }
    let drafts = json(&repo.loam(&["list", "--json", "--status", "draft"]));
    assert_eq!(drafts[0]["path"], "docs/design/why.md");

    let show = json(&repo.loam(&["show", "docs/design/why.md", "--json"]));
    assert_eq!(show["status"], "draft");
    assert!(show["body"].as_str().unwrap().contains("The reason"));
    // Paths as typed from inside the docs root work too.
    assert_eq!(code(&repo.loam(&["show", "design/why.md"])), 0);
    assert_eq!(code(&repo.loam(&["show", "nope.md"])), 2);

    let hits = json(&repo.loam(&["search", "colour", "--json"]));
    let ranks: Vec<(&str, &str)> = hits
        .as_array()
        .unwrap()
        .iter()
        .map(|h| (h["path"].as_str().unwrap(), h["rank"].as_str().unwrap()))
        .collect();
    assert_eq!(
        ranks,
        [
            ("docs/guide/colour.md", "title"),
            ("docs/design/why.md", "summary"),
            ("docs/notes.md", "body")
        ]
    );
    assert_eq!(hits[2]["line"], 5);
    let none = repo.loam(&["search", "zebra"]);
    assert_eq!(code(&none), 1);
}

#[test]
fn search_is_quick_on_a_real_folder() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/corpus/real");
    for case in ["poptop", "code-as-color", "measure-of-the-world"] {
        let start = std::time::Instant::now();
        let out = Command::new(env!("CARGO_BIN_EXE_loam"))
            .args(["search", "the", "--json"])
            .current_dir(corpus.join(case))
            .output()
            .unwrap();
        assert_eq!(code(&out), 0, "{case}: {}", stderr(&out));
        assert!(
            start.elapsed().as_millis() < 500,
            "{case} took {:?}",
            start.elapsed()
        );
    }
}

// ─── mv (0034) ───────────────────────────────────────────────────────────────

fn mv_repo() -> Repo {
    Repo::with_config(&[
        (
            "README.md",
            "See [the design](docs/design/why.md#costs) and\n[the folder](docs/design/).\n",
        ),
        (
            "docs/guide/start.md",
            "# Start\n\nRead [why](../design/why.md#costs), [again](/docs/design/why.md),\nand [what it\nreplaced](../design/why.md \"title\").\n",
        ),
        (
            "docs/design/why.md",
            "---\nsupersedes: old.md\n---\n\n# Why\n\n## Costs\n\nBack to [start](../guide/start.md#start) and [self](#costs).\n",
        ),
        (
            "docs/design/old.md",
            "---\nsuperseded_by: why.md\n---\n\n# Old\n",
        ),
        (
            "docs/unrelated.md",
            "# Unrelated\r\n\r\n[start](guide/start.md)\r\n",
        ),
    ])
}

#[test]
fn mv_dry_run_changes_nothing() {
    let repo = mv_repo();
    let before = repo.snapshot();
    let out = repo.loam(&[
        "mv",
        "docs/design/why.md",
        "docs/reference/why.md",
        "--dry-run",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("would move docs/design/why.md → docs/reference/why.md"),
        "{text}"
    );
    assert!(
        text.contains("docs/guide/start.md:3: ../design/why.md#costs → ../reference/why.md#costs"),
        "{text}"
    );
    assert!(
        text.contains("docs/guide/start.md:5: ../design/why.md → ../reference/why.md"),
        "a wrapped link: {text}"
    );
    assert!(
        text.contains("README.md:1: docs/design/why.md#costs → docs/reference/why.md#costs"),
        "{text}"
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn mv_rewrites_links_to_and_from_the_page() {
    let repo = mv_repo();
    let unrelated = std::fs::read(repo.path("docs/unrelated.md")).unwrap();
    let out = repo.loam(&["mv", "docs/design/why.md", "docs/reference/why.md"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(!repo.path("docs/design/why.md").exists());
    assert_eq!(
        repo.read("docs/guide/start.md"),
        "# Start\n\nRead [why](../reference/why.md#costs), [again](/docs/reference/why.md),\nand [what it\nreplaced](../reference/why.md \"title\").\n"
    );
    assert_eq!(
        repo.read("README.md"),
        "See [the design](docs/reference/why.md#costs) and\n[the folder](docs/design/).\n"
    );
    let why = repo.read("docs/reference/why.md");
    assert!(why.contains("supersedes: ../design/old.md\n"), "{why}");
    assert!(
        why.contains("[start](../guide/start.md#start) and [self](#costs)"),
        "unchanged: {why}"
    );
    assert_eq!(
        repo.read("docs/design/old.md"),
        "---\nsuperseded_by: ../reference/why.md\n---\n\n# Old\n"
    );
    assert_eq!(
        std::fs::read(repo.path("docs/unrelated.md")).unwrap(),
        unrelated,
        "untouched, byte for byte"
    );
    let check = repo.loam(&["check", "--strict"]);
    assert_eq!(code(&check), 0, "{}", stderr(&check));

    // And back again is where it started, except where a link written from the
    // page's new place still reads correctly from its old one, and is left as
    // it is rather than rewritten into another spelling of the same path.
    assert_eq!(
        code(&repo.loam(&["mv", "docs/reference/why.md", "docs/design/why.md"])),
        0
    );
    let why = repo.read("docs/design/why.md");
    assert!(why.contains("supersedes: ../design/old.md\n"), "{why}");
    let start = mv_repo().snapshot();
    let back = repo.snapshot();
    for ((p, a), (q, b)) in start.iter().zip(&back) {
        assert_eq!(p, q);
        if p != "docs/design/why.md" {
            assert_eq!(a, b, "{p}");
        }
    }
    assert_eq!(code(&repo.loam(&["check", "--strict"])), 0);
}

#[test]
fn mv_moves_a_directory_and_refuses_to_overwrite() {
    let repo = mv_repo();
    assert_eq!(code(&repo.loam(&["mv", "docs/design", "docs/why"])), 0);
    assert!(
        repo.read("docs/guide/start.md")
            .contains("(../why/why.md#costs)")
    );
    assert!(repo.read("README.md").contains("[the folder](docs/why/)"));
    assert!(repo.path("docs/why/old.md").exists());
    let out = repo.loam(&["mv", "docs/why/old.md", "docs/guide/start.md"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("already exists"));
    // Not loam's own configuration.
    let out = repo.loam(&["mv", "loam.toml", "elsewhere.toml"]);
    assert_eq!(code(&out), 2);
    assert!(repo.path("loam.toml").exists());
    // Into an existing directory, as mv(1) does.
    assert_eq!(
        code(&repo.loam(&["mv", "docs/why/old.md", "docs/guide"])),
        0
    );
    assert!(repo.path("docs/guide/old.md").exists());
    assert_eq!(code(&repo.loam(&["check", "--strict"])), 0);
}

#[cfg(unix)]
#[test]
fn an_interrupted_move_leaves_every_file_old_or_new() {
    use std::os::unix::fs::PermissionsExt;
    let repo = mv_repo();
    // A directory loam cannot write in: rewriting the page inside it fails
    // part way through the move.
    repo.write(
        "docs/locked/cites.md",
        "# Cites\n\n[why](../design/why.md)\n",
    );
    let locked = repo.path("docs/locked");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let before = repo.snapshot();
    let out = repo.loam(&["mv", "docs/design/why.md", "docs/reference/why.md"]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(code(&out), 2, "{}", stdout(&out));
    // What a completed move writes, from a copy that could complete it.
    let done = mv_repo();
    done.write(
        "docs/locked/cites.md",
        "# Cites\n\n[why](../design/why.md)\n",
    );
    assert_eq!(
        code(&done.loam(&["mv", "docs/design/why.md", "docs/reference/why.md"])),
        0
    );
    let new = done.snapshot();
    for (path, bytes) in repo.snapshot() {
        assert!(!path.contains(".tmp"), "a temporary file was left: {path}");
        let old = before.iter().find(|(p, _)| *p == path).map(|(_, b)| b);
        // The moved page is rewritten where it stands, then renamed.
        let there = path.replace("docs/design/why.md", "docs/reference/why.md");
        let finished = new.iter().find(|(p, _)| *p == there).map(|(_, b)| b);
        assert!(
            old == Some(&bytes) || finished == Some(&bytes),
            "{path} is neither as it was nor as it will be"
        );
    }
    assert!(
        repo.path("docs/design/why.md").exists(),
        "the page did not move"
    );
}

// ─── supersede (0035) ────────────────────────────────────────────────────────

#[test]
fn supersede_updates_both_pages_once() {
    let repo = Repo::with_config(&[
        (
            "docs/design/architecture.md",
            "# Architecture\n\nHow it was.\n",
        ),
        (
            "docs/design/architecture-v2.md",
            "---\nlayout: wide\n---\n\n# Architecture, again\n\nHow it is.\n",
        ),
        (
            "docs/guide/a.md",
            "# A\n\n[arch](../design/architecture.md)\n",
        ),
    ]);
    let out = repo.loam(&[
        "supersede",
        "docs/design/architecture.md",
        "docs/design/architecture-v2.md",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(
        repo.read("docs/design/architecture.md"),
        "---\nsuperseded_by: architecture-v2.md\nstatus: superseded\n---\n\n> **Superseded** by [Architecture, again](architecture-v2.md).\n\n# Architecture\n\nHow it was.\n"
    );
    assert_eq!(
        repo.read("docs/design/architecture-v2.md"),
        "---\nlayout: wide\nsupersedes: architecture.md\n---\n\n# Architecture, again\n\nHow it is.\n"
    );
    let once = repo.snapshot();
    let again = repo.loam(&[
        "supersede",
        "docs/design/architecture.md",
        "docs/design/architecture-v2.md",
    ]);
    assert_eq!(code(&again), 0);
    assert!(stdout(&again).contains("already superseded"));
    assert_eq!(repo.snapshot(), once, "the notice is added once");

    // The old page keeps its title and summary; check points at the stale link.
    let show = json(&repo.loam(&["show", "docs/design/architecture.md", "--json"]));
    assert_eq!(show["title"], "Architecture");
    assert_eq!(show["summary"], "How it was.");
    assert_eq!(show["status"], "superseded");
    let check = repo.loam(&["check"]);
    assert!(
        stderr(&check).contains(
            "docs/guide/a.md:3: link to `../design/architecture.md`, which is superseded"
        ),
        "{}",
        stderr(&check)
    );
}

// ─── The writer's musts (0062) ───────────────────────────────────────────────

#[test]
fn writes_keep_line_endings_a_bom_and_unknown_keys_in_order() {
    let repo = Repo::with_config(&[
        (
            "docs/old.md",
            "\u{feff}---\r\nzeta: 1\r\nalpha: [b, a]\r\nnested:\r\n  deep: true\r\n---\r\n\r\n# Old\r\n",
        ),
        ("docs/new.md", "# New\n"),
    ]);
    assert_eq!(
        code(&repo.loam(&["supersede", "docs/old.md", "docs/new.md"])),
        0
    );
    assert_eq!(
        repo.read("docs/old.md"),
        "\u{feff}---\r\nzeta: 1\r\nalpha: [b, a]\r\nnested:\r\n  deep: true\r\nsuperseded_by: new.md\r\nstatus: superseded\r\n---\r\n\r\n> **Superseded** by [New](new.md).\r\n\r\n# Old\r\n"
    );
}

#[test]
fn values_that_would_read_back_differently_are_quoted() {
    // A page named so that, unquoted, YAML would read its path as a boolean
    // under 1.1 or a number under 1.2.
    let repo = Repo::with_config(&[("docs/no.md", "# No\n"), ("docs/012.md", "# Twelve\n")]);
    assert_eq!(
        code(&repo.loam(&["supersede", "docs/no.md", "docs/012.md"])),
        0
    );
    let show = json(&repo.loam(&["show", "docs/012.md", "--json"]));
    assert_eq!(show["supersedes"][0], "no.md");
    let check = repo.loam(&["check", "--strict"]);
    assert_eq!(code(&check), 0, "{}", stderr(&check));
}

#[test]
fn hooks_run_after_a_change_and_not_with_no_hooks() {
    let repo = Repo::with_config(&[
        (
            "loam.toml",
            "format = 1\n\n[hooks]\nafter-change = \"touch hook-ran\"\n\n[[kind]]\nname = \"page\"\n",
        ),
        ("docs/a.md", "# A\n"),
    ]);
    assert_eq!(code(&repo.loam(&["new", "page", "B", "--no-hooks"])), 0);
    assert!(!repo.path("hook-ran").exists());
    assert_eq!(code(&repo.loam(&["new", "page", "C"])), 0);
    assert!(repo.path("hook-ran").exists());
}

#[test]
fn no_command_but_init_writes_loam_toml() {
    // Spec §7.1: a tool rewriting the configuration must not reorder the kinds.
    // The simplest way to keep that is to rewrite it never: only `init` writes
    // it, and only when asked to replace it.
    let config = "format = 1\n\n[[kind]]\nname = \"zeta\"\ndir = \"z\"\n\n[[kind]]\nname = \"alpha\"\ndir = \"a\"\n\n[[kind]]\nname = \"page\"\n";
    let repo = Repo::new(&[
        ("loam.toml", config),
        ("docs/README.md", MARKED),
        ("docs/a/one.md", "# One\n"),
    ]);
    for args in [
        &["new", "zeta", "Two"][..],
        &["render"],
        &["mv", "docs/a/one.md", "docs/z/one.md"],
        &["supersede", "docs/z/one.md", "docs/z/two.md"],
        &["render"],
        &["check", "--render"],
    ] {
        let out = repo.loam(args);
        assert_eq!(code(&out), 0, "{args:?}: {}", stderr(&out));
        assert_eq!(repo.read("loam.toml"), config, "{args:?} rewrote loam.toml");
    }
}

// ─── found by using loam on a docs folder it had never seen ────────────────

#[test]
fn a_summary_link_to_its_own_section_still_works_from_the_index() {
    let repo = Repo::with_config(&[
        (
            "docs/README.md",
            "# Docs\n\n<!-- loam:index:begin -->\n<!-- loam:index:end -->\n",
        ),
        (
            "docs/guide/p.md",
            "# P\n\nSee [below](#later) and [the other](q.md).\n\n## Later\n",
        ),
        ("docs/guide/q.md", "# Q\n"),
    ]);
    assert_eq!(code(&repo.loam(&["render"])), 0);
    let index = repo.read("docs/README.md");
    assert!(
        index.contains("[below](guide/p.md#later) and [the other](guide/q.md)"),
        "{index}"
    );
    assert_eq!(code(&repo.loam(&["check", "--strict"])), 0);
}

#[test]
fn search_takes_a_quoted_phrase_as_words_and_ranks_by_where_they_are() {
    let repo = Repo::with_config(&[
        ("docs/README.md", "# Docs\n\nDocker cache, Docker cache.\n"),
        (
            "docs/guide/docker.md",
            "# Using it in Docker\n\nMount a cache to keep builds fast.\n",
        ),
        (
            "docs/guide/zz.md",
            "# Other\n\nA cache, a cache, a cache; and once, Docker.\n",
        ),
    ]);
    let out = repo.loam(&["search", "docker cache"]);
    assert_eq!(code(&out), 0);
    let t = String::from_utf8_lossy(&out.stdout);
    let docker = t.find("docs/guide/docker.md").expect("found");
    let other = t.find("docs/guide/zz.md").expect("found");
    assert!(docker < other, "the page titled for it first: {t}");
    assert!(
        !t.contains("docs/README.md"),
        "the index repeats everything: {t}"
    );

    let out = repo.loam(&["search", "docker", "kubernetes", "cache"]);
    assert_eq!(code(&out), 1);
    let t = String::from_utf8_lossy(&out.stderr);
    assert!(
        t.contains("these have the most") && t.contains("docker.md"),
        "{t}"
    );
}

#[test]
fn a_link_to_a_file_git_ignores_is_not_called_broken() {
    let repo = Repo::with_config(&[
        (".gitignore", "/docs/reference/cli.md\n"),
        (
            "docs/guide/a.md",
            "# A\n\nSee [the CLI](../reference/cli.md#run) and [gone](gone.md).\n",
        ),
    ]);
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(repo.path(""))
        .output()
        .unwrap();
    assert!(git.status.success());
    let out = repo.loam(&["check"]);
    let t = stdout(&out) + &stderr(&out);
    assert!(t.contains("1 link(s) point at files git ignores"), "{t}");
    assert!(
        !t.contains("reference/cli.md`, which does not exist"),
        "{t}"
    );
    assert!(t.contains("gone.md`, which does not exist"), "{t}");
    repo.write(
        "loam.toml",
        &(repo.read("loam.toml") + "\n[check.severity]\nignored-link = \"warning\"\n"),
    );
    let out = repo.loam(&["check"]);
    let t = stdout(&out) + &stderr(&out);
    assert!(
        t.contains("a file git ignores: built rather than committed"),
        "{t}"
    );
}

#[test]
fn init_in_a_site_uses_its_home_page_as_the_index() {
    let repo = Repo::new(&[
        ("mkdocs.yml", "site_name: x\n"),
        ("docs/index.md", "# Home\n\nWelcome.\n"),
        ("docs/guides/a.md", "# A\n"),
    ]);
    assert_eq!(code(&repo.loam(&["init"])), 0);
    let config = repo.read("loam.toml");
    assert!(config.contains("path = \"index.md\""), "{config}");
    assert!(
        config.contains("# after-change"),
        "no hook until it has markers: {config}"
    );
    assert!(!repo.path("docs/README.md").exists(), "no second home page");
}

#[test]
fn init_lists_only_what_needs_attention_in_a_large_folder() {
    let mut files: Vec<(String, String)> = (0..50)
        .map(|i| (format!("docs/guides/p{i}.md"), format!("# P{i}\n")))
        .collect();
    files.push(("docs/guides/untitled.md".into(), "No heading.\n".into()));
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let repo = Repo::new(&refs);
    let out = stdout(&repo.loam(&["init"]));
    assert!(out.contains("guide  51 page(s)"), "{out}");
    assert!(out.contains("docs/guides/untitled.md"), "{out}");
    assert!(!out.contains("docs/guides/p7.md"), "{out}");
    assert!(out.contains("… and 50 more"), "{out}");
}

#[test]
fn a_title_that_only_adds_a_word_to_one_that_exists_is_caught() {
    let repo = Repo::with_config(&[
        ("docs/guide/index.md", "# Guides\n\nEvery guide.\n"),
        ("docs/guide/docker.md", "# Using uv in Docker\n"),
        ("docs/guide/jupyter.md", "# Using uv with Jupyter\n"),
        ("docs/guide/lambda.md", "# Using uv with AWS Lambda\n"),
    ]);
    let out = repo.loam(&["new", "guide", "Using uv in Docker containers"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let t = stderr(&out);
    assert!(
        t.contains("docs/guide/docker.md") && t.contains("(shares: docker)"),
        "{t}"
    );
    assert!(
        !t.contains("index.md"),
        "a section's landing page is not a duplicate: {t}"
    );
    assert_eq!(
        code(&repo.loam(&["new", "guide", "Using uv with Bazel"])),
        0
    );
}

#[test]
fn a_mistyped_page_or_kind_is_answered_with_what_was_meant() {
    let repo = Repo::with_config(&[
        ("docs/guide/cache.md", "# Cache\n"),
        ("docs/design/index.md", "# Design\n"),
    ]);
    let t = stderr(&repo.loam(&["show", "cache.md"]));
    assert!(t.contains("did you mean docs/guide/cache.md?"), "{t}");
    let t = stderr(&repo.loam(&["show", "docs/guide/cahce.md"]));
    assert!(t.contains("did you mean docs/guide/cache.md?"), "{t}");
    let out = repo.loam(&["list", "--kind", "guides"]);
    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("no kind called `guides`"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn mv_keeps_a_link_written_with_dot_slash_that_way() {
    let repo = Repo::with_config(&[
        (
            "docs/guide/a.md",
            "# A\n\nSee [b](./b.md) and [b again](b.md).\n",
        ),
        ("docs/guide/b.md", "# B\n"),
    ]);
    assert_eq!(
        code(&repo.loam(&["mv", "docs/guide/b.md", "docs/guide/c.md"])),
        0
    );
    assert!(
        repo.read("docs/guide/a.md")
            .contains("[b](./c.md) and [b again](c.md)"),
        "{}",
        repo.read("docs/guide/a.md")
    );
}

#[test]
fn set_says_when_a_covered_path_matches_nothing() {
    let repo = Repo::with_config(&[("docs/guide/a.md", "# A\n"), ("src/lib.rs", "")]);
    let out = repo.loam(&[
        "set",
        "docs/guide/a.md",
        "covers+=src/lib.rs",
        "covers+=src/nope",
    ]);
    assert_eq!(code(&out), 0);
    let t = stderr(&out);
    assert!(t.contains("`src/nope` matches no file"), "{t}");
    assert!(!t.contains("src/lib.rs` matches"), "{t}");
}

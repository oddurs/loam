// loam — loam.toml, the project's configuration.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Spec §7.1 names the keys that bear on reading pages: `format`, `docs.root`,
// `links.cairn`, and each kind's `name` and `dir`. Everything else here is the
// program's — descriptions, templates, the index, hooks — and a reader of the
// format may ignore all of it. Unknown keys are warned about, never refused.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "loam.toml";
pub const FORMAT: i64 = 1;

#[derive(Clone, Debug)]
pub struct Kind {
    pub name: String,
    /// Relative to the docs root, without slashes at either end; "" for the root.
    pub dir: String,
    /// The heading the index gives the kind. Defaults to the name, capitalised.
    pub title: Option<String>,
    pub description: Option<String>,
    pub template: Option<String>,
    /// Whether the index lists the kind's pages.
    pub index: bool,
    /// How long a page of this kind stays true without a review: `180d`.
    pub stale_after: Option<String>,
}

impl Kind {
    pub fn heading(&self) -> String {
        self.title.clone().unwrap_or_else(|| {
            let mut c = self.name.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    /// The repository: the directory holding loam.toml.
    pub root: PathBuf,
    /// Relative to the repository; "" when the docs root is the repository.
    pub docs: String,
    pub cairn: Option<String>,
    pub kinds: Vec<Kind>,
    /// The generated index, relative to the repository.
    pub index: String,
    /// Shell command run after a command changes pages.
    pub after_change: Option<String>,
    /// `[check.severity]`: a finding's code, and `error`, `warning` or `ignore`.
    pub severity: std::collections::HashMap<String, String>,
    pub warnings: Vec<String>,
}

fn clean(dir: &str) -> String {
    let d = dir.trim_matches('/');
    if d == "." {
        String::new()
    } else {
        d.to_string()
    }
}

impl Config {
    /// Find loam.toml in `start` or a directory above it.
    pub fn discover(start: &Path) -> Result<Config> {
        let mut dir = start.to_path_buf();
        loop {
            if dir.join(CONFIG_FILE).is_file() {
                return Config::load(&dir);
            }
            if !dir.pop() {
                bail!(
                    "no {CONFIG_FILE} found in {} or any parent directory\n\
                     run `loam init` to adopt the docs folder here",
                    start.display()
                );
            }
        }
    }

    pub fn load(root: &Path) -> Result<Config> {
        let path = root.join(CONFIG_FILE);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        Config::parse(root, &text)
    }

    pub fn parse(root: &Path, text: &str) -> Result<Config> {
        let doc: toml::Table = text
            .parse()
            .with_context(|| format!("{CONFIG_FILE} is not valid TOML"))?;

        // Spec §9: refuse, naming both versions. Never read on a best-effort basis.
        match doc.get("format") {
            Some(toml::Value::Integer(FORMAT)) => {}
            Some(other) => bail!(
                "{CONFIG_FILE} is format {other}, and this loam reads format {FORMAT}\n\
                 a newer loam may read it; this one will not guess"
            ),
            None => bail!(
                "{CONFIG_FILE} has no `format`, and this loam reads format {FORMAT}\n\
                 add `format = {FORMAT}` if the project was written for this version"
            ),
        }

        let mut warnings = Vec::new();
        let mut unknown = |where_: &str, key: &str| {
            warnings.push(format!(
                "{CONFIG_FILE}: unknown key `{where_}{key}`, ignored"
            ))
        };

        let table = |name: &str| doc.get(name).and_then(toml::Value::as_table);
        let string = |t: Option<&toml::Table>, key: &str| {
            t.and_then(|t| t.get(key))
                .and_then(toml::Value::as_str)
                .map(str::to_string)
        };

        for key in doc.keys() {
            if !["format", "docs", "links", "index", "hooks", "check", "kind"]
                .contains(&key.as_str())
            {
                unknown("", key);
            }
        }
        for (section, known) in [
            ("docs", &["root"][..]),
            ("links", &["cairn"][..]),
            ("index", &["path"][..]),
            ("hooks", &["after-change"][..]),
            ("check", &["severity"][..]),
        ] {
            if let Some(t) = table(section) {
                for key in t.keys() {
                    if !known.contains(&key.as_str()) {
                        unknown(&format!("{section}."), key);
                    }
                }
            }
        }

        let docs = clean(&string(table("docs"), "root").unwrap_or_else(|| "docs".into()));
        let cairn = string(table("links"), "cairn").map(|c| clean(&c));
        let index_rel = string(table("index"), "path").unwrap_or_else(|| "README.md".into());
        let index = join(&docs, index_rel.trim_matches('/'));
        let after_change = string(table("hooks"), "after-change");
        let mut severity = std::collections::HashMap::new();
        if let Some(t) = table("check")
            .and_then(|c| c.get("severity"))
            .and_then(toml::Value::as_table)
        {
            for (code, level) in t {
                match level.as_str() {
                    Some(l @ ("error" | "warning" | "ignore")) => {
                        severity.insert(code.clone(), l.to_string());
                    }
                    _ => bail!(
                        "{CONFIG_FILE}: check.severity.{code} must be \"error\", \"warning\" or \"ignore\""
                    ),
                }
            }
        }

        let mut kinds: Vec<Kind> = Vec::new();
        if let Some(list) = doc.get("kind") {
            let Some(list) = list.as_array() else {
                bail!("{CONFIG_FILE}: `kind` must be an array of tables, [[kind]]")
            };
            for (i, entry) in list.iter().enumerate() {
                let Some(t) = entry.as_table() else {
                    bail!("{CONFIG_FILE}: kind {} is not a table", i + 1)
                };
                let Some(name) = t.get("name").and_then(toml::Value::as_str) else {
                    bail!("{CONFIG_FILE}: kind {} has no `name`", i + 1)
                };
                for key in t.keys() {
                    if ![
                        "name",
                        "dir",
                        "title",
                        "description",
                        "template",
                        "index",
                        "stale_after",
                    ]
                    .contains(&key.as_str())
                    {
                        unknown("kind.", key);
                    }
                }
                let dir = clean(t.get("dir").and_then(toml::Value::as_str).unwrap_or("."));
                if let Some(other) = kinds.iter().find(|k| k.name == name) {
                    bail!("{CONFIG_FILE}: two kinds are called `{}`", other.name);
                }
                if let Some(other) = kinds.iter().find(|k| k.dir == dir) {
                    bail!(
                        "{CONFIG_FILE}: kinds `{}` and `{name}` both claim {}; a directory has one kind",
                        other.name,
                        if dir.is_empty() {
                            "the docs root".to_string()
                        } else {
                            format!("`{dir}`")
                        }
                    );
                }
                kinds.push(Kind {
                    name: name.to_string(),
                    dir,
                    title: t
                        .get("title")
                        .and_then(toml::Value::as_str)
                        .map(str::to_string),
                    description: t
                        .get("description")
                        .and_then(toml::Value::as_str)
                        .map(str::to_string),
                    template: t
                        .get("template")
                        .and_then(toml::Value::as_str)
                        .map(str::to_string),
                    index: t
                        .get("index")
                        .and_then(toml::Value::as_bool)
                        .unwrap_or(true),
                    stale_after: match t.get("stale_after") {
                        None => None,
                        Some(v) => match v.as_str().filter(|a| crate::fresh::parse_age(a).is_some()) {
                            Some(a) => Some(a.to_string()),
                            None => bail!("{CONFIG_FILE}: kind `{name}`: stale_after must be a number of days, weeks, months or years, as \"180d\", \"26w\", \"6m\" or \"1y\""),
                        },
                    },
                });
            }
        }

        Ok(Config {
            severity,
            root: root.to_path_buf(),
            docs,
            cairn,
            kinds,
            index,
            after_change,
            warnings,
        })
    }

    pub fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds.iter().find(|k| k.name == name)
    }

    /// Spec §5.4: the kind whose directory is the longest claim on `path`.
    pub fn kind_for(&self, path: &str) -> Option<&Kind> {
        let inside = self.inside_docs(path)?;
        let dir = inside.rsplit_once('/').map_or("", |(d, _)| d);
        self.kinds
            .iter()
            .filter(|k| k.dir.is_empty() || dir == k.dir || dir.starts_with(&format!("{}/", k.dir)))
            .max_by_key(|k| k.dir.len())
    }

    /// The part of a repository path below the docs root.
    pub fn inside_docs<'a>(&self, path: &'a str) -> Option<&'a str> {
        if self.docs.is_empty() {
            Some(path)
        } else {
            path.strip_prefix(&self.docs)?.strip_prefix('/')
        }
    }

    /// Spec §7.2: a Markdown file under the docs root, not hidden.
    pub fn is_page_path(&self, path: &str) -> bool {
        path.ends_with(".md")
            && self
                .inside_docs(path)
                .is_some_and(|inside| !inside.split('/').any(|p| p.starts_with(['.', '_'])))
    }

    pub fn abs(&self, path: &str) -> PathBuf {
        if path.is_empty() {
            self.root.clone()
        } else {
            self.root.join(path)
        }
    }
}

/// Join two repository-relative paths.
pub fn join(dir: &str, rest: &str) -> String {
    match (dir.is_empty(), rest.is_empty()) {
        (true, _) => rest.to_string(),
        (_, true) => dir.to_string(),
        _ => format!("{dir}/{rest}"),
    }
}

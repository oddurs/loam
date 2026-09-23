// loam — every page under the docs root, read into one model.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// This is the reader of spec/README.md. Every command starts from a `Tree`.
// A page that cannot be read is a finding on that page; it never stops the
// rest of the tree being read.

use crate::config::{Config, join};
use crate::scan::{self, Block, RawLink, WS};
use crate::yaml::{self, Yaml};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub line: usize,
    pub code: &'static str,
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkClass {
    External,
    Outside,
    Page,
    Cairn,
    File,
}

impl LinkClass {
    pub fn name(self) -> &'static str {
        match self {
            LinkClass::External => "external",
            LinkClass::Outside => "outside",
            LinkClass::Page => "page",
            LinkClass::Cairn => "cairn",
            LinkClass::File => "file",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Link {
    pub raw: RawLink,
    pub class: LinkClass,
    pub target: Option<String>,
    pub fragment: Option<String>,
    pub item: Option<u64>,
    pub current: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Page {
    pub path: String,
    /// The file as read, byte for byte.
    pub bytes: Vec<u8>,
    /// Without a byte order mark, CRLF as LF. `None` when not UTF-8.
    pub text: Option<String>,
    /// `Some` when the page has well-formed frontmatter, however empty.
    pub frontmatter: Option<Vec<(String, Yaml)>>,
    pub body: String,
    pub body_line: usize,
    pub title: Option<String>,
    pub title_from: Option<&'static str>,
    pub kind: Option<String>,
    pub kind_from: Option<&'static str>,
    pub status: String,
    pub status_from: &'static str,
    pub summary: Option<String>,
    pub summary_from: Option<&'static str>,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
    /// The program's own key: a page's place among its kind in the index.
    pub order: Option<i64>,
    pub anchors: Vec<String>,
    pub links: Vec<Link>,
    pub findings: Vec<Finding>,
}

impl Page {
    pub fn is_readable(&self) -> bool {
        self.text.is_some()
            && !self
                .findings
                .iter()
                .any(|f| f.detail.as_deref() == Some("no closing delimiter"))
    }
}

/// Exact, case-sensitive existence of repository paths (spec §2), without
/// trusting a filesystem that ignores case.
pub struct Fs {
    root: std::path::PathBuf,
    listings: RefCell<HashMap<String, Option<HashSet<String>>>>,
}

impl Fs {
    pub fn new(root: &Path) -> Fs {
        Fs {
            root: root.to_path_buf(),
            listings: RefCell::default(),
        }
    }

    fn listing(&self, dir: &str) -> Option<HashSet<String>> {
        if let Some(l) = self.listings.borrow().get(dir) {
            return l.clone();
        }
        let path = if dir.is_empty() {
            self.root.clone()
        } else {
            self.root.join(dir)
        };
        let names = std::fs::read_dir(&path).ok().map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        });
        self.listings
            .borrow_mut()
            .insert(dir.to_string(), names.clone());
        names
    }

    /// Whether `path` names a file or directory, spelled exactly so.
    pub fn exists(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        let mut dir = String::new();
        for part in path.split('/') {
            match self.listing(&dir) {
                Some(names) if names.contains(part) => dir = join(&dir, part),
                _ => return false,
            }
        }
        true
    }

    pub fn is_file(&self, path: &str) -> bool {
        self.exists(path) && self.root.join(path).is_file()
    }
}

pub struct Tree {
    pub config: Config,
    pub pages: BTreeMap<String, Page>,
    pub fs: Fs,
}

/// Split a file into frontmatter and body (spec §3.1).
/// Returns (frontmatter text, body, body's first line), or `None` for the
/// frontmatter when there is none; `Err` when it never closes.
pub fn split(text: &str) -> Result<(Option<String>, String, usize), ()> {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines[0].trim_end_matches(WS) != "---" {
        return Ok((None, text.to_string(), 1));
    }
    for i in 1..lines.len() {
        let l = lines[i].trim_end_matches(WS);
        if l == "---" || l == "..." {
            return Ok((
                Some(lines[1..i].join("\n")),
                lines[i + 1..].join("\n"),
                i + 2,
            ));
        }
    }
    Err(())
}

fn finding(line: usize, code: &'static str, detail: Option<&str>) -> Finding {
    Finding {
        line,
        code,
        detail: detail.map(str::to_string),
    }
}

/// Read one page from its bytes. Links are resolved later, against the tree.
pub fn read_page(config: &Config, path: &str, bytes: Vec<u8>) -> Page {
    let mut findings = Vec::new();
    let text = String::from_utf8(bytes.clone()).ok().map(|t| {
        t.strip_prefix('\u{feff}')
            .unwrap_or(&t)
            .replace("\r\n", "\n")
    });
    let mut frontmatter = None;
    let (mut body, mut body_line, mut readable) = (String::new(), 1, true);

    match &text {
        None => {
            findings.push(finding(1, "invalid-encoding", None));
            readable = false;
        }
        Some(t) => match split(t) {
            Err(()) => {
                findings.push(finding(
                    1,
                    "malformed-frontmatter",
                    Some("no closing delimiter"),
                ));
                readable = false;
            }
            Ok((front, b, first)) => {
                body = b;
                body_line = first;
                if let Some(front) = front {
                    match yaml::parse_mapping(&front) {
                        Ok(map) => frontmatter = Some(map),
                        Err(problem) => findings.push(finding(
                            1,
                            "malformed-frontmatter",
                            Some(problem.detail()),
                        )),
                    }
                }
            }
        },
    }

    let get = |key: &str| {
        frontmatter
            .as_ref()
            .and_then(|m| m.iter().find(|(k, _)| k == key))
            .map(|(_, v)| v)
    };
    let string_key = |key: &str, findings: &mut Vec<Finding>| match get(key) {
        None | Some(Yaml::Null) => None,
        Some(Yaml::Str(s)) => Some(s.clone()),
        Some(_) => {
            findings.push(finding(1, "malformed-key", Some(key)));
            None
        }
    };
    let path_list = |key: &str, findings: &mut Vec<Finding>| match get(key) {
        None | Some(Yaml::Null) => Vec::new(),
        Some(Yaml::Str(s)) => vec![s.clone()],
        Some(Yaml::Seq(items)) if items.iter().all(|i| matches!(i, Yaml::Str(_))) => items
            .iter()
            .map(|i| match i {
                Yaml::Str(s) => s.clone(),
                _ => unreachable!(),
            })
            .collect(),
        Some(_) => {
            findings.push(finding(1, "malformed-key", Some(key)));
            Vec::new()
        }
    };

    let blocks = if readable {
        scan::blocks(&body, body_line)
    } else {
        Vec::new()
    };

    // Keys in the order spec §4 lists them, so findings sort the same way
    // whichever order the frontmatter was written in.
    let fm_title = string_key("title", &mut findings);
    let fm_kind = string_key("kind", &mut findings);
    let fm_status = string_key("status", &mut findings);
    let fm_summary = string_key("summary", &mut findings);
    let supersedes = path_list("supersedes", &mut findings);
    let superseded_by = path_list("superseded_by", &mut findings);
    let order = match get("order") {
        None | Some(Yaml::Null) => None,
        Some(Yaml::Int(i)) => Some(*i),
        Some(_) => {
            findings.push(finding(1, "malformed-key", Some("order")));
            None
        }
    };

    // §5.2
    let title_index = blocks
        .iter()
        .position(|b| matches!(b, Block::Heading { level: 1, .. }));
    let (title, title_from) = match fm_title {
        Some(t) => (Some(t), Some("frontmatter")),
        None => match title_index.map(|i| &blocks[i]) {
            Some(Block::Heading { text, .. }) => (Some(text.clone()), Some("heading")),
            _ => (None, None),
        },
    };
    if title.is_none() && readable {
        findings.push(finding(1, "untitled", None));
    }

    // §5.4
    let (kind, kind_from) = match fm_kind {
        Some(k) => {
            if config.kind(&k).is_none() {
                findings.push(finding(1, "unknown-kind", Some(&k)));
            }
            (Some(k), Some("frontmatter"))
        }
        None => match config.kind_for(path) {
            Some(k) => (Some(k.name.clone()), Some("directory")),
            None => {
                findings.push(finding(1, "unclaimed", None));
                (None, None)
            }
        },
    };

    // §4.1, §5.5
    let (status, status_from) = match fm_status {
        Some(s) => {
            if !["draft", "current", "superseded"].contains(&s.as_str()) {
                findings.push(finding(1, "unknown-status", Some(&s)));
            }
            (s, "frontmatter")
        }
        None if !superseded_by.is_empty() => ("superseded".to_string(), "default"),
        None => ("current".to_string(), "default"),
    };

    // §5.3
    let (summary, summary_from) = match fm_summary {
        Some(s) => (Some(s), Some("frontmatter")),
        None => {
            let start = title_index.map_or(0, |i| i + 1);
            match scan::summary(&blocks[start..]) {
                Some(s) => (Some(s), Some("paragraph")),
                None => (None, None),
            }
        }
    };

    let anchors = scan::anchors(&blocks);
    let source_lines: Vec<&str> = text.as_deref().unwrap_or("").split('\n').collect();
    let raw_links = scan::links(&blocks, &|n| source_lines.get(n - 1).map(|s| s.to_string()));
    let links = raw_links
        .into_iter()
        .map(|raw| Link {
            raw,
            class: LinkClass::File,
            target: None,
            fragment: None,
            item: None,
            current: None,
        })
        .collect();

    Page {
        path: path.to_string(),
        bytes,
        text,
        frontmatter,
        body,
        body_line,
        title,
        title_from,
        kind,
        kind_from,
        status,
        status_from,
        summary,
        summary_from,
        supersedes,
        superseded_by,
        order,
        anchors,
        links,
        findings,
    }
}

/// Where a destination leads (spec §6.2): (target, fragment, class) where
/// class is "external", "outside", "self" or "path".
pub fn resolve(
    from_page: &str,
    destination: &str,
) -> (Option<String>, Option<String>, &'static str) {
    let dest = scan::unescape(destination);
    if is_external(&dest) {
        return (None, None, "external");
    }
    let (path, fragment) = match dest.split_once('#') {
        Some((p, f)) => (p, Some(percent_decode(f)).filter(|f| !f.is_empty())),
        None => (dest.as_str(), None),
    };
    let path = path.split('?').next().unwrap_or("");
    let path = percent_decode(path);
    if path.is_empty() {
        return (Some(from_page.to_string()), fragment, "self");
    }
    let joined = match path.strip_prefix('/') {
        Some(rooted) => rooted.to_string(),
        None => join(from_page.rsplit_once('/').map_or("", |(d, _)| d), &path),
    };
    match normalise(&joined) {
        Some(t) => (Some(t), fragment, "path"),
        None => (None, fragment, "outside"),
    }
}

pub fn is_external(dest: &str) -> bool {
    if dest.starts_with("//") {
        return true;
    }
    let mut chars = dest.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

/// Remove `.` and empty segments and apply `..`; `None` above the root.
pub fn normalise(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            p => parts.push(p),
        }
    }
    Some(parts.join("/"))
}

/// Percent-decoding as UTF-8, invalid sequences replaced, as a browser does.
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(h), Some(l)) = (
                (b[i + 1] as char).to_digit(16),
                (b[i + 2] as char).to_digit(16),
            )
        {
            out.push((h * 16 + l) as u8);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl Tree {
    pub fn read(config: Config) -> Result<Tree> {
        let fs = Fs::new(&config.root);
        let mut pages = BTreeMap::new();
        let start = config.docs.clone();
        let mut stack = vec![start];
        while let Some(dir) = stack.pop() {
            let abs = config.abs(&dir);
            let Ok(entries) = std::fs::read_dir(&abs) else {
                continue;
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(['.', '_']) {
                    continue;
                }
                let path = join(&dir, &name);
                let Ok(kind) = entry.file_type() else {
                    continue;
                };
                if kind.is_dir() {
                    stack.push(path);
                } else if config.is_page_path(&path) {
                    let bytes = std::fs::read(entry.path()).unwrap_or_default();
                    pages.insert(path.clone(), read_page(&config, &path, bytes));
                }
            }
        }
        let mut tree = Tree { config, pages, fs };
        tree.resolve_links();
        Ok(tree)
    }

    /// The anchors of any Markdown file in the repository, page or not.
    fn anchors_of(&self, path: &str, cache: &mut HashMap<String, Vec<String>>) -> Vec<String> {
        if let Some(p) = self.pages.get(path) {
            return p.anchors.clone();
        }
        cache
            .entry(path.to_string())
            .or_insert_with(|| {
                let bytes = std::fs::read(self.config.abs(path)).unwrap_or_default();
                let text = String::from_utf8_lossy(&bytes);
                let text = text
                    .strip_prefix('\u{feff}')
                    .unwrap_or(&text)
                    .replace("\r\n", "\n");
                match split(&text) {
                    Ok((_, body, first)) => scan::anchors(&scan::blocks(&body, first)),
                    Err(()) => Vec::new(),
                }
            })
            .clone()
    }

    fn cairn_items(&self) -> HashMap<u64, Vec<String>> {
        let mut items: HashMap<u64, Vec<String>> = HashMap::new();
        let Some(dir) = &self.config.cairn else {
            return items;
        };
        if let Ok(entries) = std::fs::read_dir(self.config.abs(dir)) {
            for e in entries.filter_map(|e| e.ok()) {
                let name = e.file_name().to_string_lossy().into_owned();
                if let Some(n) = leading_number(&name).filter(|_| name.ends_with(".md")) {
                    items.entry(n).or_default().push(join(dir, &name));
                }
            }
        }
        for v in items.values_mut() {
            v.sort();
        }
        items
    }

    /// Spec §6.4: the item a link names, if it is a cairn reference.
    fn cairn_id(&self, target: &str) -> Option<u64> {
        let dir = self.config.cairn.as_ref()?;
        let name = target.strip_prefix(dir.as_str())?.strip_prefix('/')?;
        if name.contains('/') || !name.ends_with(".md") {
            return None;
        }
        leading_number(name)
    }

    fn resolve_links(&mut self) {
        let items = self.cairn_items();
        let mut cache = HashMap::new();
        let paths: Vec<String> = self.pages.keys().cloned().collect();
        for path in &paths {
            let mut links = std::mem::take(&mut self.pages.get_mut(path).expect("listed").links);
            let mut findings = Vec::new();
            for link in &mut links {
                let (target, fragment, how) = resolve(path, &link.raw.destination);
                link.target = target.clone();
                link.fragment = fragment.clone();
                let mut code = None;
                match how {
                    "external" => link.class = LinkClass::External,
                    "outside" => {
                        link.class = LinkClass::Outside;
                        code = Some("escapes-repository");
                    }
                    _ => {
                        let target = target.expect("a path or self has a target");
                        if how == "self" {
                            link.class = LinkClass::Page;
                        } else if let Some(id) = self.cairn_id(&target) {
                            link.class = LinkClass::Cairn;
                            link.item = Some(id);
                            match items.get(&id) {
                                None => code = Some("broken-cairn-link"),
                                Some(files) if !files.contains(&target) => {
                                    code = Some("stale-cairn-link");
                                    link.current = Some(files[0].clone());
                                }
                                _ => {}
                            }
                        } else if self.pages.contains_key(&target) {
                            link.class = LinkClass::Page;
                        } else {
                            link.class = LinkClass::File;
                            if !self.fs.exists(&target) {
                                code = Some("broken-link");
                            }
                        }
                        if code.is_none()
                            && let Some(frag) = &fragment
                            && target.ends_with(".md")
                            && self.fs.is_file(&target)
                            && !self.anchors_of(&target, &mut cache).contains(frag)
                        {
                            code = Some("broken-anchor");
                        }
                    }
                }
                if let Some(code) = code {
                    findings.push(finding(link.raw.line, code, Some(&link.raw.destination)));
                }
            }
            let page = self.pages.get_mut(path).expect("listed");
            page.links = links;
            page.findings.extend(findings);
        }

        // §4.2
        for path in &paths {
            let page = &self.pages[path];
            let mut findings = Vec::new();
            for (key, inverse) in [
                ("supersedes", "superseded_by"),
                ("superseded_by", "supersedes"),
            ] {
                let values = if key == "supersedes" {
                    &page.supersedes
                } else {
                    &page.superseded_by
                };
                for value in values {
                    let (target, _, how) = resolve(path, value);
                    match target.filter(|t| how == "path" && self.pages.contains_key(t)) {
                        None => findings.push(finding(1, "broken-supersession", Some(value))),
                        Some(t) => {
                            let other = &self.pages[&t];
                            let back = if inverse == "supersedes" {
                                &other.supersedes
                            } else {
                                &other.superseded_by
                            };
                            if !back
                                .iter()
                                .any(|v| resolve(&t, v).0.as_deref() == Some(path.as_str()))
                            {
                                findings.push(finding(1, "one-sided-supersession", Some(value)));
                            }
                        }
                    }
                }
            }
            let page = self.pages.get_mut(path).expect("listed");
            page.findings.extend(findings);
            page.findings.sort();
        }
    }
}

pub fn leading_number(name: &str) -> Option<u64> {
    let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

impl Finding {
    /// A sentence for a person, from the code and its detail.
    pub fn message(&self) -> String {
        let d = self.detail.as_deref().unwrap_or("");
        match self.code {
            "invalid-encoding" => "not UTF-8, so nothing in it can be read".into(),
            "malformed-frontmatter" => format!("frontmatter is {d}"),
            "malformed-key" => format!("`{d}` has a value of the wrong type, and is ignored"),
            "unknown-status" => format!("status `{d}` is not draft, current or superseded"),
            "broken-supersession" => format!("supersession names `{d}`, which is not a page"),
            "one-sided-supersession" => format!("`{d}` does not say so in return"),
            "untitled" => "no title, and no level-1 heading to take one from".into(),
            "unclaimed" => "no kind claims this directory, and the page names none".into(),
            "unknown-kind" => format!("kind `{d}` is not declared in loam.toml"),
            "broken-link" => format!("link to `{d}`, which does not exist"),
            "escapes-repository" => format!("link to `{d}` leaves the repository"),
            "broken-anchor" => format!("link to `{d}`, whose anchor does not exist"),
            "stale-cairn-link" => format!("link to `{d}` names an item that has been renamed"),
            "broken-cairn-link" => format!("link to `{d}` names no item in the backlog"),
            other => format!("{other} {d}"),
        }
    }
}

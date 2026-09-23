// loam — the write path: a lock, atomic writes, and frontmatter edits that
// leave every byte they were not asked to change.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// The spec puts four musts on a writer (cairn item 0062): reproduce the line
// endings a file used (§3.1), quote a value that would otherwise read back as
// something else (§3.2), keep every frontmatter key it did not change, in
// order (§4.5), and never reorder the kinds (§7.1 — no command here rewrites
// loam.toml). Editing the frontmatter as text, one key at a time, is what makes
// the third cheap to keep: a key loam does not know is never parsed and
// re-emitted, so it cannot be reformatted.

use crate::config::Config;
use crate::yaml::{self, Yaml};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

// ─── The lock ────────────────────────────────────────────────────────────────

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);
const STALE_AFTER: Duration = Duration::from_secs(300);

/// Held for the duration of a write. Readers never take it.
pub struct Lock {
    path: PathBuf,
}

impl Lock {
    /// Inside the docs root, where a dotfile is never a page (spec §7.2).
    pub fn acquire(config: &Config) -> Result<Lock> {
        let dir = config.abs(&config.docs);
        std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        let path = dir.join(".loam.lock");
        let deadline = Instant::now() + ACQUIRE_TIMEOUT;
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    use std::io::Write;
                    let _ = writeln!(f, "pid {}", std::process::id());
                    return Ok(Lock { path });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e).with_context(|| format!("creating {}", path.display())),
            }
            let age = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| SystemTime::now().duration_since(t).ok());
            if age.is_some_and(|a| a > STALE_AFTER) {
                eprintln!(
                    "loam: breaking a lock left behind by a process that is gone ({})",
                    path.display()
                );
                let _ = std::fs::remove_file(&path);
                continue;
            }
            if Instant::now() >= deadline {
                bail!(
                    "another loam is writing to these pages\n\
                     waited {}s for {}; if nothing else is running, delete that file",
                    ACQUIRE_TIMEOUT.as_secs(),
                    path.display()
                );
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// ─── Atomic writes ───────────────────────────────────────────────────────────

/// Write so that a reader sees the old contents or the new, never a mixture:
/// a temporary file beside the target, flushed, then renamed over it.
pub fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    let name = path
        .file_name()
        .map_or_else(|| "loam".into(), |n| n.to_string_lossy().into_owned());
    let temp = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| -> Result<()> {
        let mut f =
            std::fs::File::create(&temp).with_context(|| format!("creating {}", temp.display()))?;
        f.write_all(contents)
            .with_context(|| format!("writing {}", temp.display()))?;
        f.sync_all()
            .with_context(|| format!("flushing {}", temp.display()))?;
        std::fs::rename(&temp, path).with_context(|| format!("replacing {}", path.display()))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result?;
    #[cfg(unix)]
    if let Ok(dir) = std::fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

// ─── A file as lines, keeping what ends them ─────────────────────────────────

/// A text file split so it can be put back exactly: a byte order mark, and
/// each line with its own terminator (`\n`, `\r\n`, or none on the last).
#[derive(Clone, Debug, PartialEq)]
pub struct Lines {
    pub bom: bool,
    pub lines: Vec<(String, &'static str)>,
}

impl Lines {
    pub fn parse(text: &str) -> Lines {
        let (bom, text) = match text.strip_prefix('\u{feff}') {
            Some(t) => (true, t),
            None => (false, text),
        };
        let mut lines = Vec::new();
        let mut rest = text;
        loop {
            match rest.find('\n') {
                Some(i) => {
                    let (content, eol) = match rest[..i].strip_suffix('\r') {
                        Some(c) => (c, "\r\n"),
                        None => (&rest[..i], "\n"),
                    };
                    lines.push((content.to_string(), eol));
                    rest = &rest[i + 1..];
                }
                None => {
                    lines.push((rest.to_string(), ""));
                    break;
                }
            }
        }
        Lines { bom, lines }
    }

    /// The ending a new line should have: whatever the file mostly uses.
    pub fn eol(&self) -> &'static str {
        let crlf = self.lines.iter().filter(|(_, e)| *e == "\r\n").count();
        let lf = self.lines.iter().filter(|(_, e)| *e == "\n").count();
        if crlf > lf { "\r\n" } else { "\n" }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        if self.bom {
            out.push('\u{feff}');
        }
        for (content, eol) in &self.lines {
            out.push_str(content);
            out.push_str(eol);
        }
        out
    }

    /// Insert whole lines before index `at`, each ended as the file prefers.
    pub fn insert(&mut self, at: usize, new: &[String]) {
        let eol = self.eol();
        for (i, l) in new.iter().enumerate() {
            self.lines.insert(at + i, (l.clone(), eol));
        }
        // An insertion at the very end follows a last line with no ending.
        if at > 0 && at == self.lines.len() - new.len() && self.lines[at - 1].1.is_empty() {
            self.lines[at - 1].1 = eol;
            if let Some(last) = self.lines.last_mut() {
                last.1 = "";
            }
        }
    }

    /// The indices of the frontmatter's opening and closing delimiters.
    pub fn frontmatter(&self) -> Option<(usize, usize)> {
        let trimmed = |i: usize| self.lines[i].0.trim_end_matches([' ', '\t']).to_string();
        if self.lines.is_empty() || trimmed(0) != "---" {
            return None;
        }
        (1..self.lines.len())
            .find(|&i| matches!(trimmed(i).as_str(), "---" | "..."))
            .map(|close| (0, close))
    }
}

// ─── Values ──────────────────────────────────────────────────────────────────

/// A string as a YAML scalar that reads back as the same string (spec §3.2).
pub fn scalar(s: &str) -> String {
    let plain_ok = !s.is_empty()
        && yaml::resolve(s) == Yaml::Str(s.to_string())
        && !s.starts_with([
            '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"', '%',
            '@', '`', ' ',
        ])
        && !s.ends_with([' ', ':'])
        && !s.contains(": ")
        && !s.contains(" #")
        && !s.contains(['\n', '\r', '\t'])
        && s.chars().all(|c| !c.is_control());
    if plain_ok {
        return s.to_string();
    }
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub enum Value {
    Str(String),
    List(Vec<String>),
}

fn key_lines(key: &str, value: &Value) -> Vec<String> {
    match value {
        Value::Str(s) => vec![format!("{key}: {}", scalar(s))],
        Value::List(items) if items.is_empty() => vec![format!("{key}: []")],
        Value::List(items) => {
            let mut out = vec![format!("{key}:")];
            out.extend(items.iter().map(|i| format!("  - {}", scalar(i))));
            out
        }
    }
}

/// The lines a top-level key occupies inside the frontmatter: its own, and
/// every following line that belongs to its value — indented, a block
/// sequence entry at the margin, or blank inside such a value.
fn key_extent(lines: &Lines, open: usize, close: usize, key: &str) -> Option<(usize, usize)> {
    let starts = |l: &str| {
        l.strip_prefix(key).is_some_and(|rest| {
            rest.starts_with(':') && (rest.len() == 1 || rest[1..].starts_with([' ', '\t']))
        })
    };
    let start = (open + 1..close).find(|&i| starts(&lines.lines[i].0))?;
    let mut end = start + 1;
    while end < close {
        let l = &lines.lines[end].0;
        let continues = l.starts_with([' ', '\t']) || l.starts_with("- ") || l == "-";
        if continues {
            end += 1;
        } else if l.trim().is_empty() {
            // Blank: part of the value only if the value continues after it.
            let next = (end..close).find(|&j| !lines.lines[j].0.trim().is_empty());
            match next {
                Some(j) if lines.lines[j].0.starts_with([' ', '\t']) => end = j,
                _ => break,
            }
        } else {
            break;
        }
    }
    Some((start, end))
}

/// Set a top-level frontmatter key, replacing its lines where it already is
/// and appending it otherwise. A page without frontmatter gains some.
pub fn set_key(lines: &mut Lines, key: &str, value: &Value) {
    let new = key_lines(key, value);
    match lines.frontmatter() {
        Some((open, close)) => match key_extent(lines, open, close, key) {
            Some((start, end)) => {
                let eol = lines.eol();
                let endings: Vec<&'static str> =
                    lines.lines[start..end].iter().map(|(_, e)| *e).collect();
                lines.lines.drain(start..end);
                for (i, l) in new.iter().enumerate() {
                    lines.lines.insert(
                        start + i,
                        (l.clone(), endings.get(i).copied().unwrap_or(eol)),
                    );
                }
            }
            None => lines.insert(close, &new),
        },
        None => {
            let mut block = vec!["---".to_string()];
            block.extend(new);
            block.push("---".to_string());
            let first_blank = lines
                .lines
                .first()
                .is_some_and(|(l, _)| l.trim().is_empty());
            if !first_blank {
                block.push(String::new());
            }
            lines.insert(0, &block);
        }
    }
}

/// Set fields of a top-level mapping key, written as a block, keeping any
/// other fields already in it (spec §4.5). A key that is absent, or written
/// in some other shape, is replaced by a block of just these fields.
pub fn set_mapping(lines: &mut Lines, key: &str, fields: &[(&str, String)]) {
    let block = lines.frontmatter().and_then(|(open, close)| {
        let (start, end) = key_extent(lines, open, close, key)?;
        (lines.lines[start].0.trim_end() == format!("{key}:")).then_some((start, end))
    });
    let Some((start, mut end)) = block else {
        let mut new = vec![format!("{key}:")];
        new.extend(fields.iter().map(|(k, v)| format!("  {k}: {}", scalar(v))));
        remove_key(lines, key);
        match lines.frontmatter() {
            Some((_, close)) => lines.insert(close, &new),
            None => {
                set_key(lines, key, &Value::Str(String::new()));
                remove_key(lines, key);
                let close = lines.frontmatter().map_or(1, |(_, c)| c);
                lines.insert(close, &new);
            }
        }
        return;
    };
    let indent = lines.lines[start + 1..end]
        .iter()
        .find(|(l, _)| !l.trim().is_empty())
        .map_or("  ".to_string(), |(l, _)| {
            l[..l.len() - l.trim_start().len()].to_string()
        });
    for (field, value) in fields {
        let line = format!("{indent}{field}: {}", scalar(value));
        let prefix = format!("{indent}{field}:");
        match (start + 1..end).find(|&i| lines.lines[i].0.starts_with(&prefix)) {
            Some(i) => lines.lines[i].0 = line,
            None => {
                lines.insert(end, &[line]);
                end += 1;
            }
        }
    }
}

/// Remove a top-level key, if the frontmatter has it.
pub fn remove_key(lines: &mut Lines, key: &str) {
    if let Some((open, close)) = lines.frontmatter()
        && let Some((start, end)) = key_extent(lines, open, close, key)
    {
        lines.lines.drain(start..end);
    }
}

/// The index of the first line after the frontmatter, or 0 without one.
pub fn body_start(lines: &Lines) -> usize {
    lines.frontmatter().map_or(0, |(_, close)| close + 1)
}

// ─── Paths ───────────────────────────────────────────────────────────────────

/// The relative path from the directory `from_dir` to `to`, both relative to
/// the repository, as a link from a page in `from_dir` would write it.
pub fn relative(from_dir: &str, to: &str) -> String {
    let from: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to_parts: Vec<&str> = to.split('/').filter(|s| !s.is_empty()).collect();
    let common = from
        .iter()
        .zip(&to_parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts: Vec<&str> = std::iter::repeat_n("..", from.len() - common).collect();
    parts.extend(&to_parts[common..]);
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

pub fn dir_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(d, _)| d)
}

/// Encode a path for a link destination written without angle brackets.
pub fn encode_destination(path: &str) -> String {
    let mut out = String::new();
    for c in path.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untouched_lines_round_trip() {
        for text in [
            "",
            "a",
            "a\n",
            "a\r\nb\n",
            "\u{feff}---\r\nx: 1\r\n---\r\nbody",
            "x\n\n",
        ] {
            assert_eq!(Lines::parse(text).render(), text);
        }
    }

    #[test]
    fn scalars_read_back_as_written() {
        for s in [
            "no",
            "yes",
            "12:30",
            "true",
            "null",
            "~",
            "1.2",
            "012",
            "a: b",
            "a #b",
            "- x",
            "",
            " lead",
            "é",
            "plain words",
            "docs/x.md",
        ] {
            let text = format!("k: {}\n", scalar(s));
            let parsed = yaml::parse_mapping(&text).unwrap();
            assert_eq!(
                parsed[0].1,
                Yaml::Str(s.to_string()),
                "{s:?} written as {text:?}"
            );
        }
        assert_eq!(scalar("docs/x.md"), "docs/x.md");
    }

    #[test]
    fn set_key_keeps_the_rest_and_the_line_endings() {
        let mut l = Lines::parse(
            "---\r\nlayout: post\r\nstatus: current\r\ntags:\r\n  - a\r\n---\r\n# T\r\n",
        );
        set_key(&mut l, "status", &Value::Str("superseded".into()));
        set_key(&mut l, "superseded_by", &Value::List(vec!["new.md".into()]));
        assert_eq!(
            l.render(),
            "---\r\nlayout: post\r\nstatus: superseded\r\ntags:\r\n  - a\r\nsuperseded_by:\r\n  - new.md\r\n---\r\n# T\r\n"
        );
        set_key(&mut l, "tags", &Value::Str("b".into()));
        assert!(l.render().contains("tags: b\r\nsuperseded_by"));
    }

    #[test]
    fn a_page_without_frontmatter_gains_it() {
        let mut l = Lines::parse("# Title\n\nText.\n");
        set_key(&mut l, "status", &Value::Str("draft".into()));
        assert_eq!(l.render(), "---\nstatus: draft\n---\n\n# Title\n\nText.\n");
    }

    #[test]
    fn set_mapping_keeps_other_fields_and_replaces_other_shapes() {
        let mut l = Lines::parse("---\nreviewed:\n    by: me\n    commit: old\n---\n# T\n");
        set_mapping(
            &mut l,
            "reviewed",
            &[("commit", "abc1234".into()), ("date", "2026-09-23".into())],
        );
        assert_eq!(
            l.render(),
            "---\nreviewed:\n    by: me\n    commit: abc1234\n    date: 2026-09-23\n---\n# T\n"
        );
        let mut l = Lines::parse("---\nreviewed: {commit: x}\ntitle: T\n---\n");
        set_mapping(&mut l, "reviewed", &[("commit", "1234567".into())]);
        assert_eq!(
            l.render(),
            "---\ntitle: T\nreviewed:\n  commit: \"1234567\"\n---\n"
        );
        let mut l = Lines::parse("# T\n");
        set_mapping(&mut l, "reviewed", &[("date", "2026-09-23".into())]);
        assert_eq!(
            l.render(),
            "---\nreviewed:\n  date: 2026-09-23\n---\n\n# T\n"
        );
    }

    #[test]
    fn relative_paths() {
        assert_eq!(relative("docs/guide", "docs/design/x.md"), "../design/x.md");
        assert_eq!(relative("docs", "docs/x.md"), "x.md");
        assert_eq!(relative("", "docs/x.md"), "docs/x.md");
        assert_eq!(relative("docs/a/b", "README.md"), "../../../README.md");
    }
}

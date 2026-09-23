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

/// Write a file that must not exist yet: never replaces one, even one another
/// program created a moment ago. The data goes to a temporary file, which is
/// then hard-linked into place — an operation that fails if the name is taken.
pub fn write_new(path: &Path, contents: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    let name = path
        .file_name()
        .map_or_else(|| "loam".into(), |n| n.to_string_lossy().into_owned());
    let temp = parent.join(format!(".{name}.{}.new", std::process::id()));
    let result = (|| -> Result<()> {
        let mut f =
            std::fs::File::create(&temp).with_context(|| format!("creating {}", temp.display()))?;
        f.write_all(contents)
            .with_context(|| format!("writing {}", temp.display()))?;
        f.sync_all()
            .with_context(|| format!("flushing {}", temp.display()))?;
        std::fs::hard_link(&temp, path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                anyhow::anyhow!(
                    "{} already exists; loam new never overwrites a page",
                    path.display()
                )
            } else {
                anyhow::Error::from(e).context(format!("creating {}", path.display()))
            }
        })
    })();
    let _ = std::fs::remove_file(&temp);
    result
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
    /// Written exactly as given: a number, a boolean, already a scalar.
    Raw(String),
}

fn key_lines(key: &str, value: &Value) -> Vec<String> {
    match value {
        Value::Str(s) => vec![format!("{key}: {}", scalar(s))],
        Value::Raw(s) => vec![format!("{key}: {s}")],
        Value::List(items) if items.is_empty() => vec![format!("{key}: []")],
        Value::List(items) => {
            let mut out = vec![format!("{key}:")];
            out.extend(items.iter().map(|i| format!("  - {}", scalar(i))));
            out
        }
    }
}

fn indent_of(l: &str) -> usize {
    l.len() - l.trim_start_matches([' ', '\t']).len()
}

fn is_comment_or_blank(l: &str) -> bool {
    let t = l.trim();
    t.is_empty() || t.starts_with('#')
}

/// The indent of the frontmatter's top-level keys: usually none, but a mapping
/// indented as a whole is still a mapping.
fn base_indent(lines: &Lines, open: usize, close: usize) -> usize {
    (open + 1..close)
        .map(|i| lines.lines[i].0.as_str())
        .find(|l| !is_comment_or_blank(l))
        .map_or(0, indent_of)
}

/// Whether `line`, at the base indent, begins `key`: written plain, in either
/// kind of quotes, and with or without space before its colon.
fn begins_key(line: &str, base: usize, key: &str) -> bool {
    if indent_of(line) != base {
        return false;
    }
    let rest = &line[base..];
    let after = [key.to_string(), format!("\"{key}\""), format!("'{key}'")]
        .iter()
        .find_map(|k| rest.strip_prefix(k.as_str()))
        .map(|r| r.trim_start_matches([' ', '\t']));
    after.is_some_and(|r| r.starts_with(':') && (r.len() == 1 || r[1..].starts_with([' ', '\t'])))
}

/// The lines each top-level occurrence of a key occupies: its own, and every
/// following line belonging to its value — more indented, a sequence entry at
/// the key's own indent, or a blank or comment line with more of the value
/// after it. A key written twice is legal YAML, and the last one is the one
/// read, so the last is the one an edit must change.
fn key_extents(lines: &Lines, open: usize, close: usize, key: &str) -> Vec<(usize, usize)> {
    extents_in(lines, open + 1, close, base_indent(lines, open, close), key)
}

/// As `key_extents`, for the keys at indent `base` among lines `from..close`:
/// the fields of a nested mapping, as well as the top level.
fn extents_in(
    lines: &Lines,
    from: usize,
    close: usize,
    base: usize,
    key: &str,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = from;
    while i < close {
        if !begins_key(&lines.lines[i].0, base, key) {
            i += 1;
            continue;
        }
        let start = i;
        let mut end = start + 1;
        while end < close {
            let l = &lines.lines[end].0;
            let entry = indent_of(l) == base
                && (l[base..].starts_with("- ") || l[base..].trim_end() == "-");
            if !is_comment_or_blank(l) && (indent_of(l) > base || entry) {
                end += 1;
                continue;
            }
            if is_comment_or_blank(l) {
                // Part of the value only if the value goes on after it.
                let next = (end..close).find(|&j| !is_comment_or_blank(&lines.lines[j].0));
                match next {
                    Some(j)
                        if indent_of(&lines.lines[j].0) > base
                            || (indent_of(&lines.lines[j].0) == base
                                && lines.lines[j].0[base..].starts_with("- ")) =>
                    {
                        end = j;
                        continue;
                    }
                    _ => break,
                }
            }
            break;
        }
        out.push((start, end));
        i = end;
    }
    out
}

fn key_extent(lines: &Lines, open: usize, close: usize, key: &str) -> Option<(usize, usize)> {
    key_extents(lines, open, close, key).pop()
}

/// Set a top-level frontmatter key, replacing its lines where it already is
/// and appending it otherwise. A page without frontmatter gains some.
pub fn set_key(lines: &mut Lines, key: &str, value: &Value) {
    let base = lines
        .frontmatter()
        .map_or(0, |(o, c)| base_indent(lines, o, c));
    let pad = " ".repeat(base);
    let new: Vec<String> = key_lines(key, value)
        .into_iter()
        .map(|l| format!("{pad}{l}"))
        .collect();
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
    // The key's last occurrence, when it is written as a block: nothing after
    // its colon but perhaps a comment.
    let block = lines.frontmatter().and_then(|(open, close)| {
        let (start, end) = key_extent(lines, open, close, key)?;
        let line = &lines.lines[start].0;
        let after = line[line.find(':')? + 1..].trim();
        (after.is_empty() || after.starts_with('#')).then_some((
            start,
            end,
            base_indent(lines, open, close),
        ))
    });
    let Some((start, mut end, base)) = block else {
        let pad = " ".repeat(
            lines
                .frontmatter()
                .map_or(0, |(o, c)| base_indent(lines, o, c)),
        );
        let mut new = vec![format!("{pad}{key}:")];
        new.extend(
            fields
                .iter()
                .map(|(k, v)| format!("{pad}  {k}: {}", scalar(v))),
        );
        remove_key(lines, key);
        if lines.frontmatter().is_none() {
            set_key(lines, key, &Value::Str(String::new()));
            remove_key(lines, key);
        }
        let close = lines.frontmatter().map_or(1, |(_, c)| c);
        lines.insert(close, &new);
        return;
    };
    // The fields' indent, from the first line that is one — not a comment.
    let indent = (start + 1..end)
        .map(|i| lines.lines[i].0.as_str())
        .find(|l| !is_comment_or_blank(l))
        .map_or(base + 2, indent_of);
    let pad = " ".repeat(indent);
    for (field, value) in fields {
        let line = format!("{pad}{field}: {}", scalar(value));
        match extents_in(lines, start + 1, end, indent, field).pop() {
            // The field and any lines its value runs on to: a folded scalar.
            Some((f, t)) => {
                let eol = lines.lines[f].1;
                lines.lines.drain(f..t);
                lines.lines.insert(f, (line, eol));
                end -= t - f - 1;
            }
            None => {
                lines.insert(end, &[line]);
                end += 1;
            }
        }
    }
}

/// Remove a top-level key, if the frontmatter has it.
pub fn remove_key(lines: &mut Lines, key: &str) {
    if let Some((open, close)) = lines.frontmatter() {
        // Every occurrence, last first so earlier indices stay true.
        for (start, end) in key_extents(lines, open, close, key).into_iter().rev() {
            lines.lines.drain(start..end);
        }
    }
}

/// The index of the first line after the frontmatter, or 0 without one.
pub fn body_start(lines: &Lines) -> usize {
    lines.frontmatter().map_or(0, |(_, close)| close + 1)
}

// ─── Markers ─────────────────────────────────────────────────────────────────

/// The lines holding a pair of marker comments, each alone on its line and
/// outside code: a fenced example of the markers, or a sentence mentioning
/// them, is not where generated text goes.
pub fn markers(lines: &Lines, begin: &str, end: &str) -> Option<(usize, usize)> {
    let text: Vec<&str> = lines.lines.iter().map(|(l, _)| l.as_str()).collect();
    let code: std::collections::HashSet<usize> = crate::scan::blocks(&text.join("\n"), 1)
        .into_iter()
        .filter_map(|b| match b {
            crate::scan::Block::Code { line } => Some(line - 1),
            _ => None,
        })
        .collect();
    let find = |m: &str, from: usize| {
        (from..text.len()).find(|&i| !code.contains(&i) && text[i].trim() == m)
    };
    let b = find(begin, 0)?;
    let e = find(end, b + 1)?;
    Some((b, e))
}

/// Replace the lines from `begin` to `end`, inclusive, with `new`, each ended
/// as the file prefers; the last keeps whatever ended the old last line.
pub fn replace_lines(lines: &mut Lines, begin: usize, end: usize, new: &[String]) {
    let eol = lines.eol();
    let last = lines.lines[end].1;
    lines.lines.drain(begin..=end);
    for (i, l) in new.iter().enumerate() {
        lines.lines.insert(begin + i, (l.clone(), eol));
    }
    if !new.is_empty() {
        lines.lines[begin + new.len() - 1].1 = last;
    }
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

    fn reads(text: &str, key: &str) -> Option<Yaml> {
        let l = Lines::parse(text);
        let (open, close) = l.frontmatter()?;
        let front: Vec<String> = l.lines[open + 1..close]
            .iter()
            .map(|(t, _)| t.clone())
            .collect();
        let map = yaml::parse_mapping(&front.join("\n")).ok()?;
        map.into_iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    #[test]
    fn a_comment_at_the_margin_inside_a_value_stays_inside_it() {
        let mut l =
            Lines::parse("---\ncovers:\n# the core\n  - src/a.rs\n  - src/b.rs\ntitle: T\n---\n");
        set_key(&mut l, "covers", &Value::List(vec!["src/c.rs".into()]));
        assert_eq!(l.render(), "---\ncovers:\n  - src/c.rs\ntitle: T\n---\n");
    }

    #[test]
    fn a_key_written_twice_is_edited_where_yaml_reads_it() {
        let mut l = Lines::parse("---\nstatus: draft\ntitle: A\nstatus: current\n---\n");
        set_key(&mut l, "status", &Value::Str("superseded".into()));
        assert_eq!(
            reads(&l.render(), "status"),
            Some(Yaml::Str("superseded".into()))
        );
        remove_key(&mut l, "status");
        assert_eq!(l.render(), "---\ntitle: A\n---\n", "every occurrence goes");
    }

    #[test]
    fn keys_in_other_styles_are_found() {
        for (text, want) in [
            (
                "---\n\"status\": draft\n---\n",
                "---\nstatus: current\n---\n",
            ),
            ("---\n'status': draft\n---\n", "---\nstatus: current\n---\n"),
            ("---\nstatus : draft\n---\n", "---\nstatus: current\n---\n"),
            (
                "---\n  title: A\n  status: draft\n---\n",
                "---\n  title: A\n  status: current\n---\n",
            ),
        ] {
            let mut l = Lines::parse(text);
            set_key(&mut l, "status", &Value::Str("current".into()));
            assert_eq!(l.render(), want, "{text:?}");
            assert_eq!(
                reads(&l.render(), "status"),
                Some(Yaml::Str("current".into()))
            );
        }
    }

    #[test]
    fn set_mapping_survives_comments_folded_values_and_trailing_comments() {
        // A folded commit: the whole value is replaced, not its first line.
        let mut l = Lines::parse(
            "---\nreviewed:\n  commit: >-\n    0123456789abcdef\n  date: 2026-01-01\n---\n",
        );
        set_mapping(
            &mut l,
            "reviewed",
            &[("commit", "abcdef1".into()), ("date", "2026-09-23".into())],
        );
        assert_eq!(
            l.render(),
            "---\nreviewed:\n  commit: abcdef1\n  date: 2026-09-23\n---\n"
        );
        // A comment indented differently does not set the fields' indent.
        let mut l = Lines::parse("---\nreviewed:\n    # by hand\n  commit: abcdef1\n---\n");
        set_mapping(
            &mut l,
            "reviewed",
            &[("commit", "1234abc".into()), ("date", "2026-09-23".into())],
        );
        assert_eq!(
            l.render(),
            "---\nreviewed:\n    # by hand\n  commit: 1234abc\n  date: 2026-09-23\n---\n"
        );
        assert!(reads(&l.render(), "reviewed").is_some(), "still YAML");
        // A comment after the key's colon: still a block, and `by` is kept.
        let mut l = Lines::parse("---\nreviewed: # hand\n  by: me\n---\n");
        set_mapping(&mut l, "reviewed", &[("commit", "abcdef1".into())]);
        assert_eq!(
            l.render(),
            "---\nreviewed: # hand\n  by: me\n  commit: abcdef1\n---\n"
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

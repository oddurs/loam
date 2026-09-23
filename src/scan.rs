// loam — the block scan, anchors and links of a page's body.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// This is spec §5.1, §6.1 and §6.3, and deliberately nothing more: a small
// line-based scan every reader performs exactly, not a Markdown parser. Where
// it differs from GitHub the spec says so and says why. Each function names
// the section it implements; read them side by side.

use regex::Regex;
use std::sync::LazyLock;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

/// Whitespace is spaces and tabs, and nothing else (spec §2).
pub const WS: &[char] = &[' ', '\t'];

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// One line of fenced or indented code.
    Code {
        line: usize,
    },
    Heading {
        line: usize,
        level: usize,
        text: String,
    },
    /// Consecutive lines, each with its line number.
    Paragraph {
        lines: Vec<(usize, String)>,
    },
    Break {
        line: usize,
    },
}

static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^ {0,3}(`{3,}|~{3,})(.*)$").unwrap());
static FENCE_CLOSE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}(`+|~+)[ \t]*$").unwrap());
static ATX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}(#{1,6})(?:[ \t]+(.*?))?[ \t]*$").unwrap());
static CLOSING_HASHES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|[ \t]+)#+$").unwrap());
static SETEXT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^ {0,3}(=+|-+)[ \t]*$").unwrap());
static LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}(?:[-*+]|[0-9]{1,9}[.)])(?:[ \t]|$)").unwrap());
static REFDEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}\[(?:[^\]\\]|\\.)+\]:[ \t]*(<[^>\n]*>|[^ \t]+)").unwrap());

fn is_break(line: &str) -> bool {
    // Up to three spaces, then three or more of one of `-`, `*`, `_`, with only
    // whitespace between and after. The regex crate has no backreferences.
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return false;
    }
    let rest = &line[indent..];
    let Some(c) = rest.chars().next() else {
        return false;
    };
    if !matches!(c, '-' | '*' | '_') {
        return false;
    }
    let mut count = 0;
    for ch in rest.chars() {
        if ch == c {
            count += 1;
        } else if ch != ' ' && ch != '\t' {
            return false;
        }
    }
    count >= 3
}

fn is_list(line: &str) -> bool {
    LIST.is_match(line)
}

fn is_quote(line: &str) -> bool {
    line.trim_start_matches(WS).starts_with('>')
}

/// Spec §5.1. `first_line` is the file's line number of the body's first line.
pub fn blocks(body: &str, first_line: usize) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para: Vec<(usize, String)> = Vec::new();
    let mut fence: Option<String> = None;
    let mut indented_code = false;
    let mut prev_blank = true;

    fn flush(out: &mut Vec<Block>, para: &mut Vec<(usize, String)>) {
        if !para.is_empty() {
            out.push(Block::Paragraph {
                lines: std::mem::take(para),
            });
        }
    }

    for (offset, line) in body.split('\n').enumerate() {
        let n = first_line + offset;
        if let Some(open) = &fence {
            if let Some(m) = FENCE_CLOSE.captures(line) {
                let run = &m[1];
                if run.starts_with(&open[..1]) && run.len() >= open.len() {
                    fence = None;
                }
            }
            out.push(Block::Code { line: n });
            continue;
        }
        if line.trim_matches(WS).is_empty() {
            flush(&mut out, &mut para);
            prev_blank = true;
            continue;
        }
        let indented = line.starts_with('\t') || line.starts_with("    ");
        if indented && (indented_code || (prev_blank && para.is_empty())) {
            indented_code = true;
            prev_blank = false;
            out.push(Block::Code { line: n });
            continue;
        }
        indented_code = false;
        prev_blank = false;
        if let Some(m) = FENCE.captures(line) {
            let run = &m[1];
            if !(run.starts_with('`') && m[2].contains('`')) {
                flush(&mut out, &mut para);
                fence = Some(run.to_string());
                out.push(Block::Code { line: n });
                continue;
            }
        }
        if let Some(m) = SETEXT.captures(line)
            && !para.is_empty()
            && !is_list(&para[0].1)
            && !is_quote(&para[0].1)
        {
            let level = if m[1].starts_with('=') { 1 } else { 2 };
            let text = para
                .iter()
                .map(|(_, t)| t.trim_matches(WS))
                .collect::<Vec<_>>()
                .join(" ");
            let line = para[0].0;
            para.clear();
            out.push(Block::Heading { line, level, text });
            continue;
        }
        if let Some(m) = ATX.captures(line) {
            flush(&mut out, &mut para);
            let text = m.get(2).map_or("", |g| g.as_str());
            let text = CLOSING_HASHES
                .replace(text, "")
                .trim_matches(WS)
                .to_string();
            out.push(Block::Heading {
                line: n,
                level: m[1].len(),
                text,
            });
            continue;
        }
        if is_break(line) {
            flush(&mut out, &mut para);
            out.push(Block::Break { line: n });
            continue;
        }
        if !para.is_empty()
            && (is_list(line) || is_quote(line))
            && !(is_list(&para[0].1) || is_quote(&para[0].1))
        {
            flush(&mut out, &mut para);
        }
        para.push((n, line.to_string()));
    }
    flush(&mut out, &mut para);
    out
}

// ─── §5.3 Summary ────────────────────────────────────────────────────────────

static IMAGE_INLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[?!\[(?:[^\]\\]|\\.)*\]\([^)]*\)(?:\]\([^)]*\))?").unwrap());
static IMAGE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[?!\[(?:[^\]\\]|\\.)*\]\[[^\]]*\](?:\]\([^)]*\))?").unwrap());

/// A paragraph that is nothing but images: a banner, a screenshot, badges.
pub fn is_picture(lines: &[(usize, String)]) -> bool {
    let text = lines
        .iter()
        .map(|(_, l)| l.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let text = IMAGE_INLINE.replace_all(&text, "");
    let text = IMAGE_REF.replace_all(&text, "");
    text.trim_matches(WS).is_empty()
}

/// A paragraph that is prose, not a list, a quote, a table or HTML.
pub fn is_summary_candidate(lines: &[(usize, String)]) -> bool {
    let first = &lines[0].1;
    if is_list(first) || first.trim_start_matches(WS).starts_with(['>', '|', '<']) {
        return false;
    }
    !lines.iter().any(|(_, l)| REFDEF.is_match(l))
}

/// Spec §5.3: the summary, from the blocks after the title heading.
pub fn summary(blocks: &[Block]) -> Option<String> {
    for block in blocks {
        if let Block::Paragraph { lines } = block {
            if is_picture(lines) {
                continue;
            }
            if is_summary_candidate(lines) {
                return Some(
                    lines
                        .iter()
                        .map(|(_, l)| l.trim_matches(WS))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        return None;
    }
    None
}

// ─── §6.3 Anchors ────────────────────────────────────────────────────────────

static H_IMAGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!\[(?:[^\]\\]|\\.)*\]\([^)]*\)|!\[(?:[^\]\\]|\\.)*\]\[[^\]]*\]").unwrap()
});
static H_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[((?:[^\]\\]|\\.)*)\](?:\([^)]*\)|\[[^\]]*\])").unwrap());
static H_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>\n]*>").unwrap());
static ESCAPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\([!-/:-@\[-`{-~])").unwrap());

/// The text of a heading as it renders, markup removed (spec §6.3, steps 1–6).
pub fn heading_text(source: &str) -> String {
    let t = H_IMAGE.replace_all(source, "");
    let t = H_LINK.replace_all(&t, "$1");
    let t = H_TAG.replace_all(&t, "");
    let t = ESCAPE.replace_all(&t, "$1");
    let t = strip_emphasis_underscores(&t);
    html_escape::decode_html_entities(&t).into_owned()
}

/// Step 5: a run of `_` with a letter or digit on one side and not the other.
fn strip_emphasis_underscores(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let alnum = |c: Option<&char>| {
        c.is_some_and(|c| {
            matches!(
                c.general_category_group(),
                unicode_properties::GeneralCategoryGroup::Letter
                    | unicode_properties::GeneralCategoryGroup::Number
            )
        })
    };
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '_' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i] == '_' {
            i += 1;
        }
        let before = if start == 0 {
            None
        } else {
            chars.get(start - 1)
        };
        let after = chars.get(i);
        if alnum(before) != alnum(after) {
            continue;
        }
        out.extend(&chars[start..i]);
    }
    out
}

/// A word character in the sense of Unicode TS #18, Annex C.
fn is_word(c: char) -> bool {
    if c.is_alphabetic() || c == '\u{200c}' || c == '\u{200d}' {
        return true;
    }
    matches!(
        c.general_category(),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
            | GeneralCategory::DecimalNumber
            | GeneralCategory::ConnectorPunctuation
    )
}

/// Steps 7–9: GitHub's slug.
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|&c| c == '-' || c == ' ' || is_word(c))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

static TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<([A-Za-z][A-Za-z0-9-]*)[^>]*>").unwrap());
static ID_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)[ \t\n]id[ \t]*=[ \t]*(?:"([^"]*)"|'([^']*)')"#).unwrap());
static ID_OR_NAME_ATTR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)[ \t\n](?:id|name)[ \t]*=[ \t]*(?:"([^"]*)"|'([^']*)')"#).unwrap()
});

/// Spec §6.3: heading slugs made unique as GitHub does, then HTML ids.
pub fn anchors(blocks: &[Block]) -> Vec<String> {
    let mut seen: std::collections::HashMap<String, usize> = Default::default();
    let mut result = Vec::new();
    for block in blocks {
        if let Block::Heading { text, .. } = block {
            let base = slugify(&heading_text(text));
            let mut slug = base.clone();
            while seen.contains_key(&slug) {
                let count = seen.get_mut(&base).expect("the original is recorded first");
                *count += 1;
                slug = format!("{base}-{count}");
            }
            seen.insert(slug.clone(), 0);
            result.push(slug);
        }
    }
    for block in blocks {
        let lines: Vec<String> = match block {
            Block::Heading { text, .. } => vec![text.clone()],
            Block::Paragraph { lines } => lines.iter().map(|(_, l)| l.clone()).collect(),
            _ => continue,
        };
        let text = lines
            .iter()
            .map(|l| mask_code_spans(l))
            .collect::<Vec<_>>()
            .join("\n");
        for tag in TAG.captures_iter(&text) {
            let attrs = if tag[1].eq_ignore_ascii_case("a") {
                &*ID_OR_NAME_ATTR
            } else {
                &*ID_ATTR
            };
            for m in attrs.captures_iter(&tag[0]) {
                let value = m.get(1).or(m.get(2)).map_or("", |g| g.as_str()).to_string();
                if !result.contains(&value) {
                    result.push(value);
                }
            }
        }
    }
    result
}

// ─── §6.1 Finding links ──────────────────────────────────────────────────────

/// A link as written: where its destination sits in the line, so that a writer
/// can replace exactly those bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct RawLink {
    pub line: usize,
    /// The destination as written, backslash escapes and all, without `<>`.
    pub destination: String,
    /// Byte range of `destination` within the line.
    pub start: usize,
    pub end: usize,
    pub angle: bool,
}

/// Replace each code span with spaces, byte for byte, so offsets survive.
pub fn mask_code_spans(line: &str) -> String {
    let b = line.as_bytes();
    let mut out = b.to_vec();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            i += 2;
            continue;
        }
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < b.len() && b[j] == b'`' {
            j += 1;
        }
        let run = j - i;
        // The next run of exactly the same length.
        let mut k = j;
        let mut close = None;
        while k < b.len() {
            if b[k] == b'`' {
                let s = k;
                while k < b.len() && b[k] == b'`' {
                    k += 1;
                }
                if k - s == run {
                    close = Some(k);
                    break;
                }
            } else {
                k += 1;
            }
        }
        match close {
            Some(end) => {
                for byte in &mut out[i..end] {
                    if *byte != b'\n' {
                        *byte = b' ';
                    }
                }
                i = end;
            }
            None => i = j,
        }
    }
    // Only ASCII bytes were replaced with ASCII, and whole multi-byte
    // characters inside a span became spaces, so this is valid UTF-8.
    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

static TITLE_THEN_CLOSE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^[ \t]*(?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\((?:[^()\\]|\\.)*\))?[ \t]*\)"#)
        .unwrap()
});

/// Inline links and images in `text` — one heading, or one paragraph's lines
/// joined with `\n`, code spans already masked. A link's text may run over
/// several lines; its destination is on one, and that is the link's line.
/// `lines` gives each line's number and its byte offset in `text`.
fn inline_links(text: &str, lines: &[(usize, usize)], out: &mut Vec<RawLink>) {
    let b = text.as_bytes();
    let locate = |offset: usize| {
        let i = lines.partition_point(|&(_, start)| start <= offset) - 1;
        lines[i]
    };
    let mut i = 0;
    while let Some(found) = text[i..].find("](") {
        let close = i + found;
        // The `]` must close an earlier `[`, brackets balanced.
        let mut depth = 0i32;
        let mut j = close as isize;
        let mut opened = false;
        while j >= 0 {
            let c = b[j as usize];
            let escaped = j > 0 && b[j as usize - 1] == b'\\';
            if c == b']' && !escaped {
                depth += 1;
            } else if c == b'[' && !escaped {
                depth -= 1;
                if depth == 0 {
                    opened = true;
                    break;
                }
            }
            j -= 1;
        }
        let mut k = close + 2;
        if !opened {
            i = k;
            continue;
        }
        while k < b.len() && (b[k] == b' ' || b[k] == b'\t') {
            k += 1;
        }
        let (start, end, rest, angle);
        if k < b.len() && b[k] == b'<' {
            match text[k..].find(['>', '\n']) {
                Some(e) if b[k + e] == b'>' => {
                    start = k + 1;
                    end = k + e;
                    rest = end + 1;
                    angle = true;
                }
                _ => {
                    i = k;
                    continue;
                }
            }
        } else {
            let mut parens = 0;
            start = k;
            while k < b.len() {
                let c = b[k];
                if c == b'\\' && k + 1 < b.len() && b[k + 1] != b'\n' {
                    k += 2;
                    continue;
                }
                if c == b' ' || c == b'\t' || c == b'\n' {
                    break;
                }
                if c == b'(' {
                    parens += 1;
                } else if c == b')' {
                    if parens == 0 {
                        break;
                    }
                    parens -= 1;
                }
                k += 1;
            }
            end = k.min(b.len());
            rest = end;
            angle = false;
        }
        let tail_end = text[rest..].find('\n').map_or(b.len(), |e| rest + e);
        if TITLE_THEN_CLOSE.is_match(&text[rest..tail_end]) && end > start {
            let (line, line_start) = locate(start);
            out.push(RawLink {
                line,
                destination: text[start..end].to_string(),
                start: start - line_start,
                end: end - line_start,
                angle,
            });
        }
        i = rest.max(close + 2);
    }
}

/// Spec §6.1: every link in the blocks that are not code.
pub fn links(blocks: &[Block], heading_lines: &dyn Fn(usize) -> Option<String>) -> Vec<RawLink> {
    let mut out = Vec::new();
    for block in blocks {
        match block {
            Block::Heading { line, .. } => {
                // Found in the heading's source line, so offsets are offsets
                // into the file. A setext heading's first line stands for it.
                if let Some(source) = heading_lines(*line) {
                    inline_links(&mask_code_spans(&source), &[(*line, 0)], &mut out);
                }
            }
            Block::Paragraph { lines } => {
                let mut joined = String::new();
                let mut starts = Vec::new();
                let mut found = Vec::new();
                for (n, line) in lines {
                    if !joined.is_empty() {
                        joined.push('\n');
                    }
                    starts.push((*n, joined.len()));
                    if let Some(m) = REFDEF.captures(line) {
                        let g = m.get(1).expect("the destination group always matches");
                        let (start, end, angle) = if g.as_str().starts_with('<') {
                            (g.start() + 1, g.end() - 1, true)
                        } else {
                            (g.start(), g.end(), false)
                        };
                        if end > start {
                            found.push(RawLink {
                                line: *n,
                                destination: line[start..end].to_string(),
                                start,
                                end,
                                angle,
                            });
                        }
                        // A definition is not searched for inline links.
                        joined.push_str(&" ".repeat(line.len()));
                    } else {
                        joined.push_str(line);
                    }
                }
                inline_links(&mask_code_spans(&joined), &starts, &mut found);
                // In the order they appear: by line, then left to right.
                found.sort_by_key(|l| (l.line, l.start));
                out.extend(found);
            }
            _ => {}
        }
    }
    out
}

/// A backslash before ASCII punctuation is removed before a destination is
/// resolved (spec §6.1).
pub fn unescape(destination: &str) -> String {
    ESCAPE.replace_all(destination, "$1").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_match_github() {
        for (heading, slug) in [
            ("Café résumé", "café-résumé"),
            ("Roadmaps — design rationale", "roadmaps--design-rationale"),
            ("100% ✓ done 🎉", "100--done-"),
            ("C++ / C#", "c--c"),
            (
                "*emph* and _under_ and snake_case_name and __init__",
                "emph-and-under-and-snake_case_name-and-init",
            ),
            ("a &amp; b &#233;", "a--b-é"),
            ("[link text](x.md) and ![image](x.png)", "link-text-and-"),
            ("Über 日本語 ١٢٣ Ⓐ ²", "über-日本語-١٢٣-ⓐ-"),
            ("e\\_scaped \\* chars", "e_scaped--chars"),
        ] {
            assert_eq!(slugify(&heading_text(heading)), slug, "{heading}");
        }
    }

    #[test]
    fn duplicate_slugs() {
        let b = blocks(
            "# Hello World\n# Hello World\n# Hello World-1\n# Hello World\n",
            1,
        );
        assert_eq!(
            anchors(&b),
            [
                "hello-world",
                "hello-world-1",
                "hello-world-1-1",
                "hello-world-2"
            ]
        );
    }

    #[test]
    fn link_spans_point_at_the_destination() {
        let b = blocks(
            "See [a](one.md#x) and `[b](no.md)` and [c](<two words.md>).",
            1,
        );
        let l = links(&b, &|_| None);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].destination, "one.md#x");
        assert_eq!(l[1].destination, "two words.md");
        assert!(l[1].angle);
    }
}

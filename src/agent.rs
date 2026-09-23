// loam — whether a person or an agent is writing, and the likeness of pages.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

/// The agent loam is acting for, if any.
///
/// Declared first: `--agent NAME`, then `LOAM_AGENT`, then `CAIRN_AGENT`, so
/// one variable serves both programs. Then recognised: `AI_AGENT`, which agents
/// have begun to set for the programs they run, and Claude Code's `CLAUDECODE`.
///
/// cairn declines to detect at all, on the ground that a program that wants to
/// be treated as a person simply is one. That is as true here — this is a
/// guard rail, not a boundary — but a guard rail an agent must remember to
/// switch on is mostly not there, and the cost of a false detection is a draft
/// a person promotes with one command.
pub fn acting(flag: Option<&str>) -> Option<String> {
    let clean = |s: &str| {
        let s: String = s
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
            .take(64)
            .collect();
        Some(s).filter(|s| !s.is_empty())
    };
    if let Some(f) = flag {
        return clean(f);
    }
    for var in ["LOAM_AGENT", "CAIRN_AGENT", "AI_AGENT"] {
        if let Ok(v) = std::env::var(var)
            && let Some(name) = clean(&v)
        {
            return Some(name);
        }
    }
    std::env::var("CLAUDECODE")
        .ok()
        .filter(|v| v == "1")
        .map(|_| "claude-code".to_string())
}

// ─── Likeness (0048) ─────────────────────────────────────────────────────────

const STOP: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "can", "do", "does", "for", "from", "how",
    "in", "into", "is", "it", "its", "not", "of", "on", "or", "our", "that", "the", "their",
    "this", "to", "we", "what", "when", "where", "which", "why", "with", "without", "you", "your",
    "about", "notes", "note", "page",
];

/// A word reduced to a common stem, crudely: enough that "parsers" and
/// "parse", "crates" and "crate", "rendering" and "render" meet.
fn stem(word: &str) -> String {
    let mut w = word.to_string();
    for suffix in ["ings", "ing", "ers", "er", "ies", "es", "ed", "s"] {
        if w.len() > suffix.len() + 2 && w.ends_with(suffix) {
            w.truncate(w.len() - suffix.len());
            if suffix == "ies" {
                w.push('y');
            }
            break;
        }
    }
    if w.len() > 3 && w.ends_with('e') {
        w.pop();
    }
    w
}

pub fn words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() > 1 && !STOP.contains(w))
        .map(stem)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// How much of a new title an existing page already says: the words they
/// share, when they share enough to be worth a look. Plain overlap of words,
/// no embeddings (the boundary, cairn 0019).
///
/// `common` are words so frequent among the pages compared that sharing one says
/// nothing: a project's own name, or the thing every page is about.
pub fn likeness(
    title: &str,
    other_title: &str,
    other_summary: &str,
    common: &[String],
) -> Option<Vec<String>> {
    // The same title is the same page, however common its words.
    if !words(title).is_empty() && words(title) == words(other_title) {
        return Some(words(title));
    }
    let new: Vec<String> = words(title)
        .into_iter()
        .filter(|w| !common.contains(w))
        .collect();
    if new.is_empty() {
        return None;
    }
    let theirs_title = words(other_title);
    let mut theirs = theirs_title.clone();
    theirs.extend(words(other_summary));
    let shared: Vec<String> = new.iter().filter(|w| theirs.contains(w)).cloned().collect();
    let same_title = new == theirs_title;
    // Two words in common and at least half the new title, or the same words.
    (same_title || (shared.len() >= 2 && shared.len() * 2 >= new.len())).then_some(shared)
}

/// Words in at least a third of these pages, when there are enough of them to
/// say so.
pub fn common(pages: &[(String, String)]) -> Vec<String> {
    if pages.len() < 3 {
        return Vec::new();
    }
    let mut counts: std::collections::HashMap<String, usize> = Default::default();
    for (title, summary) in pages {
        let mut w = words(title);
        w.extend(words(summary));
        w.sort();
        w.dedup();
        for word in w {
            *counts.entry(word).or_default() += 1;
        }
    }
    let mut out: Vec<String> = counts
        .into_iter()
        .filter(|(_, n)| n * 3 >= pages.len())
        .map(|(w, _)| w)
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_example_pair_is_alike() {
        let shared = likeness(
            "Rust Markdown parsers",
            "Markdown crates",
            "Which Rust crates parse Markdown, and what each costs to embed.",
            &[],
        );
        assert_eq!(
            shared,
            Some(vec!["markdown".into(), "pars".into(), "rust".into()])
        );
    }

    #[test]
    fn a_word_every_page_shares_is_not_evidence() {
        let pages: Vec<(String, String)> = ["Style guide", "Build", "Contributing", "Releases"]
            .iter()
            .map(|t| (t.to_string(), "For The Measure of the World.".to_string()))
            .collect();
        let common = common(&pages);
        assert_eq!(common, ["measur", "world"]);
        assert_eq!(
            likeness(
                "Style Guide: The Measure of the World",
                "Build",
                "For The Measure of the World.",
                &common
            ),
            None
        );
    }

    #[test]
    fn unrelated_pages_are_not() {
        assert_eq!(
            likeness(
                "Rendering tables",
                "Markdown crates",
                "Which Rust crates parse Markdown.",
                &[]
            ),
            None
        );
        assert_eq!(
            likeness("How it works", "How replay works", "", &[]),
            None,
            "stop words and one shared word"
        );
    }
}

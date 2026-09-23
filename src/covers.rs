// loam — which files a page covers (spec §4.3).
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.

/// A page's `covers`, compiled.
#[derive(Clone, Debug, Default)]
pub struct Covers {
    include: Vec<Vec<String>>,
    exclude: Vec<Vec<String>>,
    /// The patterns as written, for messages.
    pub patterns: Vec<String>,
}

fn segments(pattern: &str) -> Vec<String> {
    pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .map(str::to_string)
        .collect()
}

impl Covers {
    pub fn new(patterns: &[String]) -> Covers {
        let mut c = Covers {
            patterns: patterns.to_vec(),
            ..Covers::default()
        };
        for p in patterns {
            match p.strip_prefix('!') {
                Some(rest) => c.exclude.push(segments(rest)),
                None => c.include.push(segments(p)),
            }
        }
        c
    }

    /// The including patterns as git pathspecs: each as a glob, and as a
    /// directory whose contents it covers. Exclusions are left to `covers`.
    pub fn pathspecs(&self) -> Vec<String> {
        let mut out = Vec::new();
        for seg in &self.include {
            let glob = seg
                .iter()
                .map(|s| s.replace('\\', "\\\\").replace('[', "\\["))
                .collect::<Vec<_>>()
                .join("/");
            out.push(format!(":(glob){glob}"));
            out.push(format!(":(glob){glob}/**"));
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.include.is_empty()
    }

    pub fn covers(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('/').collect();
        self.include.iter().any(|p| matches(p, &parts))
            && !self.exclude.iter().any(|p| matches(p, &parts))
    }

    /// The including pattern, as written, that covers `path`, if any.
    pub fn which(&self, path: &str) -> Option<&str> {
        if !self.covers(path) {
            return None;
        }
        let parts: Vec<&str> = path.split('/').collect();
        let includes = self.patterns.iter().filter(|p| !p.starts_with('!'));
        includes
            .zip(&self.include)
            .find(|(_, seg)| matches(seg, &parts))
            .map(|(p, _)| p.as_str())
    }

    /// Each including pattern as written, and whether any of `paths` matches it.
    pub fn unmatched<'a>(&'a self, paths: &[String]) -> Vec<&'a str> {
        let includes = self.patterns.iter().filter(|p| !p.starts_with('!'));
        includes
            .zip(&self.include)
            .filter(|(_, seg)| {
                !paths
                    .iter()
                    .any(|f| matches(seg, &f.split('/').collect::<Vec<_>>()))
            })
            .map(|(p, _)| p.as_str())
            .collect()
    }
}

/// A pattern matches a path, or a directory the path is below.
fn matches(pattern: &[String], path: &[&str]) -> bool {
    // Every prefix of the path is a candidate: matching a directory covers
    // everything under it.
    (1..=path.len()).any(|n| whole(pattern, &path[..n]))
}

fn whole(pattern: &[String], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((first, rest)) if first == "**" => (0..=path.len()).any(|i| whole(rest, &path[i..])),
        Some((first, rest)) => match path.split_first() {
            Some((seg, tail)) => segment(first, seg) && whole(rest, tail),
            None => false,
        },
    }
}

/// `*` any run of characters, `?` exactly one, anything else itself.
fn segment(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(p: &[&str]) -> Covers {
        Covers::new(&p.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn the_rules_of_spec_4_3() {
        let covers = c(&[
            "src/engine",
            "src/**/*.rs",
            "!src/engine/tests",
            "/Cargo.toml",
            "docs/?.md",
        ]);
        assert!(covers.covers("src/engine/step.rs"));
        assert!(
            covers.covers("src/engine/deep/x.txt"),
            "below a directory it names"
        );
        assert!(covers.covers("src/main.rs"), "** matches no segments");
        assert!(covers.covers("src/a/b/c.rs"));
        assert!(!covers.covers("src/a/b/c.txt"));
        assert!(!covers.covers("src/engine/tests/t.rs"), "uncovered by !");
        assert!(covers.covers("Cargo.toml"), "a leading / is ignored");
        assert!(covers.covers("docs/a.md"));
        assert!(!covers.covers("docs/ab.md"));
        assert!(!covers.covers("src/engineering.rs.bak"));
        assert!(!covers.covers("srcx/engine/a.rs"));
        assert_eq!(covers.which("src/engine/step.rs"), Some("src/engine"));
        assert_eq!(
            covers.unmatched(&["src/engine/a.rs".into()]),
            ["/Cargo.toml", "docs/?.md"]
        );
        // `*` stays within a segment, but a segment it matches may be a
        // directory, and then everything below it is covered.
        assert!(c(&["*"]).covers("a/b"));
        assert!(!c(&["*.md"]).covers("a/b.md"));
    }
}

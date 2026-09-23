// loam check — report what is wrong with the docs, each with a path and line.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Two sources of findings. The format's own (spec §6.5), which every conforming
// reader reports the same way. And loam's judgements, which the spec leaves to
// a program: a superseded page that names no successor, a link still pointing
// at a superseded page, an index that has drifted.
//
// Exit status: 0 when nothing fails; 1 when a finding is an error, or a
// warning under --strict; 2 when the check could not run at all.

use super::Ctx;
use crate::tree::{Finding, Tree};
use anyhow::Result;
use serde_json::json;

#[derive(clap::Args)]
pub struct Args {
    /// Fail on warnings as well as errors
    #[arg(short, long)]
    pub strict: bool,

    /// Also check that the index is what `loam render` would write
    #[arg(long)]
    pub render: bool,

    /// Print the findings as JSON on standard output
    #[arg(long)]
    pub json: bool,

    /// Print nothing when everything passes
    #[arg(short, long)]
    pub quiet: bool,
}

/// Findings that are errors unless the project says otherwise: a page that
/// cannot be read at all, and an index that does not match its pages.
const ERRORS: &[&str] = &["invalid-encoding", "malformed-frontmatter", "stale-index"];

pub struct Reported {
    pub path: String,
    pub finding: Finding,
    pub severity: String,
}

/// loam's own findings, beyond the format's (spec §6.5 permits them).
pub fn judgements(tree: &Tree) -> Vec<(String, Finding)> {
    let mut out = Vec::new();
    for (path, page) in &tree.pages {
        if page.status == "superseded"
            && page.status_from == "frontmatter"
            && page.superseded_by.is_empty()
        {
            out.push((
                path.clone(),
                Finding {
                    line: 1,
                    code: "superseded-without-successor",
                    detail: None,
                },
            ));
        }
        if !page.superseded_by.is_empty()
            && page.status_from == "frontmatter"
            && page.status != "superseded"
        {
            out.push((
                path.clone(),
                Finding {
                    line: 1,
                    code: "successor-without-superseded",
                    detail: Some(page.status.clone()),
                },
            ));
        }
        if *path == tree.config.index {
            continue;
        }
        for link in &page.links {
            let Some(target) = &link.target else { continue };
            let Some(other) = tree.pages.get(target) else {
                continue;
            };
            if other.status != "superseded" || target == path {
                continue;
            }
            // The successor names what it replaced, and should.
            let successor = other
                .superseded_by
                .iter()
                .any(|v| crate::tree::resolve(target, v).0.as_deref() == Some(path));
            if !successor {
                out.push((
                    path.clone(),
                    Finding {
                        line: link.raw.line,
                        code: "link-to-superseded",
                        detail: Some(link.raw.destination.clone()),
                    },
                ));
            }
        }
    }
    out
}

pub fn message(f: &Finding) -> String {
    let d = f.detail.as_deref().unwrap_or("");
    match f.code {
        "superseded-without-successor" => {
            "superseded, but `superseded_by` names nothing to read instead".into()
        }
        "successor-without-superseded" => {
            format!("`superseded_by` names a successor, but status is `{d}`")
        }
        "link-to-superseded" => {
            format!("link to `{d}`, which is superseded; link to what replaced it")
        }
        "stale-index" => "the index is not what `loam render` would write; run it".into(),
        _ => f.message(),
    }
}

pub fn collect(tree: &Tree, render: bool) -> Result<Vec<Reported>> {
    let mut all: Vec<(String, Finding)> = Vec::new();
    for (path, page) in &tree.pages {
        all.extend(page.findings.iter().map(|f| (path.clone(), f.clone())));
    }
    all.extend(judgements(tree));
    if render && !super::render::is_current(tree)? {
        all.push((
            tree.config.index.clone(),
            Finding {
                line: 1,
                code: "stale-index",
                detail: None,
            },
        ));
    }
    all.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    Ok(all
        .into_iter()
        .filter_map(|(path, finding)| {
            let severity = tree
                .config
                .severity
                .get(finding.code)
                .cloned()
                .unwrap_or_else(|| {
                    if ERRORS.contains(&finding.code) {
                        "error"
                    } else {
                        "warning"
                    }
                    .into()
                });
            (severity != "ignore").then_some(Reported {
                path,
                finding,
                severity,
            })
        })
        .collect())
}

/// For a link broken only by the case of its letters, the file it meant:
/// it works on a Mac and fails on GitHub, which is the hardest kind to see.
fn case_hint(tree: &Tree, path: &str, f: &Finding) -> Option<String> {
    if f.code != "broken-link" {
        return None;
    }
    let target = crate::tree::resolve(path, f.detail.as_deref()?).0?;
    let (dir, name) = target.rsplit_once('/').unwrap_or(("", &target));
    let entries = std::fs::read_dir(tree.config.abs(dir)).ok()?;
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n != name && n.eq_ignore_ascii_case(name))
        .map(|n| format!("; `{n}` differs only in case"))
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let tree = ctx.tree()?;
    let found = collect(&tree, args.render)?;
    let errors = found.iter().filter(|r| r.severity == "error").count();
    let warnings = found.len() - errors;
    let failed = errors > 0 || (args.strict && warnings > 0);

    if args.json {
        let list: Vec<_> = found
            .iter()
            .map(|r| {
                json!({
                    "path": r.path,
                    "line": r.finding.line,
                    "code": r.finding.code,
                    "severity": r.severity,
                    "detail": r.finding.detail,
                    "message": message(&r.finding) + &case_hint(&tree, &r.path, &r.finding).unwrap_or_default(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(u8::from(failed));
    }

    // `loam: file:line: message`, the GNU shape cairn uses, so an editor's
    // matcher for one works for the other.
    for r in &found {
        let label = if r.severity == "error" {
            crate::style::red("loam")
        } else {
            crate::style::yellow("loam")
        };
        eprintln!(
            "{label}: {}:{}: {} [{}]",
            r.path,
            r.finding.line,
            message(&r.finding) + &case_hint(&tree, &r.path, &r.finding).unwrap_or_default(),
            r.finding.code
        );
    }
    let pages = tree.pages.len();
    if failed {
        eprintln!();
        eprintln!(
            "{} {errors} error(s), {warnings} warning(s) across {pages} page(s)",
            crate::style::red("failed:")
        );
    } else if !args.quiet {
        println!(
            "{} {pages} page(s), {warnings} warning(s)",
            crate::style::green("ok:")
        );
    }
    Ok(u8::from(failed))
}

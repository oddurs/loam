// loam — the reading of a repository, as spec/corpus/README.md shapes it.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// `loam reading` exists so the conformance corpus can hold loam to the same
// expectations as the reader in spec/: `make conformance` runs both.

use crate::tree::{Page, Tree};
use serde_json::{Value, json};

pub fn page_json(page: &Page) -> Value {
    json!({
        "frontmatter": page.frontmatter.as_ref().map(|m| Value::Object(
            m.iter().map(|(k, v)| (k.clone(), v.to_json())).collect()
        )),
        "title": page.title,
        "title_from": page.title_from,
        "kind": page.kind,
        "kind_from": page.kind_from,
        "status": page.status,
        "status_from": page.status_from,
        "summary": page.summary,
        "summary_from": page.summary_from,
        "order": page.order,
        "covers": page.covers,
        "reviewed": page.reviewed.as_ref().map(|r| json!({"commit": r.commit, "date": r.date})),
        "generated": page.generated,
        "supersedes": page.supersedes,
        "superseded_by": page.superseded_by,
        "anchors": page.anchors,
        "links": page.links.iter().map(|l| {
            let mut o = json!({
                "line": l.raw.line,
                "destination": l.raw.destination,
                "class": l.class.name(),
                "target": l.target,
                "fragment": l.fragment,
            });
            if let Some(item) = l.item {
                o["item"] = json!(item);
            }
            if let Some(current) = &l.current {
                o["current"] = json!(current);
            }
            o
        }).collect::<Vec<_>>(),
        "findings": page.findings.iter().map(|f| json!({
            "line": f.line, "code": f.code, "detail": f.detail,
        })).collect::<Vec<_>>(),
    })
}

pub fn reading(tree: &Tree) -> Value {
    json!({
        "format": crate::config::FORMAT,
        "pages": tree.pages.iter().map(|(p, page)| (p.clone(), page_json(page))).collect::<serde_json::Map<_, _>>(),
    })
}

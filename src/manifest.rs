// loam — the manifest: the whole docs tree as one document (0051, 0052).
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Everything a site needs to be built from a docs folder, so it reads nothing
// else but the pages' bodies: each page's reading, exactly as the conformance
// corpus shapes it, and what loam knows beyond one page — the index's sections
// and order, backlinks, headings with their anchors, freshness. Specified in
// spec/manifest.md, with a JSON Schema beside it; `manifest_version` changes
// only for a change a reader of the old one could misread.

use crate::cmd::stale::freshness_json;
use crate::fresh::Freshness;
use crate::tree::{LinkClass, Tree};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

pub const MANIFEST_VERSION: u32 = 1;

/// What was learned from git, or why nothing was.
pub struct History {
    pub pages: HashMap<String, Freshness>,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

pub fn manifest(tree: &Tree, history: &History) -> Value {
    let backlinks = tree.backlinks_all();
    let mut pages = Map::new();
    for (path, page) in &tree.pages {
        let mut v = crate::reading::page_json(page);
        let items: std::collections::BTreeSet<u64> = page
            .links
            .iter()
            .filter(|l| l.class == LinkClass::Cairn)
            .filter_map(|l| l.item)
            .collect();
        v["path"] = json!(path);
        v["body_line"] = json!(page.body_line);
        v["lead"] = json!(crate::index::lead_of(page));
        v["headings"] = json!(page.headings);
        v["backlinks"] = json!(backlinks.get(path).cloned().unwrap_or_default());
        v["items"] = json!(items);
        v["freshness"] = history.pages.get(path).map_or(Value::Null, freshness_json);
        pages.insert(path.clone(), v);
    }
    let config = &tree.config;
    json!({
        "manifest_version": MANIFEST_VERSION,
        "format": crate::config::FORMAT,
        "generator": format!("loam {}", env!("CARGO_PKG_VERSION")),
        "docs": config.docs,
        "index": config.index,
        "cairn": config.cairn,
        "kinds": config.kinds.iter().map(|k| json!({
            "name": k.name,
            "dir": crate::config::join(&config.docs, &k.dir),
            "heading": k.heading(),
            "description": k.description.as_deref().map(str::trim).filter(|d| !d.is_empty()),
            "indexed": k.index,
        })).collect::<Vec<_>>(),
        "sections": crate::index::sections(tree).iter().map(|s| json!({
            "role": s.role,
            "kind": s.kind,
            "heading": s.heading,
            "description": s.description,
            "pages": s.pages.iter().map(|p| &p.path).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "pages": pages,
        "freshness": {
            "notes": history.notes,
            "error": history.error,
        },
    })
}

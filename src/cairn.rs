// loam — what cairn says about the items pages cite (0054).
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// A page cites an item by a link to its file, and loam resolves the link by
// the number the file name begins with (spec §6.4): that needs no cairn. What
// the item *is* — its title, whether it is still open — is cairn's to say, so
// loam asks it, once, as cairn's manual prescribes for any program reading a
// backlog: `cairn list --all --json`, never the item files parsed again.

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub title: String,
    pub status: String,
    /// cairn's grouping of the status: `open`, `active`, `done` or `dropped`.
    pub category: String,
    #[serde(rename = "type")]
    pub kind: String,
}

/// Every item cairn knows in the repository at `root`, by number; or why
/// cairn could not be asked.
pub fn items(root: &Path) -> Result<BTreeMap<u64, Item>, String> {
    let out = Command::new("cairn")
        .args(["list", "--all", "--json"])
        .current_dir(root)
        .output()
        .map_err(|_| "cairn is not installed, so the items cited are unchecked".to_string())?;
    if !out.status.success() {
        return Err(format!(
            "cairn list failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let list: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("cairn list printed something other than a list: {e}"))?;
    let text = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or("").to_string();
    Ok(list
        .iter()
        .filter_map(|v| {
            Some((
                v["id"].as_u64()?,
                Item {
                    title: text(v, "title"),
                    status: text(v, "status"),
                    category: text(v, "category"),
                    kind: text(v, "type"),
                },
            ))
        })
        .collect())
}

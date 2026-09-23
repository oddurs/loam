// loam set — change a page's frontmatter from the command line.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// The same surgical edit `supersede` and `review` make, for any key: the one
// key named changes, and no other byte does. The format's own keys are held to
// their types (spec §4), so `loam set` cannot write what `loam check` would
// then report; any other key is the project's, and is written as given.

use super::Ctx;
use crate::tree::resolve;
use crate::write::{Lines, Value, remove_key, scalar, set_key, write_atomic};
use crate::yaml::{self, Yaml};
use anyhow::{Result, bail};

#[derive(clap::Args)]
pub struct Args {
    /// The page
    pub page: String,
    /// `key=value` to set, `key=` to remove, `key+=value` and `key-=value` to
    /// add to or take from a list
    #[arg(required = true, value_name = "KEY=VALUE")]
    pub assignments: Vec<String>,
}

const LISTS: &[&str] = &["covers", "supersedes", "superseded_by"];
const STRINGS: &[&str] = &["title", "summary", "generated", "kind", "status"];

enum Op {
    Set(String),
    Remove,
    Add(String),
    Take(String),
}

fn parse(assignment: &str) -> Result<(String, Op)> {
    let Some(eq) = assignment.find('=') else {
        bail!("`{assignment}` is not KEY=VALUE; use `{assignment}=` to remove a key");
    };
    let (key, value) = (&assignment[..eq], assignment[eq + 1..].to_string());
    let (key, op) = if let Some(k) = key.strip_suffix('+') {
        (k, Op::Add(value))
    } else if let Some(k) = key.strip_suffix('-') {
        (k, Op::Take(value))
    } else if value.is_empty() {
        (key, Op::Remove)
    } else {
        (key, Op::Set(value))
    };
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        bail!("`{key}` is not a key a page can have: letters, digits, `_` and `-`");
    }
    Ok((key.to_string(), op))
}

/// A value for a key the format does not define, written as the person typed
/// it: `3` stays a number, `true` a boolean, and anything else a string.
fn typed(value: &str) -> String {
    match yaml::resolve(value) {
        Yaml::Str(_) => scalar(value),
        _ => value.to_string(),
    }
}

pub fn run(ctx: &Ctx, args: Args) -> Result<u8> {
    let (lock, tree) = ctx.locked_tree()?;
    let path = ctx.repo_path(&tree.config, &args.page)?;
    let Some(page) = tree.pages.get(&path) else {
        bail!("{path} is not a page")
    };
    let Ok(text) = std::str::from_utf8(&page.bytes) else {
        bail!("{path} is not UTF-8, so loam will not rewrite it")
    };
    if !page.is_readable() {
        bail!("{path} cannot be read; run `loam check` to see why");
    }
    let mut lines = Lines::parse(text);
    let mut lists: std::collections::HashMap<String, Vec<String>> = Default::default();
    // A list `+=` and `-=` can edit: absent, one string, or strings. Anything
    // else — numbers, mappings, a mapping where a list was meant — would be
    // rewritten into something it was not, so it is refused.
    let current = |key: &str| -> Result<Vec<String>> {
        match page
            .frontmatter
            .as_ref()
            .and_then(|m| m.iter().find(|(k, _)| k == key))
            .map(|(_, v)| v)
        {
            None | Some(Yaml::Null) => Ok(Vec::new()),
            Some(Yaml::Str(s)) => Ok(vec![s.clone()]),
            Some(Yaml::Seq(items)) if items.iter().all(|i| matches!(i, Yaml::Str(_))) => Ok(items
                .iter()
                .filter_map(|i| match i {
                    Yaml::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .collect()),
            Some(_) => bail!(
                "`{key}` holds more than a list of strings; edit it by hand rather than with += or -="
            ),
        }
    };

    for assignment in &args.assignments {
        let (key, op) = parse(assignment)?;
        if key == "reviewed" {
            bail!("`reviewed` records a reading against a commit; use `loam review {path}`");
        }
        if let Op::Set(v) = &op {
            match key.as_str() {
                "status" if !["draft", "current", "superseded"].contains(&v.as_str()) => {
                    bail!("status is draft, current or superseded, not `{v}`")
                }
                "kind" if tree.config.kind(v).is_none() => {
                    let names: Vec<&str> =
                        tree.config.kinds.iter().map(|k| k.name.as_str()).collect();
                    bail!(
                        "no kind called `{v}`; loam.toml declares: {}",
                        names.join(", ")
                    )
                }
                "order" if v.parse::<i64>().is_err() => bail!("order is a whole number, not `{v}`"),
                _ => {}
            }
        }
        let is_list = LISTS.contains(&key.as_str());
        match op {
            Op::Remove => {
                remove_key(&mut lines, &key);
                lists.remove(&key);
            }
            Op::Set(v) if is_list => {
                lists.insert(key.clone(), vec![v.clone()]);
                set_key(&mut lines, &key, &Value::List(vec![v]));
            }
            Op::Set(v) if STRINGS.contains(&key.as_str()) => {
                set_key(&mut lines, &key, &Value::Str(v))
            }
            Op::Set(v) if key == "order" => set_key(&mut lines, &key, &Value::Raw(v)),
            Op::Set(v) => set_key(&mut lines, &key, &Value::Raw(typed(&v))),
            Op::Add(v) | Op::Take(v) if !is_list && STRINGS.contains(&key.as_str()) => {
                bail!("`{key}` holds one value, not a list; use `{key}={v}`")
            }
            Op::Add(v) => {
                if !lists.contains_key(&key) {
                    lists.insert(key.clone(), current(&key)?);
                }
                let list = lists.get_mut(&key).expect("just inserted");
                if !list.contains(&v) {
                    list.push(v);
                }
                set_key(&mut lines, &key, &Value::List(list.clone()));
            }
            Op::Take(v) => {
                if !lists.contains_key(&key) {
                    lists.insert(key.clone(), current(&key)?);
                }
                let list = lists.get_mut(&key).expect("just inserted");
                // A page is taken out of a supersession however it was spelled.
                let pages = key == "supersedes" || key == "superseded_by";
                let target = resolve(&path, &v).0;
                list.retain(|x| {
                    *x != v && !(pages && target.is_some() && resolve(&path, x).0 == target)
                });
                if list.is_empty() {
                    remove_key(&mut lines, &key);
                } else {
                    set_key(&mut lines, &key, &Value::List(list.clone()));
                }
            }
        }
    }

    let out = lines.render();
    if out.as_bytes() == page.bytes {
        println!("{path} already says so");
        return Ok(0);
    }
    {
        write_atomic(&tree.config.abs(&path), out.as_bytes())?;
    }
    println!("updated {path}");
    drop(lock);
    ctx.after_change(&tree.config);
    Ok(0)
}

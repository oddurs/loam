// loam — frontmatter YAML, resolved by the 1.2 core schema.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// Spec §3.2 requires the YAML 1.2 core schema, and most YAML libraries resolve
// some other way: `no` as false, `012` as octal, a date as a timestamp. So this
// takes only events from the parser — a scalar's text, its style, its tag —
// and does the resolving itself, where the rules can be read in one place.

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser, Tag};
use yaml_rust2::scanner::{Marker, TScalarStyle};

/// A YAML value, keeping the order a mapping was written in.
#[derive(Clone, Debug, PartialEq)]
pub enum Yaml {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Seq(Vec<Yaml>),
    Map(Vec<(String, Yaml)>),
}

impl Yaml {
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::Value;
        match self {
            Yaml::Null => Value::Null,
            Yaml::Bool(b) => Value::Bool(*b),
            Yaml::Int(i) => Value::from(*i),
            Yaml::Float(f) => serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number),
            Yaml::Str(s) => Value::String(s.clone()),
            Yaml::Seq(items) => Value::Array(items.iter().map(Yaml::to_json).collect()),
            Yaml::Map(entries) => Value::Object(
                entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect(),
            ),
        }
    }
}

/// Why frontmatter could not be read as a mapping. The two strings are the
/// details spec/corpus/README.md names.
#[derive(Debug, PartialEq)]
pub enum Problem {
    NotYaml,
    NotAMapping,
}

impl Problem {
    pub fn detail(&self) -> &'static str {
        match self {
            Problem::NotYaml => "not YAML",
            Problem::NotAMapping => "not a mapping",
        }
    }
}

/// Parse frontmatter text into a mapping. Empty or comment-only text is an
/// empty mapping (spec §3.1).
pub fn parse_mapping(text: &str) -> Result<Vec<(String, Yaml)>, Problem> {
    let mut builder = Builder::default();
    let mut parser = Parser::new_from_str(text);
    if parser.load(&mut builder, false).is_err() || builder.failed {
        return Err(Problem::NotYaml);
    }
    match builder.root {
        None | Some(Yaml::Null) => Ok(Vec::new()),
        Some(Yaml::Map(entries)) => Ok(entries),
        Some(_) => Err(Problem::NotAMapping),
    }
}

enum Frame {
    Seq(Vec<Yaml>, usize),
    Map(Vec<(String, Yaml)>, Option<String>, usize),
}

#[derive(Default)]
struct Builder {
    stack: Vec<Frame>,
    anchors: std::collections::HashMap<usize, Yaml>,
    root: Option<Yaml>,
    failed: bool,
    documents: usize,
    /// Values so far, an alias counting as all it stands for (spec §3.2).
    values: usize,
}

/// Spec §3.2's limits: how deep collections nest, and how many values there
/// are once aliases are expanded.
const MAX_DEPTH: usize = 100;
const MAX_VALUES: usize = 10_000;

/// How many values `value` is: itself, and every value inside it.
fn size(value: &Yaml) -> usize {
    1 + match value {
        Yaml::Seq(items) => items.iter().map(size).sum(),
        Yaml::Map(entries) => entries.iter().map(|(_, v)| size(v)).sum(),
        _ => 0,
    }
}

const CORE: &str = "tag:yaml.org,2002:";

/// The core-schema type a tag names, or `Err` for any other tag (spec §3.2).
fn core_tag(tag: &Option<Tag>) -> Result<Option<&str>, ()> {
    let Some(tag) = tag else { return Ok(None) };
    let name = match tag.handle.as_str() {
        "!!" | CORE => tag.suffix.as_str(),
        // `!<tag:yaml.org,2002:str>`, written out in full.
        "" => tag.suffix.strip_prefix(CORE).ok_or(())?,
        _ => return Err(()),
    };
    match name {
        s @ ("null" | "bool" | "int" | "float" | "str" | "seq" | "map") => Ok(Some(s)),
        _ => Err(()),
    }
}

impl Builder {
    /// Add a value, which is `weight` values once aliases are expanded.
    fn push_value(&mut self, value: Yaml, weight: usize) {
        let is_key = matches!(self.stack.last(), Some(Frame::Map(_, None, _)));
        if !is_key {
            self.values += weight;
            if self.values > MAX_VALUES {
                self.failed = true;
                return;
            }
        }
        match self.stack.last_mut() {
            None => self.root = Some(value),
            Some(Frame::Seq(items, _)) => items.push(value),
            Some(Frame::Map(entries, key, _)) => match key.take() {
                None => match key_string(&value) {
                    Some(k) => *key = Some(k),
                    // A sequence or a mapping as a key: PyYAML calls it
                    // unhashable, and nothing a page means can be written so.
                    None => self.failed = true,
                },
                Some(k) => {
                    // A repeated key: the later value wins, in the earlier
                    // key's place, as a mapping in most languages behaves.
                    if let Some(slot) = entries.iter_mut().find(|(e, _)| *e == k) {
                        slot.1 = value;
                    } else {
                        entries.push((k, value));
                    }
                }
            },
        }
    }
}

/// How a key appears once the mapping is JSON: what Python's `json` writes.
fn key_string(value: &Yaml) -> Option<String> {
    Some(match value {
        Yaml::Null => "null".into(),
        Yaml::Bool(b) => b.to_string(),
        Yaml::Int(i) => i.to_string(),
        Yaml::Float(f) => format!("{f:?}"),
        Yaml::Str(s) => s.clone(),
        Yaml::Seq(_) | Yaml::Map(_) => return None,
    })
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, event: Event, _mark: Marker) {
        if self.failed {
            return;
        }
        match event {
            Event::DocumentStart => {
                self.documents += 1;
                if self.documents > 1 {
                    self.failed = true;
                }
            }
            Event::Scalar(text, style, anchor, tag) => {
                let value = match core_tag(&tag) {
                    Err(()) => {
                        self.failed = true;
                        return;
                    }
                    Ok(Some(t)) => match construct(t, &text) {
                        Some(v) => v,
                        None => {
                            self.failed = true;
                            return;
                        }
                    },
                    Ok(None) if style == TScalarStyle::Plain => {
                        // An integer too large for 64 bits (spec §3.2).
                        if parse_int(&text).is_none() && INT.is_match(&text) {
                            self.failed = true;
                            return;
                        }
                        resolve(&text)
                    }
                    Ok(None) => Yaml::Str(text),
                };
                if anchor > 0 {
                    self.anchors.insert(anchor, value.clone());
                }
                self.push_value(value, 1);
            }
            Event::Alias(id) => match self.anchors.get(&id) {
                // Counted before it is copied, so an alias of an alias of an
                // alias cannot fill memory first.
                Some(v) => {
                    let weight = size(v);
                    if self.values + weight > MAX_VALUES {
                        self.failed = true;
                    } else {
                        let v = v.clone();
                        self.push_value(v, weight);
                    }
                }
                None => self.failed = true,
            },
            Event::SequenceStart(..) | Event::MappingStart(..) if self.stack.len() >= MAX_DEPTH => {
                self.failed = true
            }
            Event::SequenceStart(anchor, tag) => match core_tag(&tag) {
                Ok(None | Some("seq")) => self.stack.push(Frame::Seq(Vec::new(), anchor)),
                _ => self.failed = true,
            },
            Event::MappingStart(anchor, tag) => match core_tag(&tag) {
                Ok(None | Some("map")) => self.stack.push(Frame::Map(Vec::new(), None, anchor)),
                _ => self.failed = true,
            },
            Event::SequenceEnd => {
                if let Some(Frame::Seq(items, anchor)) = self.stack.pop() {
                    let v = Yaml::Seq(items);
                    if anchor > 0 {
                        self.anchors.insert(anchor, v.clone());
                    }
                    self.push_value(v, 1);
                }
            }
            Event::MappingEnd => {
                if let Some(Frame::Map(entries, _, anchor)) = self.stack.pop() {
                    let v = Yaml::Map(entries);
                    if anchor > 0 {
                        self.anchors.insert(anchor, v.clone());
                    }
                    self.push_value(v, 1);
                }
            }
            _ => {}
        }
    }
}

/// An untagged plain scalar, by the core schema's regular expressions.
pub fn resolve(text: &str) -> Yaml {
    match text {
        "" | "~" | "null" | "Null" | "NULL" => return Yaml::Null,
        "true" | "True" | "TRUE" => return Yaml::Bool(true),
        "false" | "False" | "FALSE" => return Yaml::Bool(false),
        _ => {}
    }
    if let Some(i) = parse_int(text) {
        return Yaml::Int(i);
    }
    if let Some(f) = parse_float(text) {
        return Yaml::Float(f);
    }
    Yaml::Str(text.to_string())
}

/// The core schema's integer forms, whatever their size.
static INT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"^(?:[-+]?[0-9]+|0o[0-7]+|0x[0-9a-fA-F]+)$").unwrap()
});

/// A scalar with an explicit core tag.
fn construct(tag: &str, text: &str) -> Option<Yaml> {
    Some(match tag {
        "str" => Yaml::Str(text.to_string()),
        "null" => Yaml::Null,
        "bool" => match resolve(text) {
            b @ Yaml::Bool(_) => b,
            _ => return None,
        },
        "int" => Yaml::Int(parse_int(text)?),
        "float" => Yaml::Float(
            parse_int(text)
                .map(|i| i as f64)
                .or_else(|| parse_float(text))?,
        ),
        _ => return None,
    })
}

/// `[-+]?[0-9]+`, `0o[0-7]+` or `0x[0-9a-fA-F]+`. A leading zero is decimal:
/// `012` is twelve, not the octal ten YAML 1.1 made it.
fn parse_int(text: &str) -> Option<i64> {
    if let Some(oct) = text.strip_prefix("0o") {
        if !oct.is_empty() && oct.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
            return i64::from_str_radix(oct, 8).ok();
        }
        return None;
    }
    if let Some(hex) = text.strip_prefix("0x") {
        if !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return i64::from_str_radix(hex, 16).ok();
        }
        return None;
    }
    let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<i64>().ok()
}

/// `[-+]?(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?`, and the infinities
/// and not-a-number.
fn parse_float(text: &str) -> Option<f64> {
    match text {
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => return Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => return Some(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => return Some(f64::NAN),
        _ => {}
    }
    let body = text.strip_prefix(['-', '+']).unwrap_or(text);
    let (mantissa, exponent) = match body.find(['e', 'E']) {
        Some(i) => (&body[..i], Some(&body[i + 1..])),
        None => (body, None),
    };
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    let ok_mantissa = match mantissa.split_once('.') {
        Some((whole, frac)) => {
            digits(whole) && digits(frac) && !(whole.is_empty() && frac.is_empty())
        }
        None => !mantissa.is_empty() && digits(mantissa),
    };
    let ok_exponent = exponent.is_none_or(|e| {
        let d = e.strip_prefix(['-', '+']).unwrap_or(e);
        !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit())
    });
    if !(ok_mantissa && ok_exponent) {
        return None;
    }
    text.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_schema_not_yaml_1_1() {
        assert_eq!(resolve("no"), Yaml::Str("no".into()));
        assert_eq!(resolve("yes"), Yaml::Str("yes".into()));
        assert_eq!(resolve("12:30"), Yaml::Str("12:30".into()));
        assert_eq!(resolve("0x1F"), Yaml::Int(31));
        assert_eq!(resolve("0o17"), Yaml::Int(15));
        assert_eq!(resolve("012"), Yaml::Int(12));
        assert_eq!(resolve("1.20"), Yaml::Float(1.2));
        assert_eq!(resolve("1."), Yaml::Float(1.0));
        assert_eq!(resolve(".5"), Yaml::Float(0.5));
        assert_eq!(resolve("1e3"), Yaml::Float(1000.0));
        assert_eq!(resolve("2026-09-22"), Yaml::Str("2026-09-22".into()));
        assert_eq!(resolve("0b101"), Yaml::Str("0b101".into()));
        assert_eq!(resolve("."), Yaml::Str(".".into()));
        assert_eq!(resolve(""), Yaml::Null);
    }

    #[test]
    fn mappings_tags_and_merge_keys() {
        let m = parse_mapping("a: &x {b: 1}\nc:\n  <<: *x\ne: !!str 12\n").unwrap();
        assert_eq!(m[2], ("e".into(), Yaml::Str("12".into())));
        // `<<` is an ordinary key: nothing is merged.
        let b = Yaml::Map(vec![("b".into(), Yaml::Int(1))]);
        assert_eq!(m[1].1, Yaml::Map(vec![("<<".into(), b)]));
        assert_eq!(
            parse_mapping("t: !!timestamp 2026-01-01"),
            Err(Problem::NotYaml)
        );
        assert_eq!(parse_mapping("t: !custom x"), Err(Problem::NotYaml));
        assert_eq!(parse_mapping("- a\n- b"), Err(Problem::NotAMapping));
        assert_eq!(parse_mapping("just text"), Err(Problem::NotAMapping));
        assert_eq!(parse_mapping("# only a comment\n"), Ok(vec![]));
        assert_eq!(parse_mapping("title: [never closed"), Err(Problem::NotYaml));
    }

    #[test]
    fn verbatim_tags_and_tagged_values() {
        let m = parse_mapping("t: !<tag:yaml.org,2002:str> 2026\n").unwrap();
        assert_eq!(m[0].1, Yaml::Str("2026".into()));
        assert_eq!(
            parse_mapping("t: !<tag:example.com,2000:x> 1\n"),
            Err(Problem::NotYaml)
        );
        for bad in [
            "b: !!bool yes",
            "b: !!bool tRuE",
            "i: !!int 3.0",
            "f: !!float 1_000",
        ] {
            assert_eq!(parse_mapping(bad), Err(Problem::NotYaml), "{bad}");
        }
    }

    #[test]
    fn the_three_limits() {
        // An integer beyond 64 bits, in any of its forms, but not as a float.
        for big in [
            "99999999999999999999",
            "-9223372036854775809",
            "0xFFFFFFFFFFFFFFFFFF",
        ] {
            assert_eq!(
                parse_mapping(&format!("n: {big}")),
                Err(Problem::NotYaml),
                "{big}"
            );
        }
        assert_eq!(
            parse_mapping("n: 9223372036854775807").unwrap()[0].1,
            Yaml::Int(i64::MAX)
        );
        assert_eq!(
            parse_mapping("n: !!float 99999999999999999999").unwrap()[0].1,
            Yaml::Float(1e20)
        );
        // Nesting: the mapping itself and 99 more are allowed, 100 more are not.
        let nest = |n: usize| format!("x: {}{}", "[".repeat(n), "]".repeat(n));
        assert!(parse_mapping(&nest(99)).is_ok());
        assert_eq!(parse_mapping(&nest(100)), Err(Problem::NotYaml));
        // Aliases, counted as what they stand for.
        let mut laughs = String::from("a: &a [x, x, x, x, x, x, x, x, x]\n");
        for (i, c) in ('b'..='h').enumerate() {
            let prev = (b'a' + i as u8) as char;
            laughs += &format!("{c}: &{c} [{}]\n", vec![format!("*{prev}"); 9].join(", "));
        }
        assert_eq!(parse_mapping(&laughs), Err(Problem::NotYaml));
        assert!(parse_mapping("a: &a [1, 2]\nb: [*a, *a]\n").is_ok());
    }
}

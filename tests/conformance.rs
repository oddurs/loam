// loam — the corpus in spec/corpus, read by loam.
//
// Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
//
// The same expectations the reader in spec/ is held to. When loam and a case
// disagree, one of them is wrong; the fix is never to edit the expectation to
// match what loam happens to do (spec §10).

use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn read(case: &Path) -> (i32, Option<Value>) {
    let out = Command::new(env!("CARGO_BIN_EXE_loam"))
        .arg("reading")
        .arg(case)
        .output()
        .expect("loam runs");
    let code = out.status.code().unwrap_or(-1);
    let json = (code == 0).then(|| serde_json::from_slice(&out.stdout).expect("reading is JSON"));
    (code, json)
}

#[test]
fn every_corpus_case_reads_as_expected() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/corpus");
    let mut checked = 0;
    let mut failures = Vec::new();
    for group in ["real", "hostile"] {
        let mut cases: Vec<_> = std::fs::read_dir(corpus.join(group))
            .expect("corpus group")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        cases.sort();
        for case in cases {
            let expected: Value = serde_json::from_str(
                &std::fs::read_to_string(case.join("expected.json")).expect("expectation"),
            )
            .expect("expectation is JSON");
            let (code, got) = read(&case);
            checked += 1;
            let name = case.file_name().unwrap().to_string_lossy().into_owned();
            if expected.get("refused") == Some(&Value::Bool(true)) {
                if code != 1 {
                    failures.push(format!("{group}/{name}: read a repository it must refuse"));
                }
                continue;
            }
            match got {
                None => failures.push(format!("{group}/{name}: refused or failed (exit {code})")),
                Some(got) if got != expected => {
                    let pages = |v: &Value| v["pages"].as_object().cloned().unwrap_or_default();
                    let (e, g) = (pages(&expected), pages(&got));
                    for (path, page) in &e {
                        if g.get(path) != Some(page) {
                            failures.push(format!(
                                "{group}/{name}: {path}\n  expected {page}\n  loam     {}",
                                g.get(path).map_or("nothing".into(), Value::to_string)
                            ));
                        }
                    }
                    for path in g.keys().filter(|p| !e.contains_key(*p)) {
                        failures.push(format!("{group}/{name}: {path} is not a page"));
                    }
                }
                Some(_) => {}
            }
        }
    }
    assert!(
        checked > 30,
        "the corpus is nearly empty, so this asserts little"
    );
    assert!(
        failures.is_empty(),
        "{} disagreement(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

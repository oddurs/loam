#!/usr/bin/env python3
"""Run a reader against the corpus and compare its readings with the expected ones.

Each directory under `spec/corpus/real` and `spec/corpus/hostile` is a small
repository: a `loam.toml`, a docs tree, and `expected.json`, the reading a
conforming reader must produce for it. `spec/corpus/README.md` describes the
shape of a reading.

    python3 spec/conformance.py                    # the reference reader
    python3 spec/conformance.py -- ./my-reader     # any reader: given a
                                                   # repository path, it prints
                                                   # the reading as JSON and
                                                   # exits 0, or refuses the
                                                   # repository and exits 1

When a reader disagrees with an expectation, one of them is wrong, and finding
out which is the point. Resolve it by changing the specification or the corpus
— never by teaching the reader what the corpus happens to say, which would make
it agree without making it correct.

Copyright (C) 2026 Oddur Sigurdsson. Permissive; see reader.py.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "corpus")


def cases():
    for group in ("real", "hostile"):
        base = os.path.join(CORPUS, group)
        for name in sorted(os.listdir(base)):
            path = os.path.join(base, name)
            if os.path.isdir(path):
                yield f"{group}/{name}", path


def reference(path):
    """Read with spec/reader.py, in process: (exit status, reading or None)."""
    sys.path.insert(0, HERE)
    try:
        import reader
    except ModuleNotFoundError as missing:
        # §3.2 needs a YAML parser, and writing a second YAML implementation to
        # check the first would prove nothing. Say what to install.
        package = {"yaml": "pyyaml"}.get(missing.name, missing.name or "it")
        print(f"the reference reader needs `{missing.name}`: python3 -m pip install {package}", file=sys.stderr)
        sys.exit(2)
    try:
        return 0, json.loads(json.dumps(reader.read_project(path), ensure_ascii=False))
    except reader.Refused:
        return 1, None


def external(command):
    def run(path):
        done = subprocess.run([*command, path], capture_output=True, text=True)
        if done.returncode != 0:
            return done.returncode, None
        return 0, json.loads(done.stdout)

    return run


def differences(expected, got, where=""):
    """Every place two readings differ, as (where, expected, got)."""
    if isinstance(expected, dict) and isinstance(got, dict):
        for key in sorted(set(expected) | set(got)):
            yield from differences(expected.get(key), got.get(key), f"{where}.{key}" if where else key)
    elif isinstance(expected, list) and isinstance(got, list) and len(expected) == len(got):
        for i, (e, g) in enumerate(zip(expected, got)):
            yield from differences(e, g, f"{where}[{i}]")
    elif expected != got:
        yield where, expected, got


def main(argv):
    read = reference
    if "--" in argv:
        command = argv[argv.index("--") + 1 :]
        if not command:
            print("usage: conformance.py [-- READER-COMMAND...]", file=sys.stderr)
            return 2
        read = external(command)

    failures = checked = 0
    for label, path in cases():
        expected_path = os.path.join(path, "expected.json")
        if not os.path.exists(expected_path):
            print(f"  ?  {label}: no expectation committed")
            failures += 1
            continue
        with open(expected_path, encoding="utf-8") as f:
            expected = json.load(f)
        checked += 1

        status, got = read(path)
        if expected.get("refused"):
            if status == 1:
                print(f"  ✓  {label} (refused, as it must be)")
            else:
                failures += 1
                print(f"  ✗  {label}: read a repository it must refuse (exit {status})")
            continue
        if status != 0:
            failures += 1
            print(f"  ✗  {label}: the reader refused it or failed (exit {status})")
            continue

        diffs = list(differences(expected, got))
        if not diffs:
            print(f"  ✓  {label}")
            continue
        failures += 1
        print(f"  ✗  {label}")
        for where, e, g in diffs[:20]:
            print(f"       {where}")
            print(f"         expected: {e!r}")
            print(f"         reader:   {g!r}")
        if len(diffs) > 20:
            print(f"       … and {len(diffs) - 20} more")

    print()
    if checked == 0:
        print("the corpus is empty, so this test asserts nothing", file=sys.stderr)
        return 1
    if failures:
        print(f"{failures} of {checked} cases disagree.")
        return 1
    print(f"{checked} cases, every reading as expected.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

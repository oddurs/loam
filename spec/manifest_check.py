#!/usr/bin/env python3
"""Hold `loam index --json` to spec/manifest.md.

For each folder given — by default this repository and the real folders in
spec/corpus — it checks that the manifest:

- validates against spec/manifest.schema.json;
- agrees with itself: every section's pages are pages, every page is in at most
  one section, and every link to a page is that page's backlink and the
  reverse;
- is what the Markdown index is made from: `loam index` lists the same pages,
  in the same order, under the same headings, as `sections`;
- is enough to build a site from: docs/cookbook/site.py, reading nothing else,
  builds one in which every link between pages, and every anchor, lands.

    python3 spec/manifest_check.py LOAM [FOLDER ...]

Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
"""

import contextlib
import html
import io
import json
import os
import posixpath
import re
import subprocess
import sys
import urllib.parse

import jsonschema

HERE = os.path.dirname(os.path.abspath(__file__))


def check(loam, folder, validator):
    run = lambda *a: subprocess.run([loam, "-C", folder, *a], capture_output=True, text=True, check=True).stdout
    m = json.loads(run("index", "--json"))
    problems = [f"schema: {e.json_path}: {e.message}" for e in validator.iter_errors(m)]
    pages = m["pages"]

    seen = set()
    for s in m["sections"]:
        for p in s["pages"]:
            if p not in pages:
                problems.append(f"section {s['heading']!r} lists {p}, which is not a page")
            if p in seen:
                problems.append(f"{p} is in two sections")
            seen.add(p)

    # Backlinks are exactly the page links, turned around.
    expected = {}
    for path, page in pages.items():
        if path == m["index"]:
            continue
        for link in page["links"]:
            t = link["target"]
            if link["class"] == "page" and t in pages and t != path:
                entry = {"path": path, "line": link["line"]}
                if entry not in expected.setdefault(t, []):
                    expected[t].append(entry)
    for path, page in pages.items():
        want = sorted(expected.get(path, []), key=lambda b: (b["path"], b["line"]))
        if page["backlinks"] != want:
            problems.append(f"{path}: backlinks {page['backlinks']} but links say {want}")

    # The Markdown index, read back: its headings and each row's page.
    index_dir = posixpath.dirname(m["index"])
    drawn, heading = [], None
    for line in run("index").splitlines():
        if line.startswith("## "):
            heading = line[3:]
            drawn.append((heading, []))
        # Link text may hold brackets, balanced, as a title can.
        row = re.match(r"^\| \[(?:[^\[\]\\]|\\.|\[(?:[^\[\]\\]|\\.)*\])*\]\(([^)]*)\)", line)
        if row and drawn:
            target = posixpath.normpath(posixpath.join(index_dir, urllib.parse.unquote(row.group(1))))
            drawn[-1][1].append(target)
    listed = [(s["heading"], s["pages"]) for s in m["sections"]]
    if drawn != listed:
        first = next(
            (i for i, (a, b) in enumerate(zip(drawn, listed)) if a != b),
            min(len(drawn), len(listed)),
        )
        near = lambda x: x[first] if first < len(x) else None
        problems.append(f"the Markdown index is not the sections, from section {first}: {near(drawn)!s:.300} ≠ {near(listed)!s:.300}")
    problems += site_problems(folder, m)
    return len(pages), problems


def site_problems(folder, manifest):
    """Build the cookbook's site from the manifest, and follow every link in it."""
    import importlib.util
    import tempfile

    spec = importlib.util.spec_from_file_location("site", os.path.join(os.path.dirname(HERE), "docs", "cookbook", "site.py"))
    site = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(site)
    problems = []
    with tempfile.TemporaryDirectory() as out:
        path = os.path.join(out, "manifest.json")
        with open(path, "w") as f:
            json.dump(manifest, f)
        os.environ["LOAM_ROOT"] = folder
        with contextlib.redirect_stdout(io.StringIO()):
            site.build(path, os.path.join(out, "site"))
        root = os.path.join(out, "site")
        files = [os.path.join(d, f) for d, _, fs in os.walk(root) for f in fs if f.endswith(".html")]
        ids = {f: set(re.findall(r'id="([^"]*)"', open(f).read())) for f in files}
        for f in files:
            for href in re.findall(r'href="([^"]*)"', open(f).read()):
                href = html.unescape(href)
                path, _, frag = href.partition("#")
                if re.match(r"[a-z]+:", href) or not (path.endswith(".html") or (frag and not path)):
                    continue
                target = os.path.normpath(os.path.join(os.path.dirname(f), path)) if path else f
                if not os.path.exists(target):
                    problems.append(f"site: {os.path.relpath(f, root)} links to {href}, which was not built")
                elif frag and urllib.parse.unquote(frag) not in ids[target]:
                    problems.append(f"site: {os.path.relpath(f, root)} links to {href}, an anchor it does not have")
    return problems


def main(argv):
    loam = argv[1]
    folders = argv[2:] or [
        os.path.dirname(HERE),
        *sorted(os.path.join(HERE, "corpus", "real", d) for d in os.listdir(os.path.join(HERE, "corpus", "real"))),
    ]
    with open(os.path.join(HERE, "manifest.schema.json")) as f:
        schema = json.load(f)
    jsonschema.Draft202012Validator.check_schema(schema)
    validator = jsonschema.Draft202012Validator(schema)
    failed = False
    for folder in folders:
        n, problems = check(loam, folder, validator)
        name = os.path.relpath(folder, os.path.dirname(HERE))
        print(f"  {'✗' if problems else '✓'}  {name} ({n} pages)")
        for p in problems[:10]:
            print(f"       {p}")
        failed |= bool(problems)
    print()
    print("manifest: " + ("failed" if failed else "every manifest valid, and the index made from it"))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

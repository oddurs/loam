#!/usr/bin/env python3
"""Check that the text this specification shares with cairn's is still cairn's.

`spec/README.md` marks the passages it takes word for word from the cairn item
format's specification, between `<!-- shared with the cairn item format … -->`
and `<!-- shared: end -->`. Two specifications that describe the same
frontmatter in different words are a bug waiting for a reader to find it, so
each marked passage must appear, exactly, in cairn's.

    python3 spec/shared.py                        # finds cairn's specification
    python3 spec/shared.py PATH/TO/cairn/spec/README.md

Without a path it looks in `$CAIRN_SPEC`, then beside this repository at
`../cairn/spec/README.md`, then fetches the published copy from GitHub.

When this fails, one specification changed and the other did not. Decide which
wording is right and change both; never unmark a passage to make it pass.

Copyright (C) 2026 Oddur Sigurdsson. Permissive; see reader.py.
"""

import os
import re
import sys
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
PUBLISHED = "https://raw.githubusercontent.com/oddurs/cairn/main/spec/README.md"


def cairn_spec(argv):
    candidates = argv[:1] or [os.environ.get("CAIRN_SPEC"), os.path.join(HERE, "..", "..", "cairn", "spec", "README.md")]
    for path in candidates:
        if path and os.path.exists(path):
            with open(path, encoding="utf-8") as f:
                return path, f.read()
    if argv:
        sys.exit(f"no such file: {argv[0]}")
    with urllib.request.urlopen(PUBLISHED, timeout=20) as response:
        return PUBLISHED, response.read().decode("utf-8")


def main(argv):
    with open(os.path.join(HERE, "README.md"), encoding="utf-8") as f:
        ours = f.read()
    where, theirs = cairn_spec(argv)

    passages = re.findall(r"<!-- shared with the cairn item format, (§\d+): begin -->\n(.*?)<!-- shared: end -->", ours, re.S)
    if not passages:
        print("no shared passages are marked, so this test asserts nothing", file=sys.stderr)
        return 1

    failures = 0
    for section, text in passages:
        first = text.strip().split("\n")[0]
        if text in theirs:
            print(f"  ✓  cairn {section}: {first[:60]}")
        else:
            failures += 1
            print(f"  ✗  cairn {section}: not found word for word in {where}")
            print(f"       it begins: {first[:70]}")
    print()
    if failures:
        print(f"{failures} of {len(passages)} shared passages differ from cairn's.")
        return 1
    print(f"{len(passages)} shared passages, word for word the same as cairn's.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

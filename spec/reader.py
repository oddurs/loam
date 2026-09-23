#!/usr/bin/env python3
"""A reader for the loam page format, written from the specification.

It exists because `spec/README.md` claims a reader can be written from it
alone. It follows the specification section by section, and cites the section
it is implementing wherever a rule is not obvious. It was written before any
program called loam existed, so there was no other code to consult.

Deliberately short and deliberately plain: reading it should be a reasonable
way to learn the format. `spec/conformance.py` runs it against the corpus in
`spec/corpus` and compares its answers with the committed expectations.

    python3 spec/reader.py PROJECT-DIRECTORY     # prints the reading as JSON

Copyright (C) 2026 Oddur Sigurdsson.

Permission to use, copy, modify, and distribute this software for any purpose
with or without fee is hereby granted, provided that the above copyright notice
and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS
OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
THIS SOFTWARE.
"""

import html
import json
import os
import re
import sys
import tomllib
import unicodedata
import urllib.parse

import yaml  # §3.2: frontmatter is YAML, so a YAML parser is required.

FORMAT = 1
WS = " \t"  # §2: whitespace is spaces and tabs, never the rest of Unicode's


class Refused(Exception):
    """§9: a project this reader must decline rather than misread."""


# ─── §3.2 YAML 1.2 core ─────────────────────────────────────────────────────


class Core(yaml.SafeLoader):
    """Resolve scalars by the YAML 1.2 core schema, as §3.2 requires.

    PyYAML implements YAML 1.1, under which `status: no` is the boolean false
    and `12:30` is the integer 750. Replacing its resolvers with the 1.2 core
    set is the whole adjustment. Dates are not in the core schema, so
    `2026-09-22` stays a string, which is what a page means by it.
    """


Core.yaml_implicit_resolvers = {}
for _tag, _pattern, _first in [
    ("tag:yaml.org,2002:null", r"^(?:~|null|Null|NULL|)$", [*"~nN", ""]),
    ("tag:yaml.org,2002:bool", r"^(?:true|True|TRUE|false|False|FALSE)$", "tTfF"),
    ("tag:yaml.org,2002:int", r"^(?:[-+]?[0-9]+|0o[0-7]+|0x[0-9a-fA-F]+)$", "-+0123456789"),
    (
        "tag:yaml.org,2002:float",
        r"^(?:[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)(?:[eE][-+]?[0-9]+)?"
        r"|[-+]?\.(?:inf|Inf|INF)|\.(?:nan|NaN|NAN))$",
        "-+0123456789.",
    ),
]:
    Core.add_implicit_resolver(_tag, re.compile(_pattern), list(_first))  # "" is an empty scalar

# The 1.2 core schema has no timestamps, binary, sets, ordered maps or pairs, and
# no merge key: `<<` resolves as an ordinary string, so nothing is merged. A tag
# outside the core schema is left with no constructor, PyYAML raises, and §3.2
# makes that malformed frontmatter.
def _construct_int(loader, node):
    """1.2 core integers. PyYAML's own is 1.1's, under which `012` is octal ten."""
    value = loader.construct_scalar(node)
    if value.startswith("0o"):
        return int(value[2:], 8)
    if value.startswith("0x"):
        return int(value[2:], 16)
    return int(value, 10)


Core.yaml_constructors = {
    tag: construct
    for tag, construct in yaml.SafeLoader.yaml_constructors.items()
    if tag is None or tag.rsplit(":", 1)[-1] in ("null", "bool", "int", "float", "str", "seq", "map")
}
Core.add_constructor("tag:yaml.org,2002:int", _construct_int)


# ─── §7.1 Configuration ───────────────────────────────────────────────────


def read_config(root):
    path = os.path.join(root, "loam.toml")
    with open(path, "rb") as f:
        config = tomllib.load(f)
    fmt = config.get("format")
    if fmt != FORMAT:
        # §9: refuse, naming both versions. Never read on a best-effort basis.
        found = "no format" if fmt is None else f"format {fmt}"
        raise Refused(f"{path}: {found}; this reader understands format {FORMAT}")
    docs = config.get("docs", {}).get("root", "docs")
    kinds = []
    for kind in config.get("kind", []):
        d = kind.get("dir", ".").strip("/")
        kinds.append((kind["name"], "" if d in ("", ".") else d))
    cairn = config.get("links", {}).get("cairn")
    docs = docs.strip("/")
    return {
        "docs": "" if docs == "." else docs,
        "kinds": kinds,
        "cairn": cairn.strip("/") if cairn else None,
    }


# ─── §3 Structure of a page ─────────────────────────────────────────────────


def split(text):
    """Return (frontmatter or None, body, body's first line number, problem)."""
    lines = text.split("\n")
    if lines[0].rstrip(WS) != "---":
        return None, text, 1, None
    for i in range(1, len(lines)):
        # §3.1: the *first* line that trims to `---` or `...` closes it.
        if lines[i].rstrip(WS) in ("---", "..."):
            return "\n".join(lines[1:i]), "\n".join(lines[i + 1 :]), i + 2, None
    return None, "", len(lines) + 1, "no closing delimiter"


# ─── §5 Reading the body ────────────────────────────────────────────────────

FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})(.*)$")
ATX = re.compile(r"^ {0,3}(#{1,6})(?:[ \t]+(.*?))?[ \t]*$")
SETEXT = re.compile(r"^ {0,3}(=+|-+)[ \t]*$")
BREAK = re.compile(r"^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$")
LIST = re.compile(r"^ {0,3}(?:[-*+]|\d{1,9}[.)])(?:[ \t]|$)")
REFDEF = re.compile(r"^ {0,3}\[((?:[^\]\\]|\\.)+)\]:[ \t]*(<[^>\n]*>|[^ \t]+)")  # §2: whitespace is space and tab


def blocks(body, first_line):
    """§5.1: classify every line of the body, outside code, into blocks.

    Yields (kind, line_number, lines) where kind is 'heading', 'paragraph',
    'other' or 'code'. Headings carry (level, text) in place of lines.
    """
    lines = body.split("\n")
    out = []
    para = []  # (number, text) of the paragraph being gathered
    fence = None
    indented_code = False
    prev_blank = True

    def flush():
        if para:
            out.append(("paragraph", para[0][0], [t for _, t in para]))
            para.clear()

    for offset, line in enumerate(lines):
        n = first_line + offset
        if fence:
            m = re.match(r"^ {0,3}(`+|~+)[ \t]*$", line)
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= len(fence):
                fence = None
            out.append(("code", n, [line]))
            continue
        if not line.strip(WS):
            flush()
            prev_blank = True
            continue
        indented = line.startswith("\t") or line.startswith("    ")
        if indented and (indented_code or (prev_blank and not para)):
            # §5.1: indented code begins only after a blank line, so it never
            # interrupts a paragraph, and runs until a line that is not indented.
            indented_code = True
            prev_blank = False
            out.append(("code", n, [line]))
            continue
        indented_code = False
        prev_blank = False
        m = FENCE.match(line)
        if m and not (m.group(1)[0] == "`" and "`" in m.group(2)):
            flush()
            fence = m.group(1)
            out.append(("code", n, [line]))
            continue
        m = SETEXT.match(line)
        if m and para and not LIST.match(para[0][1]) and not para[0][1].lstrip(WS).startswith(">"):
            level = 1 if m.group(1)[0] == "=" else 2
            text = " ".join(t.strip(WS) for _, t in para)
            number = para[0][0]
            para.clear()
            out.append(("heading", number, (level, text)))
            continue
        m = ATX.match(line)
        if m:
            flush()
            text = m.group(2) or ""
            text = re.sub(r"(?:^|[ \t]+)#+$", "", text).strip(WS)  # closing sequence
            out.append(("heading", n, (len(m.group(1)), text)))
            continue
        if BREAK.match(line):
            flush()
            out.append(("other", n, [line]))
            continue
        if para and (LIST.match(line) or line.lstrip(WS).startswith(">")) and not (
            LIST.match(para[0][1]) or para[0][1].lstrip(WS).startswith(">")
        ):
            flush()  # a list or a quote interrupts a paragraph
        para.append((n, line))
    flush()
    return out


def is_summary_candidate(lines):
    """§5.3: a paragraph that is prose, not a list, quote, table or picture."""
    first = lines[0]
    if LIST.match(first) or first.lstrip(WS).startswith((">", "|", "<")):
        return False
    return not any(REFDEF.match(line) for line in lines)


def is_picture(lines):
    """§5.3: a paragraph that is only images (a banner, a screenshot, badges)."""
    text = " ".join(lines)
    text = re.sub(r"\[?!\[(?:[^\]\\]|\\.)*\]\([^)]*\)(?:\]\([^)]*\))?", "", text)
    text = re.sub(r"\[?!\[(?:[^\]\\]|\\.)*\]\[[^\]]*\](?:\]\([^)]*\))?", "", text)
    return not text.strip(WS)


# ─── §6.3 Anchors ───────────────────────────────────────────────────────────


def heading_text(source):
    """§6.3: the text of a heading as it renders, markup removed."""
    t = source
    t = re.sub(r"!\[(?:[^\]\\]|\\.)*\]\([^)]*\)", "", t)  # images vanish
    t = re.sub(r"!\[(?:[^\]\\]|\\.)*\]\[[^\]]*\]", "", t)
    t = re.sub(r"\[((?:[^\]\\]|\\.)*)\]\([^)]*\)", r"\1", t)  # links keep their text
    t = re.sub(r"\[((?:[^\]\\]|\\.)*)\]\[[^\]]*\]", r"\1", t)
    t = re.sub(r"<[^>\n]*>", "", t)  # inline HTML tags
    t = re.sub(r"\\([!-/:-@\[-`{-~])", r"\1", t)  # backslash escapes
    t = re.sub(r"(?<![^\W_])_+(?=[^\W_])|(?<=[^\W_])_+(?![^\W_])", "", t)  # _emphasis_, not snake_case
    return html.unescape(t)


# Alphabetic characters outside the letter categories: circled and squared
# Latin letters, which are symbols by category and letters by property.
OTHER_ALPHABETIC = [(0x24B6, 0x24E9), (0x1F130, 0x1F149), (0x1F150, 0x1F169), (0x1F170, 0x1F189)]


def is_word(ch):
    """§6.3: a word character in the sense of Unicode TS #18, Annex C."""
    cat = unicodedata.category(ch)
    if cat[0] in "LM" or cat in ("Nd", "Nl", "Pc") or ch in "\u200c\u200d":
        return True
    return any(lo <= ord(ch) <= hi for lo, hi in OTHER_ALPHABETIC)


def slugify(text):
    """§6.3: GitHub's rule."""
    return "".join("-" if ch == " " else ch for ch in text.lower() if ch in "- " or is_word(ch))


def anchors(block_list):
    seen = {}
    result = []
    for kind, _, data in block_list:
        if kind != "heading":
            continue
        base = slugify(heading_text(data[1]))
        slug = base
        # §6.3: the same loop github-slugger runs.
        while slug in seen:
            seen[base] += 1
            slug = f"{base}-{seen[base]}"
        seen[slug] = 0
        result.append(slug)
    # §6.3: `id` and `name` attributes of HTML tags in headings and paragraphs.
    for kind, _, data in block_list:
        if kind not in ("paragraph", "heading"):
            continue
        lines = [data[1]] if kind == "heading" else data
        text = "\n".join(strip_code_spans(line) for line in lines)
        for tag in re.finditer(r"<([A-Za-z][A-Za-z0-9-]*)[^>]*>", text):
            # A browser follows a fragment to an `id` on anything, and to a
            # `name` only on an `a`.
            names = "id|name" if tag.group(1).lower() == "a" else "id"
            for m in re.finditer(rf"""[ \t\n](?:{names})[ \t]*=[ \t]*(?:"([^"]*)"|'([^']*)')""", tag.group(), re.I):
                value = m.group(1) if m.group(1) is not None else m.group(2)
                if value not in result:
                    result.append(value)
    return result


# ─── §6.1 Finding links ─────────────────────────────────────────────────────


def strip_code_spans(line):
    """Replace code spans with spaces, keeping columns, so links inside vanish."""
    out = []
    i = 0
    while i < len(line):
        if line[i] == "\\" and i + 1 < len(line):
            out.append(line[i : i + 2])
            i += 2
            continue
        if line[i] == "`":
            j = i
            while j < len(line) and line[j] == "`":
                j += 1
            run = line[i:j]
            close = line.find(run, j)
            while close != -1 and (close + len(run) < len(line) and line[close + len(run)] == "`"):
                k = close
                while k < len(line) and line[k] == "`":
                    k += 1
                close = line.find(run, k)
            if close == -1:
                out.append(run)
                i = j
                continue
            # Spaces, byte for byte, but a line break stays one (§6.1).
            out.append(re.sub(r"[^\n]", " ", line[i : close + len(run)]))
            i = close + len(run)
            continue
        out.append(line[i])
        i += 1
    return "".join(out)


def inline_destinations(line):
    """(offset, destination) of inline links and images (§6.1).

    `line` is a heading, or a paragraph's lines joined with newlines: a link's
    text may run over several lines, but its destination stays on one.
    """
    found = []
    i = 0
    while True:
        i = line.find("](", i)
        if i == -1:
            return found
        # The `]` must close a `[` on this line, with brackets balanced.
        depth, j = 0, i
        opened = False
        while j >= 0:
            c = line[j]
            escaped = j > 0 and line[j - 1] == "\\"
            if c == "]" and not escaped:
                depth += 1
            elif c == "[" and not escaped:
                depth -= 1
                if depth == 0:
                    opened = True
                    break
            j -= 1
        k = i + 2
        if not opened:
            i = k
            continue
        while k < len(line) and line[k] in " \t":
            k += 1
        if k < len(line) and line[k] == "<":
            end = line.find(">", k)
            if end == -1 or "\n" in line[k:end]:
                i = k
                continue
            dest, dest_at = line[k + 1 : end], k + 1
            rest = end + 1
        else:
            parens, start = 0, k
            while k < len(line):
                c = line[k]
                if c == "\\" and k + 1 < len(line) and line[k + 1] != "\n":
                    k += 2
                    continue
                if c in " \t\n":
                    break
                if c == "(":
                    parens += 1
                elif c == ")":
                    if parens == 0:
                        break
                    parens -= 1
                k += 1
            dest, dest_at = line[start:k], start
            rest = k
        # An optional title, then `)`.
        m = re.match(r"""[ \t]*(?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\((?:[^()\\]|\\.)*\))?[ \t]*\)""", line[rest:].split("\n", 1)[0])
        if m and dest:
            found.append((dest_at, dest))
        i = rest


def links(block_list):
    """Every link a reader must find, with the line it is on (§6.1)."""
    result = []
    for kind, number, data in block_list:
        if kind == "code":
            continue
        if kind == "heading":
            result.extend((number, dest) for _, dest in inline_destinations(strip_code_spans(data[1])))
            continue
        # A paragraph: definitions line by line, inline links across lines.
        found, joined, starts = [], [], []
        offset = 0
        for i, line in enumerate(data):
            starts.append(offset)
            m = REFDEF.match(line)
            if m:
                dest = m.group(2)
                found.append((number + i, 0, dest[1:-1] if dest.startswith("<") else dest))
                line = " " * len(line)
            joined.append(line)
            offset += len(line) + 1
        for at, dest in inline_destinations(strip_code_spans("\n".join(joined))):
            i = max(k for k, start in enumerate(starts) if start <= at)
            found.append((number + i, at - starts[i], dest))
        result.extend((n, dest) for n, _, dest in sorted(found, key=lambda f: (f[0], f[1])))
    return result


# ─── §6.2 Resolving links ───────────────────────────────────────────────────

SCHEME = re.compile(r"^[A-Za-z][A-Za-z0-9+.\-]*:")


def unescape_destination(dest):
    return re.sub(r"\\([!-/:-@\[-`{-~])", r"\1", dest)


def resolve(project, page_path, dest):
    """Return (target path or None, fragment or None, class)."""
    dest = unescape_destination(dest)
    if SCHEME.match(dest) or dest.startswith("//"):
        return None, None, "external"
    path, _, fragment = dest.partition("#")
    path = path.split("?", 1)[0]
    path = urllib.parse.unquote(path)
    fragment = urllib.parse.unquote(fragment) or None  # `page.md#` names no anchor
    if path == "":
        return page_path, fragment, "self"
    if path.startswith("/"):
        joined = path.lstrip("/")
    else:
        joined = os.path.join(os.path.dirname(page_path), path)
    parts = []
    for part in joined.split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            if not parts:
                return None, fragment, "outside"
            parts.pop()
        else:
            parts.append(part)
    return "/".join(parts), fragment, "path"


# ─── Reading a project ──────────────────────────────────────────────────────


def walk(root):
    """Every file and directory under root, as exact relative paths (§6.2)."""
    files, dirs = set(), set()
    for base, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d != ".git"]
        rel = os.path.relpath(base, root)
        rel = "" if rel == "." else rel.replace(os.sep, "/")
        if rel:
            dirs.add(rel)
        for name in filenames:
            files.add(f"{rel}/{name}" if rel else name)
    return files, dirs


def is_page(project, path):
    """§7.2: a Markdown file under the docs root, not hidden."""
    docs = project["docs"]
    if not path.endswith(".md"):
        return False
    if docs and not path.startswith(docs + "/"):
        return False
    inside = path[len(docs) + 1 :] if docs else path
    return not any(part.startswith((".", "_")) for part in inside.split("/"))


def kind_for(project, path):
    """§5.4: the kind whose directory is the longest prefix of the page's."""
    docs = project["docs"]
    inside = path[len(docs) + 1 :] if docs else path
    directory = os.path.dirname(inside)
    best = None
    for name, d in project["kinds"]:
        if d == "" or directory == d or directory.startswith(d + "/"):
            if best is None or len(d) > len(best[1]):
                best = (name, d)
    return best[0] if best else None


def cairn_id(project, target):
    """§6.4: a link into the cairn items directory whose name leads with digits."""
    items = project["cairn"]
    if not items or not target.startswith(items + "/"):
        return None
    name = target[len(items) + 1 :]
    m = re.match(r"(\d+)", name)
    if not m or "/" in name or not name.endswith(".md"):
        return None
    return int(m.group(1))


def read_page(project, path):
    with open(os.path.join(project["root"], path), "rb") as f:
        raw = f.read()
    findings = []
    try:
        text = raw.decode("utf-8")
        front_text, body, first_line, problem = split(text.removeprefix("\ufeff").replace("\r\n", "\n"))
    except UnicodeDecodeError:
        # §3: a page is UTF-8. One that is not is reported and nothing is read.
        findings.append({"line": 1, "code": "invalid-encoding", "detail": None})
        front_text, body, first_line, problem = None, "", 1, "invalid-encoding"

    meta = None
    if problem and problem != "invalid-encoding":
        findings.append({"line": 1, "code": "malformed-frontmatter", "detail": problem})
    elif front_text is not None:
        try:
            meta = yaml.load(front_text, Loader=Core) if front_text.strip() else {}
            if meta is None:
                meta = {}
            if not isinstance(meta, dict):
                findings.append({"line": 1, "code": "malformed-frontmatter", "detail": "not a mapping"})
                meta = None
        except yaml.YAMLError:
            findings.append({"line": 1, "code": "malformed-frontmatter", "detail": "not YAML"})
            meta = None

    def string_key(key):
        if meta is None or key not in meta or meta[key] is None:
            return None
        if isinstance(meta[key], str):
            return meta[key]
        findings.append({"line": 1, "code": "malformed-key", "detail": key})
        return None

    def path_list(key):
        if meta is None or key not in meta or meta[key] is None:
            return []
        value = meta[key]
        if isinstance(value, str):
            return [value]
        if isinstance(value, list) and all(isinstance(v, str) for v in value):
            return value
        findings.append({"line": 1, "code": "malformed-key", "detail": key})
        return []

    # §3.1: without a closing delimiter nothing in the file can be trusted.
    block_list = [] if problem else blocks(body, first_line)

    title, title_from = string_key("title"), "frontmatter"
    title_heading = None
    for i, (kind, number, data) in enumerate(block_list):
        if kind == "heading" and data[0] == 1:
            title_heading = i
            break
    if title is None:
        title_from = None
        if title_heading is not None:
            title, title_from = block_list[title_heading][2][1], "heading"

    kind, kind_from = string_key("kind"), "frontmatter"
    if kind is None:
        kind, kind_from = kind_for(project, path), "directory"
        if kind is None:
            kind_from = None
            findings.append({"line": 1, "code": "unclaimed", "detail": None})
    elif kind not in [k for k, _ in project["kinds"]]:
        findings.append({"line": 1, "code": "unknown-kind", "detail": kind})

    order = None
    if meta is not None and meta.get("order") is not None:
        if isinstance(meta["order"], int) and not isinstance(meta["order"], bool):
            order = meta["order"]
        else:
            findings.append({"line": 1, "code": "malformed-key", "detail": "order"})

    supersedes = path_list("supersedes")
    superseded_by = path_list("superseded_by")

    status, status_from = string_key("status"), "frontmatter"
    if status is None:
        status, status_from = ("superseded" if superseded_by else "current"), "default"
    elif status not in ("draft", "current", "superseded"):
        findings.append({"line": 1, "code": "unknown-status", "detail": status})

    summary, summary_from = string_key("summary"), "frontmatter"
    if summary is None:
        summary_from = None
        start = title_heading + 1 if title_heading is not None else 0
        for kind_, _, data in block_list[start:]:
            # §5.3: only the first block after the title is considered, and a
            # paragraph holding nothing but images is passed over.
            if kind_ == "paragraph" and is_picture(data):
                continue
            if kind_ == "paragraph" and is_summary_candidate(data):
                summary = " ".join(line.strip(WS) for line in data)
                summary_from = "paragraph"
            break

    if title is None and not problem:
        findings.append({"line": 1, "code": "untitled", "detail": None})

    return {
        "frontmatter": meta,
        "title": title,
        "title_from": title_from,
        "kind": kind,
        "kind_from": kind_from,
        "status": status,
        "status_from": status_from,
        "summary": summary,
        "summary_from": summary_from,
        "order": order,
        "supersedes": supersedes,
        "superseded_by": superseded_by,
        "anchors": anchors(block_list),
        "_links": links(block_list),
        "findings": findings,
    }


def read_project(root):
    project = read_config(root)
    project["root"] = root
    files, dirs = walk(root)
    pages = {p: read_page(project, p) for p in sorted(files) if is_page(project, p)}

    anchor_cache = {}

    def anchors_of(path):
        if path in pages:
            return pages[path]["anchors"]
        if path not in anchor_cache:
            with open(os.path.join(root, path), "rb") as f:
                text = f.read().decode("utf-8", "replace").removeprefix("\ufeff").replace("\r\n", "\n")
            _, body, first, problem = split(text)
            anchor_cache[path] = [] if problem else anchors(blocks(body, first))
        return anchor_cache[path]

    item_files = {}
    if project["cairn"]:
        for f in files:
            if f.startswith(project["cairn"] + "/") and "/" not in f[len(project["cairn"]) + 1 :]:
                m = re.match(r"(\d+)", f[len(project["cairn"]) + 1 :])
                if m and f.endswith(".md"):
                    item_files.setdefault(int(m.group(1)), []).append(f)

    for path, page in pages.items():
        out = []
        for line, dest in page.pop("_links"):
            target, fragment, cls = resolve(project, path, dest)
            link = {"line": line, "destination": dest, "class": cls, "target": target, "fragment": fragment}
            finding = None
            if cls == "external":
                link["class"] = "external"
            elif cls == "outside":
                finding = "escapes-repository"
            else:
                if cls == "self":
                    link["class"] = "page"
                elif cairn_id(project, target) is not None:
                    link["class"] = "cairn"
                    ident = cairn_id(project, target)
                    link["item"] = ident
                    candidates = sorted(item_files.get(ident, []))
                    if not candidates:
                        finding = "broken-cairn-link"
                    elif target not in candidates:
                        finding = "stale-cairn-link"
                        link["current"] = candidates[0]
                elif target in pages:
                    link["class"] = "page"
                elif target in files or target in dirs or target == "":
                    link["class"] = "file"
                else:
                    link["class"] = "file"
                    finding = "broken-link"
                if finding is None and fragment is not None and target.endswith(".md") and target in files:
                    if fragment not in anchors_of(target):
                        finding = "broken-anchor"
            if finding:
                page["findings"].append({"line": line, "code": finding, "detail": dest})
            out.append(link)
        page["links"] = out

        for key in ("supersedes", "superseded_by"):
            for value in page[key]:
                target, _, cls = resolve(project, path, value)
                if cls != "path" or target not in pages:
                    page["findings"].append({"line": 1, "code": "broken-supersession", "detail": value})

    # §4.2: supersession is stated on both pages, and each must agree.
    for path, page in pages.items():
        for value in page["supersedes"]:
            target, _, cls = resolve(project, path, value)
            if cls == "path" and target in pages:
                back = [resolve(project, target, v)[0] for v in pages[target]["superseded_by"]]
                if path not in back:
                    page["findings"].append({"line": 1, "code": "one-sided-supersession", "detail": value})
        for value in page["superseded_by"]:
            target, _, cls = resolve(project, path, value)
            if cls == "path" and target in pages:
                back = [resolve(project, target, v)[0] for v in pages[target]["supersedes"]]
                if path not in back:
                    page["findings"].append({"line": 1, "code": "one-sided-supersession", "detail": value})

    for page in pages.values():
        page["findings"].sort(key=lambda f: (f["line"], f["code"], f["detail"] or ""))
    return {"format": FORMAT, "pages": pages}


def main(argv):
    if len(argv) != 2:
        print("usage: reader.py PROJECT-DIRECTORY", file=sys.stderr)
        return 2
    try:
        reading = read_project(argv[1])
    except Refused as e:
        print(f"refused: {e}", file=sys.stderr)
        return 1
    print(json.dumps(reading, indent=2, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

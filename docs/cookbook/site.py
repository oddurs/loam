#!/usr/bin/env python3
"""A static site built from loam's manifest and the pages' bodies, nothing else.

    loam index --json > manifest.json
    python3 site.py manifest.json OUT-DIRECTORY

It reads no frontmatter, no loam.toml and no git: navigation is the manifest's
`sections`, a draft is marked by its `status`, a stale page gets a banner from
its `freshness`, each page gets a table of contents from its `headings`, a
"Linked from" list from its `backlinks` and the cairn items it cites from
`cairn_items`, and a link to another page goes to that page's HTML. The only other input is each page's body, from `body_line`
on. It needs Python Markdown (`pip install markdown`), and is a recipe, not a
feature: loam does not render HTML (see the README's "What loam is not").

Copyright (c) 2026 Oddur Sigurdsson. MIT licensed; see LICENSE.
"""

import html
import json
import os
import posixpath
import re
import sys
import urllib.parse

import markdown

STYLE = """
body { font: 16px/1.6 system-ui, sans-serif; margin: 0; display: flex; color: #222; }
nav { width: 16rem; padding: 1.5rem; background: #f6f5f1; min-height: 100vh; box-sizing: border-box; }
nav h2 { font-size: .8rem; text-transform: uppercase; letter-spacing: .05em; color: #777; margin: 1.5rem 0 .4rem; }
nav a { display: block; color: #333; text-decoration: none; padding: .1rem 0; }
nav a.here { font-weight: 600; }
main { max-width: 46rem; padding: 1.5rem 2.5rem; }
.draft { font-size: .75rem; color: #a60; margin-left: .3rem; }
.banner { background: #fff4d6; border-left: 4px solid #e0a800; padding: .6rem 1rem; }
.toc, .backlinks { font-size: .9rem; color: #555; border-top: 1px solid #ddd; margin-top: 2rem; padding-top: .5rem; }
code { background: #f1f1ee; padding: 0 .2rem; }
pre code { display: block; padding: .8rem; overflow-x: auto; }
"""


def html_path(page_path, docs):
    """Where a page's HTML goes, from the docs root: `guide/a.md` → `guide/a.html`."""
    inside = posixpath.relpath(page_path, docs) if docs else page_path
    return inside[: -len(".md")] + ".html"


def relative(from_file, to_file):
    return posixpath.relpath(to_file, posixpath.dirname(from_file) or ".")


def body_of(root, page):
    """The page's body, from the line the manifest says it begins on."""
    with open(os.path.join(root, page["path"]), encoding="utf-8", errors="replace") as f:
        lines = f.read().removeprefix("\ufeff").replace("\r\n", "\n").split("\n")
    return "\n".join(lines[page["body_line"] - 1 :])


def rewrite_links(body_html, page, pages, docs, here):
    """A link the manifest classes as a page goes to that page's HTML."""
    targets = {}
    for link in page["links"]:
        if link["class"] == "page" and link["target"] in pages:
            frag = "#" + urllib.parse.quote(link["fragment"]) if link["fragment"] else ""
            if link["target"] == page["path"] and not link["destination"].split("#")[0]:
                continue  # `#section` on this page already works
            targets[link["destination"]] = relative(here, html_path(link["target"], docs)) + frag

    def swap(m):
        href = html.unescape(m.group(1))
        return f'href="{html.escape(targets[href])}"' if href in targets else m.group(0)

    return re.sub(r'href="([^"]*)"', swap, body_html)


def anchor_headings(body_html, headings):
    """Each heading gets the anchor the manifest gives it — GitHub's, which is
    what every `#fragment` in the folder was written against."""
    slugs = iter(h["slug"] for h in headings)

    def tag(m):
        slug = next(slugs, None)
        return f'<h{m.group(1)} id="{html.escape(slug)}">' if slug else m.group(0)

    return re.sub(r"<h([1-6])>", tag, body_html)


def nav(manifest, here, current):
    docs = manifest["docs"]
    out = ["<nav>"]
    for section in manifest["sections"]:
        out.append(f"<h2>{html.escape(section['heading'])}</h2>")
        for path in section["pages"]:
            p = manifest["pages"][path]
            title = html.escape(p["title"] or posixpath.basename(path))
            cls = ' class="here"' if path == current else ""
            draft = '<span class="draft">draft</span>' if p["status"] == "draft" else ""
            out.append(f'<a{cls} href="{relative(here, html_path(path, docs))}">{title}{draft}</a>')
    out.append("</nav>")
    return "\n".join(out)


def banner(page):
    f = page["freshness"]
    if page["status"] == "superseded" and page["superseded_by"]:
        return '<p class="banner">Superseded: this page has been replaced.</p>'
    if f and f["state"] in ("stale", "updating"):
        n = len(f.get("commits", []))
        why = f"{n} commit(s) changed what it covers" if n else "it is older than its kind allows"
        return f'<p class="banner">This page may be out of date: {why} since it was last read against the code.</p>'
    return ""


def build(manifest_path, out):
    with open(manifest_path, encoding="utf-8") as f:
        manifest = json.load(f)
    # The repository root: the manifest's paths are relative to it.
    root = os.environ.get("LOAM_ROOT", ".")
    docs, pages = manifest["docs"], manifest["pages"]
    listed = [p for s in manifest["sections"] for p in s["pages"]]
    for path in listed:
        page = pages[path]
        here = html_path(path, docs)
        body = markdown.markdown(body_of(root, page), extensions=["tables", "fenced_code"])
        body = rewrite_links(body, page, pages, docs, here)
        body = anchor_headings(body, page["headings"])
        toc = "".join(
            f'<li style="margin-left:{(h["level"] - 2) * 1}rem"><a href="#{html.escape(h["slug"])}">{html.escape(h["text"])}</a></li>'
            for h in page["headings"]
            if h["level"] in (2, 3)
        )
        back = "".join(
            f'<li><a href="{relative(here, html_path(b["path"], docs))}">{html.escape(pages[b["path"]]["title"] or b["path"])}</a></li>'
            for b in page["backlinks"]
            if b["path"] in listed
        )
        known = manifest.get("cairn_items") or {}
        cites = "".join(
            f"<li>{n:04} {html.escape(known[str(n)]['title'])} ({html.escape(known[str(n)]['status'])})</li>"
            if str(n) in known
            else f"<li>{n:04}</li>"
            for n in page["items"]
        )
        doc = f"""<!doctype html>
<meta charset="utf-8"><title>{html.escape(page["title"] or path)}</title>
<style>{STYLE}</style>
{nav(manifest, here, path)}
<main>
{banner(page)}
{body}
{f'<div class="toc"><strong>On this page</strong><ul>{toc}</ul></div>' if toc else ""}
{f'<div class="backlinks"><strong>Linked from</strong><ul>{back}</ul></div>' if back else ""}
{f'<div class="backlinks"><strong>Why it is this way</strong><ul>{cites}</ul></div>' if cites else ""}
</main>
"""
        dest = os.path.join(out, here)
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        with open(dest, "w", encoding="utf-8") as f:
            f.write(doc)
    first = html_path(listed[0], docs) if listed else None
    with open(os.path.join(out, "index.html"), "w", encoding="utf-8") as f:
        f.write(f'<!doctype html><meta http-equiv="refresh" content="0; url={first}">' if first else "")
    print(f"built {len(listed)} page(s) into {out}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: site.py MANIFEST.json OUT-DIRECTORY")
    build(sys.argv[1], sys.argv[2])

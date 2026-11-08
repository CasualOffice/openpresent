#!/usr/bin/env python3
"""Finish the Pages site after `pages.yml` has copied `webapp/` and drawn its pictures (SITE-03).

Usage: python3 tools/site/build.py SITE_DIR [--site-url URL]

What it does, in order, to the assembled site in SITE_DIR:

1. Points every absolute URL in the marketing pages at the site's real address. The pages
   carry `https://casualoffice.github.io/openpresent/` in their canonical link, Open Graph,
   Twitter card and JSON-LD; with `--site-url` (a custom domain, say) each is rewritten.
   Relative links never change, so the site works under any base path.
2. Drops each `<!-- if:FILE -->...<!-- /if:FILE -->` block whose FILE is missing from the
   site, so a picture CI could not make never shows as a broken image. Every block dropped
   is named on stderr; a block left unclosed fails the build.
3. Writes `robots.txt` and `sitemap.xml` for the pages that exist.

Standard library only. Exits non-zero, naming the problem, on any failure.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from xml.sax.saxutils import escape

DEFAULT_URL = "https://casualoffice.github.io/openpresent"

# The pages a search engine should know, in sitemap order. A page that is not in the
# site is left out rather than listed as a 404, except the gallery: `pages-deploy` adds it
# after this runs, and always writes a page there, the gallery or a note that it failed.
SITEMAP_PAGES = [
    ("index.html", "1.0"),
    ("editor.html", "0.9"),
    ("docs.html", "0.8"),
    ("download.html", "0.8"),
    ("deploy.html", "0.6"),
]
ADDED_LATER = [("gallery/", "0.5")]

# The pages whose head names the site's absolute address.
MARKETING_PAGES = ["index.html", "docs.html", "download.html", "deploy.html"]

BLOCK = re.compile(r"<!-- if:([A-Za-z0-9_./-]+) -->(.*?)<!-- /if:\1 -->\n?", re.S)
OPEN_TAG = re.compile(r"<!-- if:([A-Za-z0-9_./-]+) -->")


def fail(msg: str) -> None:
    print(f"site build: {msg}", file=sys.stderr)
    sys.exit(1)


def conditional_blocks(html: str, site: Path, page: str) -> str:
    def keep(m: re.Match[str]) -> str:
        name = m.group(1)
        target = (site / name).resolve()
        if site.resolve() not in target.parents:
            fail(f"{page}: if:{name} points outside the site")
        if target.is_file() and target.stat().st_size > 0:
            return m.group(2).lstrip("\n")
        print(f"site build: {page}: {name} is missing, so its block is dropped", file=sys.stderr)
        return ""

    out = BLOCK.sub(keep, html)
    left = OPEN_TAG.search(out)
    if left:
        fail(f"{page}: <!-- if:{left.group(1)} --> has no matching <!-- /if:{left.group(1)} -->")
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("site", type=Path)
    ap.add_argument("--site-url", default=DEFAULT_URL)
    args = ap.parse_args()

    site: Path = args.site
    url = args.site_url.strip().rstrip("/")
    if not re.fullmatch(r"https://[A-Za-z0-9.-]+(?::[0-9]+)?(?:/[A-Za-z0-9._~-]+)*", url):
        fail(f"--site-url must be an https URL with no query or fragment, got {url!r}")
    if not (site / "index.html").is_file():
        fail(f"{site}/index.html is missing; copy webapp/ first")

    for page in MARKETING_PAGES:
        p = site / page
        if not p.is_file():
            fail(f"{p} is missing")
        html = p.read_text(encoding="utf-8")
        html = html.replace(DEFAULT_URL + "/", url + "/")
        html = conditional_blocks(html, site, page)
        p.write_text(html, encoding="utf-8")

    # Crawlers read robots.txt only at a host's root, so under a base path (GitHub's
    # `/openpresent/`) this file is ignored until a custom domain serves it at the root.
    base = re.sub(r"^https://[^/]+", "", url)
    (site / "robots.txt").write_text(
        f"User-agent: *\nAllow: /\nDisallow: {base}/pkg/\n\nSitemap: {url}/sitemap.xml\n",
        encoding="utf-8",
    )

    rows = []
    for page, prio in SITEMAP_PAGES:
        if (site / page).is_file():
            loc = url + "/" + ("" if page == "index.html" else page)
            rows.append(f"  <url><loc>{escape(loc)}</loc><priority>{prio}</priority></url>")
    for page, prio in ADDED_LATER:
        rows.append(f"  <url><loc>{escape(url + '/' + page)}</loc><priority>{prio}</priority></url>")
    (site / "sitemap.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        + "\n".join(rows)
        + "\n</urlset>\n",
        encoding="utf-8",
    )
    print(f"site build: {len(rows)} pages in sitemap.xml, site at {url}/")


if __name__ == "__main__":
    main()

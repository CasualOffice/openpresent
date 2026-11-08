#!/usr/bin/env python3
"""Build the slide gallery's index.html (FH-02).

Reads SITE/<deck>/ours/*.png (our renderer) and SITE/<deck>/lo/*.png (LibreOffice, the
reference), and writes SITE/index.html with each slide side by side. The reference is
labelled "LibreOffice", never "PowerPoint". Fails non-zero if no deck has any PNG of ours,
so an empty gallery never passes.

    python3 tools/gallery/build.py SITE [--lo-version TEXT] [--commit SHA]
"""

import argparse
import html
import pathlib
import re
import sys


def slide_key(path):
    """Order slide-2.png before slide-10.png."""
    numbers = re.findall(r"\d+", path.stem)
    return (int(numbers[-1]) if numbers else 0, path.name)


def pngs(folder):
    return sorted(folder.glob("*.png"), key=slide_key) if folder.is_dir() else []


def tail(path, lines=8):
    if not path.is_file():
        return ""
    text = path.read_text(encoding="utf8", errors="replace").splitlines()
    return "\n".join(text[-lines:])


def cell(image, site, label):
    if image is None:
        return f'<figure class="missing"><figcaption>{label}: no image</figcaption></figure>'
    src = html.escape(image.relative_to(site).as_posix(), quote=True)
    return (f'<figure><a href="{src}"><img loading="lazy" src="{src}" alt="{label}"></a>'
            f"<figcaption>{label}</figcaption></figure>")


def build(site, lo_version, commit):
    decks = sorted(p for p in site.iterdir() if p.is_dir())
    sections = []
    ours_total = 0
    for deck in decks:
        ours = pngs(deck / "ours")
        lo = pngs(deck / "lo")
        ours_total += len(ours)
        name = html.escape(deck.name)
        rows = []
        for i in range(max(len(ours), len(lo))):
            a = ours[i] if i < len(ours) else None
            b = lo[i] if i < len(lo) else None
            rows.append(f'<div class="row"><h3>Slide {i + 1}</h3>'
                        f'{cell(a, site, "OpenPresent")}{cell(b, site, "LibreOffice")}</div>')
        notes = ""
        for label, log in (("OpenPresent", "ours.log"), ("LibreOffice", "lo.log")):
            text = tail(deck / log)
            if text:
                notes += (f"<details><summary>{label} log</summary>"
                          f"<pre>{html.escape(text)}</pre></details>")
        sections.append(f'<section id="{name}"><h2>{name}</h2>'
                        f"<p>{len(ours)} slides ours, {len(lo)} LibreOffice</p>"
                        f'{notes}{"".join(rows)}</section>')
    toc = "".join(f'<li><a href="#{html.escape(d.name)}">{html.escape(d.name)}</a></li>'
                  for d in decks)
    page = f"""<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self'; style-src 'unsafe-inline'">
<title>OpenPresent slide gallery</title>
<style>
body {{ font: 14px system-ui, sans-serif; margin: 1.5rem; color: #222; }}
.row {{ display: grid; grid-template-columns: 5rem 1fr 1fr; gap: 1rem; align-items: start; margin: .75rem 0; }}
figure {{ margin: 0; }} img {{ width: 100%; border: 1px solid #ccc; }}
.missing {{ border: 1px dashed #c33; padding: 2rem; color: #c33; }}
section {{ border-top: 2px solid #888; margin-top: 2rem; }}
pre {{ background: #f4f4f4; padding: .5rem; overflow: auto; }}
</style></head><body>
<h1>OpenPresent slide gallery</h1>
<p>Left: OpenPresent's renderer. Right: the reference rendered by LibreOffice
({html.escape(lo_version)}), not PowerPoint. Commit {html.escape(commit)}.</p>
<p><a href="../index.html">OpenPresent home</a> · <a href="../editor.html">Open your own deck
in the editor</a>: it runs in your browser, and the file is never uploaded (VS-01). The site
puts this gallery at <code>gallery/</code>, beside the editor (SITE-01).</p>
<ul>{toc}</ul>
{"".join(sections)}
</body></html>
"""
    (site / "index.html").write_text(page, encoding="utf8")
    return len(decks), ours_total


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("site", type=pathlib.Path)
    parser.add_argument("--lo-version", default="version unknown")
    parser.add_argument("--commit", default="unknown")
    args = parser.parse_args()
    if not args.site.is_dir():
        print(f"gallery: {args.site} is not a directory", file=sys.stderr)
        return 1
    decks, ours = build(args.site, args.lo_version.strip(), args.commit)
    print(f"gallery: {decks} decks, {ours} slides rendered by OpenPresent")
    if ours == 0:
        print("gallery: no slide rendered by OpenPresent; refusing an empty gallery",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

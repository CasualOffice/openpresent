#!/usr/bin/env python3
"""Build the slide gallery's index.html (FH-02).

Reads SITE/<deck>/ours/*.png (our renderer) and SITE/<deck>/lo/*.png (LibreOffice, the
reference), and writes SITE/index.html with each slide side by side. The reference is
labelled "LibreOffice", never "PowerPoint". If SITE/fidelity.json exists (FH-01,
tools/fidelity/score.py), each slide shows its score and the page opens with the summary,
labelled agreement with LibreOffice, never PowerPoint fidelity. Fails non-zero if no deck has any PNG of ours,
so an empty gallery never passes.

    python3 tools/gallery/build.py SITE [--lo-version TEXT] [--commit SHA]
"""

import argparse
import html
import json
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


def load_scores(site):
    """fidelity.json from tools/fidelity/score.py, or None when scoring did not run."""
    path = site / "fidelity.json"
    if not path.is_file():
        return None
    try:
        report = json.loads(path.read_text(encoding="utf8"))
    except (OSError, ValueError):
        return None
    return report if isinstance(report, dict) else None


def pct(value):
    return f"{value * 100:.2f}%"


def slide_score(scores, deck, index):
    """The score shown beside one slide, or "" when there is none."""
    entry = (scores or {}).get("decks", {}).get(deck, {})
    for slide in entry.get("slides", []):
        if slide.get("slide") == index:
            if "error" in slide:
                return '<p class="score">Not scored: an image could not be read</p>'
            return (f'<p class="score">{pct(slide["differ"])} of pixels differ<br>'
                    f'SSIM {slide["ssim"]:.3f}</p>')
    return ""


def deck_score(scores, deck):
    entry = (scores or {}).get("decks", {}).get(deck)
    if not entry:
        return ""
    if "not_scored" in entry:
        return f'<p class="score">Not scored: {html.escape(entry["not_scored"])}</p>'
    s = entry.get("summary", {})
    if not s.get("slides"):
        return ""
    return (f'<p class="score">Agreement with LibreOffice: median {pct(s["differ_median"])} '
            f'of pixels differ (mean {pct(s["differ_mean"])}), median SSIM '
            f'{s["ssim_median"]:.3f}; {s["within_r1_1pct"]} of {s["slides"]} slides within 1%, '
            f'{s["within_5pct"]} within 5%</p>')


def summary(scores):
    if scores is None:
        return ('<p class="summary">No agreement score for this build: '
                "<code>fidelity.json</code> was not written.</p>")
    label = html.escape(str(scores.get("reference", "LibreOffice")))
    o = scores.get("overall", {})
    if not o.get("slides"):
        return f'<p class="summary">Agreement with {label}: no slide could be scored.</p>'
    return f"""<div class="summary"><h2>Agreement with {label}</h2>
<p><strong>{o["within_r1_1pct"]} of {o["slides"]}</strong> slides within 1% of pixels differing
(R1's target), <strong>{o["within_5pct"]}</strong> within 5%. Median {pct(o["differ_median"])}
of pixels differ (mean {pct(o["differ_mean"])}); median SSIM {o["ssim_median"]:.3f}
(mean {o["ssim_mean"]:.3f}).</p>
<p>This is agreement with {label}, a second opinion, not PowerPoint fidelity: where the two
differ, either may be wrong. A pixel differs when its colour is more than delta-E 10 from
every pixel of the other image within 1 px. Font differences dominate slides with small
text, since the runner's fonts are not the deck's. Numbers are in
<a href="fidelity.json">fidelity.json</a>.</p></div>"""


def build(site, lo_version, commit):
    decks = sorted(p for p in site.iterdir() if p.is_dir())
    scores = load_scores(site)
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
            rows.append(f'<div class="row"><div><h3>Slide {i + 1}</h3>'
                        f'{slide_score(scores, deck.name, i + 1)}</div>'
                        f'{cell(a, site, "OpenPresent")}{cell(b, site, "LibreOffice")}</div>')
        notes = ""
        for label, log in (("OpenPresent", "ours.log"), ("LibreOffice", "lo.log")):
            text = tail(deck / log)
            if text:
                notes += (f"<details><summary>{label} log</summary>"
                          f"<pre>{html.escape(text)}</pre></details>")
        sections.append(f'<section id="{name}"><h2>{name}</h2>'
                        f"<p>{len(ours)} slides ours, {len(lo)} LibreOffice</p>"
                        f'{deck_score(scores, deck.name)}{notes}{"".join(rows)}</section>')
    toc = "".join(f'<li><a href="#{html.escape(d.name)}">{html.escape(d.name)}</a></li>'
                  for d in decks)
    page = f"""<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self'; style-src 'unsafe-inline'">
<title>OpenPresent slide gallery</title>
<style>
body {{ font: 14px system-ui, sans-serif; margin: 1.5rem; color: #222; }}
.row {{ display: grid; grid-template-columns: 8rem 1fr 1fr; gap: 1rem; align-items: start; margin: .75rem 0; }}
figure {{ margin: 0; }} img {{ width: 100%; border: 1px solid #ccc; }}
.missing {{ border: 1px dashed #c33; padding: 2rem; color: #c33; }}
section {{ border-top: 2px solid #888; margin-top: 2rem; }}
.score {{ font-size: 12px; color: #444; }}
.summary {{ background: #f4f4f4; padding: .5rem 1rem; border-left: 4px solid #36c; }}
pre {{ background: #f4f4f4; padding: .5rem; overflow: auto; }}
</style></head><body>
<h1>OpenPresent slide gallery</h1>
<p>Left: OpenPresent's renderer. Right: the reference rendered by LibreOffice
({html.escape(lo_version)}), not PowerPoint. Commit {html.escape(commit)}.</p>
{summary(scores)}
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

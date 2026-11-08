#!/usr/bin/env python3
"""FH-01: score the gallery, our slide PNGs against LibreOffice's, and write fidelity.json.

    python3 tools/fidelity/score.py SITE [--lo-version TEXT] [--commit SHA] [--summary FILE]

SITE is the gallery folder `.github/workflows/pages.yml` fills: SITE/<deck>/ours/*.png from
`render-deck` and SITE/<deck>/lo/*.png from LibreOffice's PDF through `pdftoppm`, both at
96 dpi. Writes SITE/fidelity.json, and with --summary appends one Markdown line to FILE (the
job summary, so the score shows on the run page).

WHAT IT MEASURES. Agreement with LibreOffice, not PowerPoint fidelity: docs/51's F7, a second
opinion. Where we and LibreOffice differ, either may be wrong. Font differences dominate
slides with small text, since the runner's fonts are not the deck's. Always label results
"agreement with LibreOffice <version>".

Per slide, two numbers, each blind to what the other sees:

  differ   R1's measure (docs/06): the share of pixels whose colour is more than CIE76
           delta-E 10 from every pixel of the other image within 1 px. Taken both ways and
           joined, so content missing from either side counts. Lower is better; R1's target
           is 1% for one construct per deck. Real decks are reported, not gated (R3).
  ssim     mean SSIM on luma over 8 by 8 windows, as `compare.py` computes it. 1 is same.

Slides are paired in order. A deck whose two renders have different slide counts is not
scored, and says so, since hidden slides or a failed render would pair the wrong slides.
A LibreOffice image of another size is resized to ours first, and the slide says so.

Needs numpy and Pillow (pip, on CI only); the rest of tools/fidelity is standard library.
The stdlib `compare.py` takes about two seconds a page, too slow for the whole gallery; the
SSIM here equals its `ssim` on the same pair. OpenCalc has no pixel harness to start from
(owner instruction 42): its fidelity tools compare cell values, not pictures.
Never fails on a score: it exits non-zero only when SITE is not a folder.
"""
import argparse
import json
import pathlib
import re
import statistics
import sys

import numpy as np
from PIL import Image

DELTA_E = 10.0           # R1: a pixel differs beyond this.
NEIGHBOURHOOD = 1        # R1: ... from every pixel within 1 px of it.
R1_TARGET = 0.01
LOOSE_TARGET = 0.05
WINDOW = 8
C1, C2 = (0.01 * 255) ** 2, (0.03 * 255) ** 2
MAX_PIXELS = 40_000_000
Image.MAX_IMAGE_PIXELS = MAX_PIXELS


def slide_key(path):
    """Order slide-2.png before slide-10.png, as tools/gallery/build.py does."""
    numbers = re.findall(r"\d+", path.stem)
    return (int(numbers[-1]) if numbers else 0, path.name)


def pngs(folder):
    return sorted(folder.glob("*.png"), key=slide_key) if folder.is_dir() else []


def load(path, size=None):
    """RGB as uint8 (h, w, 3), alpha composited on white; resized to `size` (w, h) if given."""
    with Image.open(path) as image:
        image.load()
        if image.mode in ("RGBA", "LA", "P", "PA") or "transparency" in image.info:
            rgba = image.convert("RGBA")
            white = Image.new("RGBA", rgba.size, (255, 255, 255, 255))
            image = Image.alpha_composite(white, rgba)
        image = image.convert("RGB")
        resized = size is not None and image.size != size
        if resized:
            image = image.resize(size, Image.Resampling.LANCZOS)
        return np.asarray(image, dtype=np.uint8), resized


def to_lab(rgb):
    """sRGB (D65) to CIE L*a*b*, as compare.py's `lab`."""
    v = rgb.astype(np.float64) / 255.0
    v = np.where(v <= 0.04045, v / 12.92, ((v + 0.055) / 1.055) ** 2.4)
    r, g, b = v[..., 0], v[..., 1], v[..., 2]
    x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047
    y = 0.2126 * r + 0.7152 * g + 0.0722 * b
    z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883

    def f(t):
        return np.where(t > 0.008856, np.cbrt(t), 7.787 * t + 16 / 116)

    fx, fy, fz = f(x), f(y), f(z)
    return np.stack((116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)), axis=-1)


def far_from_neighbours(a, b):
    """True where a's colour is beyond DELTA_E of every pixel of b within NEIGHBOURHOOD."""
    h, w, _ = a.shape
    n = NEIGHBOURHOOD
    padded = np.pad(b, ((n, n), (n, n), (0, 0)), mode="edge")
    nearest = np.full((h, w), np.inf)
    for dy in range(2 * n + 1):
        for dx in range(2 * n + 1):
            shifted = padded[dy:dy + h, dx:dx + w]
            nearest = np.minimum(nearest, np.sqrt(((a - shifted) ** 2).sum(axis=-1)))
    return nearest > DELTA_E


def ssim(ra, rb):
    """Mean SSIM on luma over non-overlapping 8 by 8 windows, as compare.py."""
    weights = np.array([54, 183, 19], dtype=np.int32)
    la = ((ra.astype(np.int32) * weights).sum(axis=-1) >> 8).astype(np.float64)
    lb = ((rb.astype(np.int32) * weights).sum(axis=-1) >> 8).astype(np.float64)
    h, w = (la.shape[0] // WINDOW) * WINDOW, (la.shape[1] // WINDOW) * WINDOW
    if not h or not w:
        return 1.0 if np.array_equal(la, lb) else 0.0

    def blocks(m):
        return m[:h, :w].reshape(h // WINDOW, WINDOW, w // WINDOW, WINDOW).swapaxes(1, 2).reshape(
            h // WINDOW, w // WINDOW, WINDOW * WINDOW)

    a, b = blocks(la), blocks(lb)
    ma, mb = a.mean(axis=-1), b.mean(axis=-1)
    va, vb = a.var(axis=-1), b.var(axis=-1)
    cov = (a * b).mean(axis=-1) - ma * mb
    s = ((2 * ma * mb + C1) * (2 * cov + C2)) / ((ma * ma + mb * mb + C1) * (va + vb + C2))
    return float(s.mean())


def score_pair(ours_path, lo_path):
    ours, _ = load(ours_path)
    lo, resized = load(lo_path, size=(ours.shape[1], ours.shape[0]))
    if np.array_equal(ours, lo):
        result = {"differ": 0.0, "ssim": 1.0}
    else:
        lab_ours, lab_lo = to_lab(ours), to_lab(lo)
        mask = far_from_neighbours(lab_ours, lab_lo) | far_from_neighbours(lab_lo, lab_ours)
        result = {"differ": round(float(mask.mean()), 6), "ssim": round(ssim(ours, lo), 5)}
    if resized:
        result["note"] = "LibreOffice image resized to ours"
    return result


def summarize(differs, ssims):
    if not differs:
        return {"slides": 0}
    return {
        "slides": len(differs),
        "differ_mean": round(statistics.fmean(differs), 6),
        "differ_median": round(statistics.median(differs), 6),
        "ssim_mean": round(statistics.fmean(ssims), 5),
        "ssim_median": round(statistics.median(ssims), 5),
        "within_r1_1pct": sum(d <= R1_TARGET for d in differs),
        "within_5pct": sum(d <= LOOSE_TARGET for d in differs),
    }


def score_site(site):
    decks, all_d, all_s = {}, [], []
    for deck in sorted(p for p in site.iterdir() if p.is_dir()):
        ours, lo = pngs(deck / "ours"), pngs(deck / "lo")
        entry = {"ours_slides": len(ours), "lo_slides": len(lo)}
        if not ours or not lo or len(ours) != len(lo):
            entry["not_scored"] = (f"{len(ours)} slides ours, {len(lo)} LibreOffice; "
                                   "slides are paired only when the counts agree")
            decks[deck.name] = entry
            continue
        slides, d_list, s_list = [], [], []
        for index, (a, b) in enumerate(zip(ours, lo), start=1):
            try:
                result = score_pair(a, b)
            except (OSError, ValueError, Image.DecompressionBombError) as error:
                slides.append({"slide": index, "error": type(error).__name__})
                continue
            slides.append({"slide": index, "ours": a.relative_to(site).as_posix(),
                           "lo": b.relative_to(site).as_posix(), **result})
            d_list.append(result["differ"])
            s_list.append(result["ssim"])
        entry["slides"] = slides
        entry["summary"] = summarize(d_list, s_list)
        decks[deck.name] = entry
        all_d += d_list
        all_s += s_list
    return decks, summarize(all_d, all_s)


def lo_label(version):
    found = re.search(r"\d+(?:\.\d+)+", version or "")
    return f"LibreOffice {found.group(0)}" if found else "LibreOffice (version unknown)"


def summary_line(overall, label, decks):
    skipped = sum("not_scored" in d for d in decks.values())
    if not overall.get("slides"):
        return f"**Agreement with {label}:** no slide could be scored ({skipped} decks not scored)."
    n = overall["slides"]
    return (f"**Agreement with {label}** (not PowerPoint fidelity), {n} slides: median "
            f"{overall['differ_median'] * 100:.2f}% of pixels differ (mean "
            f"{overall['differ_mean'] * 100:.2f}%), median SSIM {overall['ssim_median']:.3f}; "
            f"{overall['within_r1_1pct']} of {n} within R1's 1%, {overall['within_5pct']} "
            f"within 5%; {skipped} decks not scored.")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("site", type=pathlib.Path)
    parser.add_argument("--lo-version", default="")
    parser.add_argument("--commit", default="unknown")
    parser.add_argument("--summary", type=pathlib.Path, help="append a Markdown line here")
    args = parser.parse_args()
    if not args.site.is_dir():
        print(f"score: {args.site} is not a directory", file=sys.stderr)
        return 1
    decks, overall = score_site(args.site)
    label = lo_label(args.lo_version)
    report = {
        "measures": f"agreement with {label}, not PowerPoint fidelity (docs/51 F7)",
        "reference": label,
        "reference_version": args.lo_version.strip(),
        "commit": args.commit,
        "method": {"differ": f"share of pixels beyond CIE76 delta-E {DELTA_E:g} of every pixel "
                             f"of the other image within {NEIGHBOURHOOD} px, both ways (R1)",
                   "ssim": f"mean SSIM on luma, {WINDOW} by {WINDOW} windows",
                   "r1_target": R1_TARGET, "loose_target": LOOSE_TARGET},
        "overall": overall,
        "decks": decks,
    }
    (args.site / "fidelity.json").write_text(json.dumps(report, indent=1, sort_keys=True) + "\n",
                                             encoding="utf8")
    line = summary_line(overall, label, decks)
    print(line)
    if args.summary:
        with open(args.summary, "a", encoding="utf8") as out:
            out.write(line + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())

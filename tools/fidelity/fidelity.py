#!/usr/bin/env python3
"""SPK-02: the fidelity pipeline. Deck to PDF and PNG, two renders compared, text positions compared.

    fidelity.py render DECK --out DIR
    fidelity.py compare RENDER_A RENDER_B --out DIR [--stem NAME] [--thresholds FILE]
    fidelity.py run --out DIR SPEC [SPEC...] [--reference PRODUCER]

A SPEC is `deck` (render twice, measuring how repeatable LibreOffice is) or `deck=candidate`
(render both and compare, as a later engine export would be compared with its source).

WHAT A RESULT MEANS. docs/51 is the rule here:

    F7, second opinion   any comparison with LibreOffice on either side. A difference is a
                         question, not a verdict, and agreement says nothing about PowerPoint.
    F6, pixels           only a non-LibreOffice candidate against a reference PowerPoint itself
                         exported. No such reference exists yet (DEC-04).

So results never say "pass" or "match" for F7, and report.md leads with this.

REFERENCE SETS, by producer, added by hand later:

    fixtures/references/<producer>/<deck stem>/deck.pdf        and or
    fixtures/references/<producer>/<deck stem>/page-001.png    ...
    fixtures/references/<producer>/<deck stem>/meta.json       optional: {"producer_version": ..., "exported_by": ...}

<producer> starts `powerpoint` (powerpoint-win-16.90, powerpoint-mac-16.90) for the pixel target;
anything else (keynote-14, google-slides, libreoffice-26.8) stays a second opinion. A set named
powerpoint whose PDF says LibreOffice wrote it is refused. A PNG-only set has no text comparison.

THRESHOLDS are per fixture and from evidence (docs/51). --thresholds names a TSV of
`stem  min_ssim  min_region_ssim  max_changed_pixels  max_text_move_pt`. A deck with no
row is measured and given no result. "Flagged" regions are only places to look.

Standard library plus soffice, pdftoppm, pdftotext, pdfinfo and pdffonts. No image library.
"""
import argparse
import json
import pathlib
import shutil
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import compare as cmp  # noqa: E402
import render as rnd  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parents[2]
BANNER_F7 = ("SECOND OPINION (F7). LibreOffice is on at least one side of this comparison. "
             "Agreement here says nothing about PowerPoint, and a difference is a question, not a verdict.")
BANNER_F6 = ("PIXELS (F6). Candidate against a reference exported by PowerPoint. "
             "Holds only for the PowerPoint version the reference names.")


def tier_of(a, b):
    """F6 needs PowerPoint on one side and no LibreOffice on the other. Everything else is F7."""
    producers = {a.get("producer", ""), b.get("producer", "")}
    has_powerpoint = any(p.startswith("powerpoint") for p in producers)
    has_libreoffice = any(p.startswith("libreoffice") for p in producers)
    return "F6-pixels" if has_powerpoint and not has_libreoffice else rnd.SECOND_OPINION


def read_meta(folder):
    path = pathlib.Path(folder) / "meta.json"
    return json.loads(path.read_text(encoding="utf8")) if path.exists() else {}


def pages_in(folder):
    return sorted(pathlib.Path(folder).glob("page-*.png"))


def load_reference(references, producer, stem, work):
    """A reference set as a render folder. A PDF gets rendered here by pdftoppm, at the same dpi."""
    source = pathlib.Path(references) / producer / stem
    if not source.is_dir():
        return None
    target = pathlib.Path(work) / f"reference-{producer}-{stem}"
    target.mkdir(parents=True, exist_ok=True)
    meta = read_meta(source)
    pdf = source / "deck.pdf"
    if pdf.exists():
        info = rnd.pdf_set(pdf, target)
        if producer.startswith("powerpoint") and "libreoffice" in (info["pdf_producer"] + info["pdf_creator"]).lower():
            raise rnd.Refused(f"{source}: named {producer}, but LibreOffice wrote the PDF ({info['pdf_producer']})")
        meta.update(info)
    pngs = pages_in(source)
    if pngs and not pdf.exists():
        for p in pngs:
            shutil.copyfile(p, target / p.name)
        meta["pages"] = len(pngs)
        meta["text_compared"] = False
    if not pngs and not pdf.exists():
        raise rnd.Refused(f"{source}: neither deck.pdf nor page-NNN.png")
    meta["producer"] = producer
    (target / "meta.json").write_text(json.dumps(meta, indent=1, sort_keys=True), encoding="utf8")
    return target


def load_thresholds(path):
    if not path:
        return {}
    table = {}
    for line in pathlib.Path(path).read_text(encoding="utf8").splitlines():
        if line.startswith(("#", "stem\t")) or not line.strip():
            continue
        stem, *values = line.split("\t")
        table[stem] = dict(zip(("min_ssim", "min_region_ssim", "max_changed_pixels", "max_text_move_pt"), map(float, values)))
    return table


def assess(result, limits):
    """Measured against this fixture's own thresholds. Words differ between tiers on purpose."""
    second_opinion = result["tier"] == rnd.SECOND_OPINION
    if limits is None:
        return "no threshold: measured only"
    over = []
    if result["pages_a"] != result["pages_b"]:
        over.append(f"pages {result['pages_a']} and {result['pages_b']}")
    for page in result["pages"]:
        if "image" not in page:
            continue
        img = page["image"]
        if img["ssim"] < limits["min_ssim"]:
            over.append(f"page {page['page']} ssim {img['ssim']}")
        if img["worst_region_ssim"] < limits["min_region_ssim"]:
            over.append(f"page {page['page']} region ssim {img['worst_region_ssim']}")
        if img["changed_pixels"] > limits["max_changed_pixels"]:
            over.append(f"page {page['page']} changed {img['changed_pixels']}")
        text = page.get("text")
        if text and text["largest_move_pt"] > limits["max_text_move_pt"]:
            over.append(f"page {page['page']} text moved {text['largest_move_pt']}pt")
    result["over_threshold"] = over
    if second_opinion:
        return "differs from the other render: a question, not a verdict" if over else "within thresholds of the other render, which says nothing about PowerPoint"
    return "outside this fixture's thresholds" if over else "within this fixture's thresholds"


def compare_sets(a, b, out, limits=None):
    """Two render folders, page by page. Writes diff PNGs into out and returns a result."""
    a, b, out = pathlib.Path(a), pathlib.Path(b), pathlib.Path(out)
    out.mkdir(parents=True, exist_ok=True)
    meta_a, meta_b = read_meta(a), read_meta(b)
    tier = tier_of(meta_a, meta_b)
    pa, pb = pages_in(a), pages_in(b)
    words_a = json.loads((a / "words.json").read_text(encoding="utf8")) if (a / "words.json").exists() else None
    words_b = json.loads((b / "words.json").read_text(encoding="utf8")) if (b / "words.json").exists() else None
    pages = []
    for i in range(max(len(pa), len(pb))):
        page = {"page": i + 1}
        if i >= len(pa) or i >= len(pb):
            page["note"] = "only in " + ("the second" if i >= len(pa) else "the first")
            pages.append(page)
            continue
        page["image"] = cmp.compare_files(pa[i], pb[i], diff_path=out / f"diff-{i + 1:03d}.png")
        if words_a is not None and words_b is not None and i < len(words_a) and i < len(words_b):
            page["text"] = cmp.compare_words(words_a[i], words_b[i])
        pages.append(page)
    fonts_a, fonts_b = meta_a.get("fonts", []), meta_b.get("fonts", [])
    result = {"tier": tier, "banner": BANNER_F7 if tier == rnd.SECOND_OPINION else BANNER_F6,
              "a": {k: meta_a.get(k) for k in ("producer", "producer_version", "deck", "deck_sha256", "isolation")},
              "b": {k: meta_b.get(k) for k in ("producer", "producer_version", "deck", "deck_sha256", "isolation")},
              "pages_a": len(pa), "pages_b": len(pb), "pages": pages, "fonts_a": fonts_a, "fonts_b": fonts_b,
              "fonts_differ": sorted(set(fonts_a) ^ set(fonts_b))}
    result["summary"] = summarise(result)
    result["assessment"] = assess(result, limits)
    (out / "comparison.json").write_text(json.dumps(result, indent=1, ensure_ascii=False), encoding="utf8")
    return result


def summarise(result):
    images = [p["image"] for p in result["pages"] if "image" in p]
    texts = [p["text"] for p in result["pages"] if "text" in p]
    return {
        "pages_compared": len(images),
        "pages_unpaired": sum(1 for p in result["pages"] if "image" not in p),
        "pages_identical": sum(1 for i in images if i["identical"]),
        "lowest_ssim": min((i["ssim"] for i in images), default=None),
        "worst_region_ssim": min((i["worst_region_ssim"] for i in images), default=None),
        "worst_region_changed": max((i["worst_region_changed"] for i in images), default=None),
        "most_changed_pixels": max((i["changed_pixels"] for i in images), default=None),
        "regions_flagged": sum(len(i["regions_flagged"]) for i in images),
        "text_compared": bool(texts),
        "words_missing": sum(len(t["missing"]) for t in texts),
        "words_extra": sum(len(t["extra"]) for t in texts),
        "words_moved": sum(len(t["moved"]) for t in texts),
        "largest_move_pt": max((t["largest_move_pt"] for t in texts), default=None),
    }


def markdown(rows, reference_note=None):
    lines = ["# Fidelity run", "", f"**{BANNER_F7}**" if any(r["tier"] == rnd.SECOND_OPINION for r in rows) else f"**{BANNER_F6}**", ""]
    if reference_note:
        lines += [reference_note, ""]
    lines += ["| Comparison | Tier | Pages | Identical | Lowest SSIM | Worst region SSIM | Most changed px | Regions flagged | Words missing, extra, moved | Largest move pt | Assessment |",
              "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | --- |"]
    for r in rows:
        s = r["summary"]
        pages = f"{r['pages_a']}, {r['pages_b']}" if r["pages_a"] != r["pages_b"] else str(r["pages_a"])
        words = f"{s['words_missing']}, {s['words_extra']}, {s['words_moved']}" if s["text_compared"] else "not compared"
        lines.append(f"| {r['label']} | {r['tier']} | {pages} | {s['pages_identical']} | {s['lowest_ssim']} | {s['worst_region_ssim']} | "
                     f"{s['most_changed_pixels']} | {s['regions_flagged']} | {words} | {s['largest_move_pt']} | {r['assessment']} |")
    return "\n".join(lines) + "\n"


def cmd_render(args):
    meta = rnd.render(args.deck, args.out, args.dpi, args.allow_no_network_denial)
    print(f"{args.deck}: {meta['pages']} pages in {args.out}. Second opinion: LibreOffice {meta['producer_version']}; isolation: {meta['isolation']}")
    return 0


def cmd_compare(args):
    limits = load_thresholds(args.thresholds).get(args.stem)
    result = compare_sets(args.a, args.b, args.out, limits)
    print(result["banner"])
    print(json.dumps(result["summary"], indent=1))
    print(f"Assessment: {result['assessment']}")
    return 0


def cmd_run(args):
    out = pathlib.Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    thresholds = load_thresholds(args.thresholds)
    rows, refused = [], []
    references = pathlib.Path(args.references)
    reference_note = None
    if args.reference:
        reference_note = f"Reference producer: `{args.reference}`, from {references}."
    for index, spec in enumerate(args.specs):
        deck, _, candidate = spec.partition("=")
        stem = pathlib.Path(deck).stem
        folder = out / f"{index:02d}-{stem}"
        try:
            first = folder / "render-a"
            rnd.render(deck, first, args.dpi, args.allow_no_network_denial)
            second = folder / "render-b"
            rnd.render(candidate or deck, second, args.dpi, args.allow_no_network_denial)
            label = f"{stem} vs {pathlib.Path(candidate).name}" if candidate else f"{stem} vs itself (repeatability)"
            result = compare_sets(first, second, folder / "compare", thresholds.get(stem))
            result["label"] = label
            rows.append(result)
            s = result["summary"]
            print(f"{label}: ssim {s['lowest_ssim']}, region {s['worst_region_ssim']}, flagged {s['regions_flagged']}, "
                  f"words {s['words_missing']}/{s['words_extra']}/{s['words_moved']}")
            if args.reference:
                with tempfile.TemporaryDirectory() as work:
                    ref = load_reference(references, args.reference, stem, work)
                    if ref is None:
                        print(f"{stem}: no {args.reference} reference")
                    else:
                        result = compare_sets(ref, first, folder / f"against-{args.reference}", thresholds.get(stem))
                        result["label"] = f"{stem}: {args.reference} reference vs LibreOffice"
                        rows.append(result)
        except rnd.Refused as err:
            refused.append({"spec": spec, "reason": str(err)})
            print(f"REFUSED {spec}: {err}")
    report = {"rows": rows, "refused": refused}
    (out / "report.json").write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding="utf8")
    md = markdown(rows, reference_note) if rows else "# Fidelity run\n\nNothing rendered.\n"
    if refused:
        md += "\n## Refused\n\n" + "".join(f"- `{r['spec']}`: {r['reason']}\n" for r in refused)
    (out / "report.md").write_text(md, encoding="utf8")
    print(f"\n{md}")
    return 0 if rows and not refused else 1


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="command", required=True)
    r = sub.add_parser("render")
    r.add_argument("deck")
    r.add_argument("--out", required=True)
    r.add_argument("--dpi", type=int, default=96)
    r.add_argument("--allow-no-network-denial", action="store_true", help="trusted decks only")
    r.set_defaults(func=cmd_render)
    c = sub.add_parser("compare")
    c.add_argument("a")
    c.add_argument("b")
    c.add_argument("--out", required=True)
    c.add_argument("--stem", default="")
    c.add_argument("--thresholds")
    c.set_defaults(func=cmd_compare)
    u = sub.add_parser("run")
    u.add_argument("specs", nargs="+", metavar="SPEC")
    u.add_argument("--out", required=True)
    u.add_argument("--dpi", type=int, default=96)
    u.add_argument("--reference", help="producer folder under fixtures/references")
    u.add_argument("--references", default=str(REPO / "fixtures/references"))
    u.add_argument("--thresholds")
    u.add_argument("--allow-no-network-denial", action="store_true", help="trusted decks only")
    u.set_defaults(func=cmd_run)
    args = ap.parse_args(argv)
    try:
        return args.func(args)
    except rnd.Refused as err:
        print(f"REFUSED: {err}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())

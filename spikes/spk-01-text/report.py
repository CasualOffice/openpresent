#!/usr/bin/env python3
"""Turns results/*.json into the Markdown tables quoted in docs/64. Numbers are only ever
copied from a results file through here, so the doc cannot state one that was not measured."""
import json
import pathlib
import statistics as st

R = pathlib.Path(__file__).resolve().parent / "results"


def load(n):
    p = R / n
    return json.loads(p.read_text()) if p.exists() else None


def kb(n):
    return f"{n / 1024:,.0f}"


def sizes():
    d = load("sizes.json")
    print("### Sizes (KiB)\n")
    print("| Build | Profile | wasm raw | gzip -9 | Brotli q11 | JS glue raw / Brotli |\n| --- | --- | ---: | ---: | ---: | ---: |")
    for r in d["rows"]:
        print(f"| {r['row']} | {r['opt']} | {kb(r['raw'])} | {kb(r['gzip'])} | {kb(r['brotli'])} | {r['glue_js_raw'] / 1024:.1f} / {r['glue_js_brotli'] / 1024:.1f} |")
    print("\n| Font file | raw | gzip -9 | Brotli q11 |\n| --- | ---: | ---: | ---: |")
    for f in d["fonts"]:
        print(f"| {f['font']} | {kb(f['raw'])} | {kb(f['gzip'])} | {kb(f['brotli'])} |")
    print()


def throughput(env, title):
    d = load(f"{env}-throughput.json")
    if not d:
        return
    print(f"### Throughput, {title}: 2,000 characters, 16px, 600px measure, ms\n")
    print("| Case | Bundle | Glyphs | Lines | Bidi + itemize | Shape incl. itemize | Line break opportunities | Full layout median | p95 | Layout + JS copy median | chars/ms | First layout, fresh page, median of 5 |")
    print("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for tag in ("o3", "small"):
        for r in d["warm"][tag]:
            cold = d["cold"].get(f"{tag}:{r['name']}", [])
            print(f"| {r['name']} | {tag} | {r['glyphs']} | {r['lines']} | {r['itemize_ms']['median']:.3f} | {r['shape_incl_itemize_ms']['median']:.3f} | {r['breaks_ms']['median']:.3f} | {r['full_ms']['median']:.3f} | {r['full_ms']['p95']:.3f} | {r['full_plus_js_copy_ms']['median']:.3f} | {r['chars_per_ms']:,.0f} | {st.median(cold):.2f} |")
    print()
    for tag in ("o3", "small"):
        inst = [x["ms"] for x in d["instantiate"] if x["tag"] == tag]
        regs = [x for x in d["register"] if x["tag"] == tag]
        print(f"{tag}: instantiate median {st.median(inst):.1f} ms; register_font median " + ", ".join(f"{k} {st.median([x[k] for x in regs]):.2f} ms" for k in ("carlito", "liberation", "arabic", "cjk")))
    print()


def node():
    d = load("node-throughput.json")
    if not d:
        return
    print(f"### Throughput, Node {d['node']} (same bundle, V8 only), ms\n")
    print("| Case | Bundle | Full layout median | p95 | chars/ms |\n| --- | --- | ---: | ---: | ---: |")
    for tag in ("o3", "small"):
        for r in d["warm"][tag]:
            print(f"| {r['name']} | {tag} | {r['full_ms']['median']:.3f} | {r['full_ms']['p95']:.3f} | {r['chars_per_ms']:,.0f} |")
    print()


NAMES = {
    "glyph_path": "Path2D per glyph",
    "shape_path": "Path2D per shape",
    "cluster_filltext": "fillText per cluster",
    "line_filltext": "fillText per line",
}


def frames(env, title):
    d = load(f"{env}-frames.json")
    if not d:
        return
    print(f"### Frame time by readback, {title}, ms\n")
    print("Each row is 5 fresh browsers. Cold is that browser's first frame; cached is 40 later frames in each, median of medians. Scene layout is not in the frame.\n")
    print("| Text | DPR | Painter | Glyphs | Distinct | Scene layout | Outline to Path2D | Cold frame median (min to max) | Cached frame median | Cached p95 | Cached submit |")
    print("| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for r in d:
        s = r["samples"]
        first = [x["first_frame_ms"] for x in s]
        print(f"| {r['kind']} | {r['dpr']} | {NAMES[r['mode']]} | {s[0]['glyphs']} | {s[0]['unique_glyphs']} | {st.median(x['scene_layout_ms'] for x in s):.1f} | {st.median(x['path_load_ms'] for x in s):.1f} | {st.median(first):.1f} ({min(first):.1f} to {max(first):.1f}) | {st.median(x['cached_total_ms']['median'] for x in s):.2f} | {st.median(x['cached_total_ms']['p95'] for x in s):.2f} | {st.median(x['cached_submit_ms']['median'] for x in s):.2f} |")
    print()


def gpu(env, title):
    d = load(f"{env}-gpu.json")
    if not d:
        return
    print(f"### Frame time, fresh canvas per sample, {title}, ms\n")
    print("3 fresh browsers per row; 25 cached samples each, median of medians. Each sample includes a new canvas and one readback, given as Empty.\n")
    print("| Text | DPR | Painter | Glyphs | Distinct | Empty | Outline to Path2D | Cold frame median (min to max) | Cached median | Cached p95 | Cached minus empty |")
    print("| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for r in d:
        s = r["samples"]
        first = [x["first_frame_ms"] for x in s]
        em = st.median(x["empty_ms"]["median"] for x in s)
        c = st.median(x["cached_ms"]["median"] for x in s)
        print(f"| {r['kind']} | {r['dpr']} | {NAMES[r['mode']]} | {s[0]['glyphs']} | {s[0]['unique_glyphs']} | {em:.2f} | {st.median(x['path_load_ms'] for x in s):.1f} | {st.median(first):.1f} ({min(first):.1f} to {max(first):.1f}) | {c:.2f} | {st.median(x['cached_ms']['p95'] for x in s):.2f} | {c - em:.2f} |")
    print()


def raf(env, title):
    d = load(f"{env}-raf.json")
    if not d:
        return
    print(f"### Frame time by rAF interval, {title}, ms (no readback, uncapped flags)\n")
    print("| Text | DPR | Painter | rAF interval median | p95 | JS submit median | Cold: submit | Cold: until 2 rAFs later |")
    print("| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |")
    for r in d:
        print(f"| {r['kind']} | {r['dpr']} | {NAMES[r['mode']]} | {r['raf_interval_ms']['median']:.2f} | {r['raf_interval_ms']['p95']:.2f} | {r['submit_ms']['median']:.2f} | {r['first_submit_ms']:.1f} | {r['first_two_rafs_ms']:.1f} |")
    print()


def appearance(env):
    d = load(f"{env}-appearance.json")
    if not d:
        return
    print(f"### Appearance metrics, {env}\n")
    print("| Panel | Font | Size | Device px | Ink fillText / Path2D | Mean abs lum diff (0-255) | Peak contrast Path2D, fillText | LCD pixels Path2D, fillText (opaque canvas) |")
    print("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for m in d["metrics"]["opaque"]:
        print(f"| {m['panel']} | {m['font']} | {m['pt']}pt {m['scale']}x | {m['device_px']:.1f} | {m['ink_ratio_cluster_over_path']:.2f} | {m['mean_abs_lum_diff_cluster']:.1f} | {m['path']['peak_contrast']:.2f}, {m['cluster']['peak_contrast']:.2f} | {m['path']['chroma_px']}, {m['cluster']['chroma_px']} |")
    print("\nProbe: centroid shift of one glyph against the requested sub-pixel offset\n")
    for p in d["probe"]:
        print(f"- {p['font']} {p['method']} {p['axis']}: " + ", ".join(f"{x:.2f}" for x in p["shifts"]))
    print()


def agreement(env):
    d = load(f"{env}-agreement.json")
    if not d:
        return
    print(f"### Engine against fillText, {env}\n")
    print("| Case | Clusters | Whole-text width diff, browser - engine, px, by setting | First divergence | Clusters adv > 0.01 px | Mean, max adv diff | Words | Words adv > 0.05 px | Max word diff |")
    print("| --- | ---: | --- | --- | ---: | --- | ---: | ---: | ---: |")
    for r in d:
        w = "; ".join(f"{k.replace('kerning=', 'k=').replace('textRendering=', 'r=')}: {v:.3f}" for k, v in r["whole_text_width_diff_px"].items())
        fd = r["first_divergence"]
        print(f"| {r['name']} | {r['clusters']} | {w} | {('char ' + str(fd['at_char']) + ' `' + fd['context'] + '`') if fd else 'none'} | {r['cluster_adv_differs_gt_0_01px']} | {r['cluster_adv_mean_abs']:.3f}, {r['cluster_adv_max']:.3f} | {r['words']} | {r['word_adv_differs_gt_0_05px']} | {r['word_adv_max']:.3f} |")
    print("\n| Case | Wrapping | Distinct clusters | Wrong form | Occurrences | Wrong form | Examples |\n| --- | --- | ---: | ---: | ---: | ---: | --- |")
    for r in d:
        for k, v in r["shape"].items():
            print(f"| {r['name']} | {k} | {v['distinct']} | {v['distinct_wrong_form']} | {v['occurrences']} | {v['occurrences_wrong_form']} | {' '.join(v['examples'][:6])} |")
    print()


if __name__ == "__main__":
    for e in ("chrome-info", "chrome-gpu-info", "shell-info", "linux-info"):
        i = load(e + ".json")
        if i:
            print(e, i.get("version"), i.get("webgl_renderer"), i.get("chrome_gpu"), "idle rAF", i["idle_raf_ms"]["median"], "\n")
    sizes()
    throughput("chrome", "installed Chrome, new headless, GPU")
    throughput("shell", "Playwright headless shell, software")
    node()
    frames("chrome", "installed Chrome, new headless, Metal GPU")
    frames("shell", "Playwright headless shell, SwiftShader")
    gpu("chrome", "installed Chrome, new headless, Metal GPU")
    gpu("shell", "Playwright headless shell, SwiftShader")
    raf("chrome", "installed Chrome, new headless, Metal GPU")
    raf("shell", "Playwright headless shell, SwiftShader")
    for e in ("chrome", "shell", "linux"):
        appearance(e)
        agreement(e)

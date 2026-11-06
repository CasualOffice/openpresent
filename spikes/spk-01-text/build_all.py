#!/usr/bin/env python3
"""Builds every row of the size ladder with wasm-pack and records raw, gzip and Brotli
sizes in results/sizes.json. Also builds the two benchmark bundles into harness/pkg."""
import glob
import gzip
import json
import os
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
BUILD = ROOT / "build"
UX14 = "layout,lb-uax14"
ROWS = [
    ("baseline (bindgen only)", ""),
    ("+ skrifa (outlines)", "outline"),
    ("+ harfrust (shaping)", "shape"),
    ("+ unicode-bidi, unicode-script", "shape,itemize"),
    ("+ unicode-linebreak = full stack", UX14),
    ("full stack, icu_segmenter instead (no complex scripts)", "layout,lb-icu"),
    ("full stack, icu_segmenter instead (auto: Thai etc.)", "layout,lb-icu,lb-icu-auto"),
    ("full stack + Carlito Regular embedded", UX14 + ",embed-carlito"),
    ("full stack + Carlito and Liberation Sans Regular embedded", UX14 + ",embed-carlito,embed-liberation"),
]
OPT = [
    # label, cargo profile, wasm-opt args
    ("release, no wasm-opt", "release", None),
    ("release, wasm-opt -O", "release", ["-O"]),
    ("small (opt-level s, fat LTO), wasm-opt -Oz", "small", ["-Oz"]),
    ("smallest (opt-level z, fat LTO, overflow checks off), wasm-opt -Oz", "smallest", ["-Oz"]),
]
WASM_OPT_FEATURES = ["--all-features"]


def wasm_opt():
    env = os.environ.get("WASM_OPT")
    if env:
        return env
    found = shutil.which("wasm-opt")
    if found:
        return found
    hits = glob.glob(os.path.expanduser("~/Library/Caches/.wasm-pack/wasm-opt-*/bin/wasm-opt"))
    if not hits:
        raise SystemExit("no wasm-opt: set WASM_OPT")
    return hits[0]


def brotli(path):
    out = subprocess.run(["brotli", "-q", "11", "-c", str(path)], capture_output=True, check=True).stdout
    return len(out)


def gz(path):
    return len(gzip.compress(path.read_bytes(), 9))


def build(features, profile, out):
    cmd = ["wasm-pack", "build", "crates/wasm", "--target", "web", "--out-dir", str(out), "--no-typescript", "--no-pack"]
    if profile == "release":
        cmd.append("--release")
    else:
        cmd += ["--profile", profile]
    if features:
        cmd += ["--", "--features", features]
    subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True)
    return out / "spk_text_wasm_bg.wasm"


def optimize(wasm, args):
    if args is None:
        return wasm
    dst = wasm.with_name("opt.wasm")
    subprocess.run([wasm_opt(), str(wasm), "-o", str(dst), *WASM_OPT_FEATURES, *args], check=True)
    return dst


def main():
    BUILD.mkdir(exist_ok=True)
    rows = []
    for name, features in ROWS:
        cached = {}
        for label, profile, args in OPT:
            out = BUILD / (features.replace(",", "_") or "baseline") / profile
            if profile not in cached:
                cached[profile] = build(features, profile, out)
            w = optimize(cached[profile], args)
            glue = out / "spk_text_wasm.js"
            rows.append({
                "row": name, "features": features, "opt": label, "raw": w.stat().st_size,
                "gzip": gz(w), "brotli": brotli(w), "glue_js_raw": glue.stat().st_size,
                "glue_js_brotli": brotli(glue),
            })
            print(rows[-1], flush=True)
    fonts = []
    for f in sorted((ROOT / "fonts").iterdir()):
        fonts.append({"font": f.name, "raw": f.stat().st_size, "gzip": gz(f), "brotli": brotli(f)})
    (ROOT / "results").mkdir(exist_ok=True)
    (ROOT / "results/sizes.json").write_text(json.dumps({"rows": rows, "fonts": fonts}, indent=1))

    pkg = ROOT / "harness/pkg"
    for tag, profile, args in (("o3", "release", ["-O"]), ("small", "small", ["-Oz"])):
        out = pkg / tag
        w = build(UX14 + ",bench", profile, out)
        shutil.move(optimize(w, args), w)
    print("done")


if __name__ == "__main__":
    main()

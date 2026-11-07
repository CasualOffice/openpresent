#!/usr/bin/env python3
"""Write fuzz/seeds/charts/ from the chart fixtures: one small part of each kind. Same input,
same files; rerun when a chart fixture changes."""
import pathlib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
SEEDS = [
    ("generated/chart-families.pptx", "ppt/charts/chart11.xml", "bar"),
    ("generated/chart-families.pptx", "ppt/charts/chart7.xml", "scatter"),
    ("generated/chart-families.pptx", "ppt/charts/chart2.xml", "area3d"),
    ("generated/chart-ex-layouts.pptx", "ppt/charts/chartEx1.xml", "ex"),
    ("generated/chart-signed-axis.pptx", "ppt/charts/chart1.xml", "signed"),
    ("generated/chart-every-row-classic.pptx", "ppt/charts/style1.xml", "style"),
    ("generated/chart-every-row-classic.pptx", "ppt/charts/colors1.xml", "colors"),
    ("generated/chart-every-row-classic.pptx", "ppt/drawings/drawing1.xml", "drawing"),
    ("hostile/chart-bad-structure.bin", "ppt/charts/chart1.xml", "bad-structure"),
]


def main():
    out = ROOT / "fuzz/seeds/charts"
    out.mkdir(parents=True, exist_ok=True)
    for deck, part, name in SEEDS:
        with zipfile.ZipFile(ROOT / "fixtures" / deck) as z:
            (out / f"chart-{name}.xml").write_bytes(z.read(part))


if __name__ == "__main__":
    main()

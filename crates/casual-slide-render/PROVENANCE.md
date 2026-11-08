# Provenance

Started from `crates/casual-calc-render` and `crates/casual-calc-layout/src/display.rs` in
OpenCalc (`../opencalc`) at commit `b23c67c`, on 2026-11-08 (owner instruction 42). Same
owner, Apache-2.0.

Copied, and changed only where slides need it:

- `src/images.rs` from `casual-calc-render/src/images.rs`: `ImageSource`, `NoImages`,
  `UndrawnReason`, `ImageReport`, the magic-number sniffing and the decode bound checked
  against the header before any allocation. Changes: JPEG is the one codec beyond PNG
  (ADR-019), so GIF, BMP, TIFF and WebP are reported by name rather than decoded; header
  reads use checked slicing; the PDF glyph report and `UnsupportedByBackend` are gone,
  as there is no PDF backend yet; the pixel bound is 40 megapixels, since slides carry
  photographs where workbooks carry logos.
- `src/raster.rs` follows `render_pixmap_with_images`: a white ground, the list in
  painter's order, pictures from an `ImageSource`, and a report of what did not reach the
  surface. The grid-specific drawing (gridlines, cell borders, merged regions, data bars)
  is not copied; every slide item is a path with a transform.
- `src/display.rs` follows OpenCalc's display list contract (its ADR-008): backend-neutral,
  deterministic, painter's order, pictures by package path. The item set is new, per
  `docs/42` §Display list, because a slide is vector art rather than a grid.

- `src/scene/chart.rs` ports OpenCalc's plotter, `casual-calc-layout/src/chart.rs` at
  `b23c67c`: the frame, title and legend taken off the plot first, the value extent that
  holds zero and stacks each sign apart, bars from the zero line, the pie from twelve
  o'clock, the accent palette varied past six series (`vary`, `ACCENT_VARIANTS`), and
  category labels thinned to fit. Changes: EMU for twips (`PX` is one pixel at 96 dpi);
  a series' or point's own `spPr` colour wins over the palette; `gapWidth` sets bar
  width; horizontal bars, line and area series, round value-axis ticks with gridlines
  (PowerPoint's look, where OpenCalc labels the two extremes), and doughnut holes from
  `holeSize`. The cached values are read from `c:chartSpace`; no workbook is opened.

New here: `src/scene.rs`, `src/scene/table.rs` (a slide's shapes to a display list) and `src/deck.rs`.

Not yet copied: glyph outlining and shaping (`shape.rs`, the text crate's, lane B), the
PDF writer (`pdf.rs`, `subset.rs`).

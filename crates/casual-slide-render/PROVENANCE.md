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

- `src/pdf.rs` from `casual-calc-render/src/pdf.rs` at `b23c67c` (`VS-04`): the object
  store and cross-reference table, `num`, the Type0 / Identity-H fonts with `/W` and
  `ToUnicode` (`write_faces`, `to_unicode_cmap`), `/ActualText` for a run whose glyphs
  do not pair one to one with its characters, and `TJ` adjustments from the rounded `/W`
  widths. Changes: the page executes the slide display list (paths with transforms,
  axial and radial shadings, pictures as Flate-compressed image XObjects with a soft
  mask, transparency through `ExtGState`), rather than cell items; a run's glyphs come
  positioned from `-text`, so nothing is shaped here; faces are found by the run's key;
  font programs are Flate-compressed; the title is UTF-16.
- `src/subset.rs` from `casual-calc-render/src/subset.rs` at `b23c67c`, unchanged but for
  checked reads in the composite reader and the `head` patch.

Not copied: glyph outlining and shaping (`shape.rs`), which live in the text crate.

# 21 — Parser limits

**The contract.** Every axis on which a hostile deck could make us spend memory or time
has a ceiling, the ceiling is enforced before the cost is paid, and crossing it returns a
typed error with a stable code from [20](20-ERROR-CODE-REGISTRY.md), or a finding, never
a panic. Limits are **required behaviour, not best effort**, and a host may tighten them
but none can be switched off. The number matches OpenCalc's docs/21, which this follows,
and the package limits were forked from OpenCalc's with the code (`F-002`).

**This page is read from the code, not the other way round.** Each default below is
quoted as the code writes it, and `tools/check-parser-limits.py` fails if a limit field
or constant exists in the code and not here, or if the quoted default differs. A row
here that the code does not keep is a defect, and is a tracker row, not a doc edit.

Status, as in [07](07-SECURITY-THREAT-MODEL.md): **Tested** means a test or hostile
fixture exercises the ceiling. **Untested** means the code enforces it and nothing has
been seen to fail without it, so it may not be relied on (`SEC-016`).

## Layers

A deck passes through these in order. Each layer charges its own budget and inherits
the one beneath it.

1. **Admission**, `casual-slide-package`: the ZIP central directory, judged before any
   part is decompressed.
2. **Reading a part**, `casual-slide-package`: decompression held to the declared size
   and to the whole-package budget.
3. **XML**, `casual-slide-ooxml`: one bounded walker and one tree, which every XML part
   goes through. DOCTYPE is refused outright (`CS-XML-0005`), so entity expansion never
   arises.
4. **Graph**, `casual-slide-ooxml`: relationships and content types.
5. **Import**, `casual-slide-import`: counts the model would allocate for.
6. **Values**: per-construct caps in the model, geometry and export.

## Admission: `PackageLimits`

`crates/casual-slide-package/src/limits.rs`. Checked when a package is opened, against
the central directory, before anything is inflated.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_input_bytes` | `1 << 30` | The compressed file, 1 GiB | CS-PKG-0001 | Tested |
| `max_entries` | `50_000` | Entries in the archive | CS-PKG-0002 | Tested |
| `max_total_uncompressed` | `4u64 << 30` | Declared uncompressed bytes over all entries, 4 GiB; also the budget every read is charged against | CS-PKG-0003 | Tested |
| `max_expansion_ratio` | `1000` | Uncompressed to compressed, over the archive **and per entry**, since ordinary parts can dilute a bomb (`SEC-013`) | CS-PKG-0003 | Tested; `hostile/zip-bomb.bin` |
| `max_path_bytes` | `4096` | Bytes in one entry name | CS-PKG-0004 | Fixture `path-too-long`, seen red with the cap removed (`SEC-020`) |

## Reading a part

`crates/casual-slide-package/src/package.rs`.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| Declared size | as declared | A part may not produce more bytes than its entry declares (`SEC-P02`) | CS-PKG-0003 | Partial: removing our bound leaves tests green, because the `zip` crate also enforces it (`SEC-017`) |
| Whole-package budget | `max_total_uncompressed` | Every read is charged; once spent, reads refuse. **Cumulative over the package's life**, so a long editing session can exhaust it on a legitimate deck (`PKG-01`) | CS-PKG-0003 | Tested |
| `MAX_UP_FRONT` | `1 << 20` | Capacity allocated before decompressing, whatever the entry declares, so a forged size cannot abort a WebAssembly instance (`SEC-P04`). Nothing proves `read_part` uses it (`SEC-017`) | none: allocation only | Partial |
| `read_part_capped` | caller's `max` | A caller-chosen ceiling for one part, refused on the declaration with nothing inflated. `PresentationPackage::read_part` does not use it, so optional parts are bounded only by the budget (`PKG-04`) | CS-PKG-0009 | Tested |

## XML and graph: `OoxmlLimits`

`crates/casual-slide-ooxml/src/limits.rs`. Includes `PackageLimits` as `package`.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_xml_part_bytes` | `256 << 20` | One XML part handed to the walker or the tree, refused on its declared size before inflating (`SEC-P13`) | CS-XML-0006 | **Untested** (`SEC-016`) |
| `max_xml_elements` | `20_000_000` | Elements in one part | CS-XML-0001 | **Untested** (`SEC-016`) |
| `max_xml_depth` | `256` | Nesting depth in one part | CS-XML-0002 | Tested; `hostile/deep-nesting.bin` |
| `max_xml_attributes` | `1024` | Attributes on one element | CS-XML-0003 | **Untested** (`SEC-016`) |
| `max_part_relationships` | `200_000` | Relationships declared by one part | CS-OPC-0002 | **Untested** (`SEC-016`) |
| `max_total_relationships` | `2_000_000` | Relationships across the package | CS-OPC-0002 | **Untested** (`SEC-016`) |
| `max_content_types` | `200_000` | Entries in `[Content_Types].xml` | CS-OPC-0003 | **Untested** (`SEC-016`) |
| `MAX_ENCODING_LAYERS` | `8` | Rounds of percent-decoding a part name or target may need; more is refused (`OPC-01`) | CS-FND-0011 as a finding | Tested |
| `MAX_REPORTED_ID_BYTES` | `64` | Bytes of a relationship id a finding will repeat, with its form checked as an NCName; longer or otherwise is left out, and still works as an id (`DOC-009`) | none: left out of the finding | Tested |

The walker accepts some malformed attributes (`XML-02`) and some non-XML (`XML-03`). Those
are correctness defects with security consequences, not missing limits, and are tracked there.

## Import: `ImportLimits`

`crates/casual-slide-import/src/properties.rs`. Over a limit, the part is refused and the
deck does not import.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_slides` | `100_000` | Slide ids in `sldIdLst`, and outline entries in `viewProps` | CS-IMP-0001 | Tested; `hostile/import-limit-slides.bin` |
| `max_masters` | `10_000` | Entries in each master id list | CS-IMP-0001 | Tested; `import-limit-masters` |
| `max_custom_shows` | `10_000` | Custom shows | CS-IMP-0001 | Tested; `import-limit-custom-shows` |
| `max_custom_show_slides` | `500_000` | Slides across all of a part's custom shows | CS-IMP-0001 | Tested; `import-limit-show-slides` |
| `max_embedded_fonts` | `4_096` | Embedded fonts | CS-IMP-0001 | Tested; `import-limit-fonts` |
| `max_customer_data` | `100_000` | `custData` entries | CS-IMP-0001 | Tested; `import-limit-customer-data` |
| `max_guides` | `65_536` | Guides in one list | CS-IMP-0001 | Tested; `import-limit-guides` |
| `max_custom_properties` | `100_000` | Custom properties | CS-IMP-0001 | Tested; `import-limit-custom-properties` |
| `max_keywords` | `100_000` | `value` children of `keywords` | CS-IMP-0001 | Tested; `import-limit-keywords` |
| `max_variant_items` | `1_000_000` | Vector and array items across a part; a `vt:vector/@size` is never believed | CS-IMP-0001 | Tested; `import-limit-variant-items` |
| `max_variant_depth` | `16` | Nesting of `vt:variant` and `vt:vector` | CS-IMP-0001 | Tested; `import-limit-variant-depth` |
| `max_value_bytes` | `16 << 20` | Bytes in one attribute or text value | CS-IMP-0002 | Tested; `import-limit-value-bytes` |
| `max_findings` | `10_000` | Distinct findings kept; one more says the list was cut | none: report capped | Tested with a tightened bound on `import-bad-values`. Made-up names are one finding since `SEC-018`, so `import-findings-flood` no longer nears it |

Colour and theme import is the exception: it never refuses a part. What it cannot believe
is left absent, reported, and kept as bytes.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_color_transforms` | `64` | Transforms read on one colour; over it, the colour is absent. Applied by `import_theme_limited` and `Findings::with_limits` | CS-CLR-0002 | Tested; fixture `color-too-many-transforms`, seen red with the cap ignored (`COLOR-01`) |
| `max_theme_list_entries` | `4096` | Entries read from one theme list; the rest is reported and kept in the part | CS-CLR-0010 | Tested; fixture `color-list-capped`, seen red |
| `max_color_findings` | `1024` | Colour findings kept for one part; the count stays exact | none: report capped | Tested; fixture `color-findings-flood`, seen red |
| `MAX_LIST_ENTRIES` | `4096` | Entries read from one theme list: custom colours, supplemental fonts, style matrix entries, gradient stops, extensions. The rest is retained, not modelled | CS-CLR-0010 | Tested |
| `MAX_FINDINGS` | `1024` | Colour findings kept for one part | none: report capped | Tested |
| `MAX_COLOR_TRANSFORMS` | `64` | Transforms on one colour, in `casual-slide-model`. Over it, the colour is absent | CS-CLR-0002 | Tested |

These are constants, not fields of `ImportLimits`, so a host cannot tighten them yet
(`COLOR-01`).

## Slides: `SlideLimits`

`crates/casual-slide-import/src/slide.rs`. Over a limit, the deck is refused with a `CS-SLD` code.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_shapes_per_part` | `100_000` | Shape-tree nodes in one part | CS-SLD | Tested; hostile fixture |
| `max_group_depth` | `64` | Group nesting | CS-SLD | Tested; hostile fixture |
| `max_slides` | `100_000` | Slides indexed | CS-SLD-0003 | Tested; fixture `import-limit-structure-slides`, seen red. `ImportLimits::max_slides` holds the same default and refuses first in `import_deck`; a host raising that one reaches this |
| `max_parts` | `250_000` | Slide parts of all kinds | CS-SLD-0004 | Fixture `slide-limit-parts`, under a tightened bound. **Not reachable at the default**: `max_entries` refuses a package first, which a test holds |
| `max_layout_ids` | `10_000` | Entries in a master's layout list | CS-SLD-0004 | Tested; fixture `slide-limit-layout-ids`, seen red |

## Drawing properties: `DrawingLimits`

`crates/casual-slide-import/src/drawing.rs`. Geometry, fills, lines, effects and 3D on one shape. Over a limit, the **whole property** is left out of the model and reported as `CS-DRW-0001`; the tree keeps it, so the deck opens and saves as it was. A part of a gradient would be a wrong gradient, which is why the list is not cut. The caps from `casual-slide-geometry` below apply again, on compile.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_gradient_stops` | `1_024` | `gs` in one `gsLst` | CS-DRW-0001 | Tested; hostile fixture |
| `max_dash_stops` | `1_024` | `ds` in one `custDash` | CS-DRW-0001 | Tested; hostile fixture |
| `max_effects` | `1_024` | Effects in one list, blip or container | CS-DRW-0001 | Tested; hostile fixture |
| `max_effect_depth` | `32` | Nesting of containers, and of fills in effects in blips, which the schema does not bound | CS-DRW-0001 | Tested; two hostile fixtures |
| `max_picture_effects` | `4_096` | `imgEffect` in one layer, marks in one `backgroundRemoval` | CS-DRW-0001 | Tested; hostile fixture |
| `max_guides` | `4_096` | `gd` in one `avLst` or `gdLst` | CS-DRW-0001 | Tested; hostile fixture |
| `max_handles` | `256` | Children of one `ahLst` | CS-DRW-0001 | Tested; hostile fixture |
| `max_sites` | `4_096` | `cxn` in one `cxnLst` | CS-DRW-0001 | Tested; hostile fixture |
| `max_paths` | `1_024` | `path` in one `pathLst` | CS-DRW-0001 | Tested; hostile fixture |
| `max_path_commands` | `262_144` | Commands in one path | CS-DRW-0001 | Tested; hostile fixture |
| `max_items_per_property` | `1_000_000` | Elements read for one property, whatever its shape | CS-DRW-0001 | Tested by tightening it through `SlideLimits`; no hostile fixture, as `max_elements` reaches first by default |

`SlideLimits::drawing` carries these to every reader on a slide-like part: a shape's properties, a picture's fill, and a text body's warp, 3D, run fills, lines and effects (`slide_limits_reach_every_drawing_reader`). A presentation's `defaultTextStyle` and a theme's `objectDefaults` are read outside `SlideLimits`, with `DrawingLimits::default()`. `read_fill`, `read_line`, `read_effects`, `read_blip_fill` and `read_preset_text_shape` take limits, which is how `one_under_each_limit_is_read` tightens one.

Hostile fixtures added for `CLOSE-01`: `text-limit-warp-guides`, `text-limit-3d-depth`, `text-bad-3d`, `drawing-limit-picture-effects`, `shape-style-bad-values`, `theme-object-defaults-bad-values`.

## Geometry

`crates/casual-slide-geometry/src/lib.rs`, module `limits`, enforced by `RawGeometry::compile`.
The largest preset has 244 guides; real freeforms have many more points, which is why that
cap is the generous one.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_GUIDES` | `4096` | `av` plus `gd` guides | CS-GEO-0001 | Tested; `tests/hostile.rs`, fixture `geometry-too-many-guides` |
| `MAX_PATHS` | `1024` | Paths | CS-GEO-0002 | Tested; fixture `geometry-too-many-paths` |
| `MAX_PATH_POINTS` | `262_144` | Points over all paths | CS-GEO-0003 | Tested; fixture `geometry-too-many-points` |
| `MAX_HANDLES` | `256` | Adjust handles | CS-GEO-0004 | Tested; fixture `geometry-too-many-handles` |
| `MAX_SITES` | `4096` | Connection sites | CS-GEO-0005 | Tested; fixture `geometry-too-many-sites` |
| `MAX_FORMULA_LEN` | `1024` | Bytes in one formula | CS-GEF-0004, a repair | Tested; fixture `geometry-long-names` |
| `MAX_NAME_LEN` | `256` | Bytes in one guide name | CS-GEF-0004, a repair | Tested; fixture `geometry-long-names` |
| `MAX_EXTENT` | `27_273_042_316_900` | Shape extent in EMU, `ST_PositiveCoordinate`'s maximum; larger values are clamped, not refused, in `eval.rs` | none: clamped | Untested by a hostile extent; a compile-time assertion holds it under the arithmetic's limit |
| `MAX_KEPT` | `32` | Diagnostics kept per compile or evaluation; the count stays exact | none: report capped | Tested |

A geometry diagnostic holds no text from the file (`SEC-019`), and every refusal and repair has a code (`GEO-04`).

## Text: `TextLimits`

`crates/casual-slide-import/src/text.rs`, nested in `ImportLimits` as `text`. Each is well beyond any real deck; the largest committed real deck is well under one percent of each. The part bounds below `max_runs_per_part` set what a text-heavy part can cost in the model: a run holds at most about 400 bytes of model before its strings, and each string is bounded by `max_value_bytes` and by the part. A size test holds that figure, so at the defaults text in one part stays near 400 MiB plus its characters. A run's fill, line and effects are boxed and sized by what the file holds, each under `DrawingLimits`, as on a shape.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_paragraphs_per_body` | `100_000` | Paragraphs in one text body | CS-TXT-0001 | Tested; `hostile/text-limit-paragraphs-in-body.bin` |
| `max_runs_per_paragraph` | `100_000` | Runs, breaks, fields and retained children in one paragraph | CS-TXT-0002 | Tested; `hostile/text-limit-runs-in-paragraph.bin` |
| `max_paragraphs_per_part` | `250_000` | Paragraphs across one part's shapes | CS-TXT-0003 | Tested; `hostile/text-limit-paragraphs-in-part.bin` |
| `max_runs_per_part` | `1_000_000` | Runs, breaks, fields and retained children across one part | CS-TXT-0004 | Tested, with the bound lowered: a deck crossing the default is too large to commit |
| `max_tab_stops` | `1_024` | Tab stops in one `tabLst`. Between the schema's 32 and this, they are kept and reported as `CS-IMF-0002` | CS-TXT-0005 | Tested; `hostile/text-limit-tab-stops.bin` |

Text bodies follow no input to any depth: `txBody`, `p`, `r` and `rPr` have a fixed shape, and a colour's transforms have their own bound. `a:t` is bounded by `max_value_bytes`.

## Tables: `TableLimits`

`crates/casual-slide-import/src/table.rs`, nested in `SlideLimits` as `table`. Each is at or beyond what Office accepts (`MS-OI29500` 2.1.1424 a and b: 1000 rows, 1000 columns), so no table Office would open is refused by one of these, though `max_paragraphs_per_part` can refuse the text of one that large. Between Office's 1000 and these, a table is kept and reported as `CS-TBF-0006`. No count or span from a file sizes anything: a span is compared with what is left of the table at that cell, and `Table::merges` is one pass over the cells that exist, with one slot per grid column. A size test holds a cell at no more than 96 bytes of model before its text and properties, which are boxed and bounded by `TextLimits` and `DrawingLimits`, so cells in one part stay near 200 MiB at the default.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_rows` | `4_096` | `tr` in one table | CS-TBL-0001 | Tested; `hostile/table-limit-rows.bin` |
| `max_columns` | `4_096` | `gridCol` in one grid | CS-TBL-0002 | Tested; `hostile/table-limit-columns.bin` |
| `max_cells_per_table` | `1_048_576` | `tc` in one table | CS-TBL-0003 | Tested; `hostile/table-limit-cells-in-table.bin` |
| `max_cells_per_part` | `2_097_152` | `tc` across one part | CS-TBL-0004 | Tested, with the bound lowered: a deck crossing the default is too large to commit |
| `max_table_styles` | `4_096` | `tblStyle` in the list | CS-TBL-0005 | Tested; `hostile/table-limit-styles.bin` |
| `max_headers_per_cell` | `1_024` | `header` in one cell's `headers` | CS-TBL-0006 | Tested; `hostile/table-limit-headers.bin` |

A table has a fixed depth, and a style's parts follow no input. What is damaged but not over a bound opens and is reported: `hostile/table-bad-structure.bin`.

## Charts: `ChartLimits` (Tier 1J)

`crates/casual-slide-import/src/chart.rs`, nested in `SlideLimits` as `chart`. Over a bound, that one part is set aside, reported under the bound's code and `CS-IMF-0012`, and kept byte for byte; **the deck opens** (owner instruction 24). Each is beyond what Office's interface makes. An embedded workbook is never read, so it spends nothing here and has no limit here (`SEC-P09`). No count in the file sizes anything: `ptCount` is compared, never allocated by.

| Field | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_chart_parts` | `8_192` | Chart-family parts read in one deck | CS-CHT-0001 | Tested, with the bound lowered: a deck crossing the default is too large to commit |
| `max_series` | `4_096` | Series in one part | CS-CHT-0002 | Tested; `hostile/chart-limit-series.bin` |
| `max_points` | `1_048_576` | Cached points in one part | CS-CHT-0003 | Tested; `hostile/chart-limit-points.bin` |
| `max_data_points` | `262_144` | `dPt` and `cx:dataPoint` in one part | CS-CHT-0004 | Tested; `hostile/chart-limit-data-points.bin` |
| `max_data_labels` | `262_144` | `dLbl` and `cx:dataLabel` in one part | CS-CHT-0005 | Tested; `hostile/chart-limit-data-labels.bin` |
| `max_levels` | `4_096` | `lvl` in one part | CS-CHT-0006 | Tested; `hostile/chart-limit-levels.bin` |
| `max_anchors` | `65_536` | Anchors in one chart drawing | CS-CHT-0007 | Tested; `hostile/chart-limit-anchors.bin` |
| `max_extensions` | `262_144` | `ext` in one part | CS-CHT-0008 | Tested; `hostile/chart-limit-extensions.bin` |
| `max_group_depth` | `64` | Groups nested in one chart drawing, the one recursion in these schemas | CS-CHT-0009 | Tested; `hostile/chart-limit-group-depth.bin` |

What is damaged but not over a bound is read as far as it goes and reported: `hostile/chart-bad-structure.bin`. What is rooted wrongly is set aside: `hostile/chart-wrong-root.bin`; what is not XML likewise, built in `crates/casual-slide-export/tests/chart.rs`, as every `open` deck in the corpus is held to parse part by part. A workbook that could not be read if anyone tried is untouched: `hostile/chart-workbook-corrupt.bin`.

## Export

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_WRITE_DEPTH` | `16` | Nesting of a variant written back, in `casual-slide-export/src/properties.rs`; the model can hold what import refused only if built by hand | CS-EXP-0008 | Tested |
| `max_plan_ops` | `4_096` | `WriterLimits`: operations in one save plan, in `casual-slide-export/src/writer/plan.rs` | CS-SAV-0018 | Tested |
| `max_clone_parts` | `10_000` | `WriterLimits`: parts one deep clone may copy, since a hostile deck can hang any number off one slide | CS-SAV-0018 | Tested |
| `max_clone_bytes` | `1 << 30` | `WriterLimits`: bytes one deep clone may copy, on top of the package budget | CS-SAV-0018 | Tested |
| `max_media_bytes` | `1 << 30` | `WriterLimits`: bytes of one new media part | CS-SAV-0018 | Tested |

## Not yet bounded

Each of these is a known gap with an owner, not an oversight:

- **Time.** No limit bounds the time a deck can take within its byte budgets (`SEC-P10`, Tier 1A).
- **Embedded packages.** An embedded deck or workbook is retained, never opened; opening one
  will be a separate admission with an inherited budget (`SEC-P09`).
- **Layout and rendering** limits arrive with Tier 2D: glyphs per frame, image pixel dimensions
  before decode.
- **Legacy `.ppt`** has no reader; a Compound File is refused as `CS-OPC-0005` (`CFB-01`, `PPT-01`).

## Tools

`tools/f4/validate.py` and `tools/ooxml-sdk/` read the same untrusted decks and are bounded
the same way or tighter: at most 100,000 entries, 256 MiB per part, an expansion ratio of
1000, 5,000,000 elements and depth 256 per part, and DOCTYPE refused. Neither runs on
`fixtures/hostile/`.

## Changing a limit

A change to a default here is a change to a protected area ([GOVERNANCE](../GOVERNANCE.md)).
It needs the reason, a hostile fixture at the new ceiling seen to fail one past it, the row in
this page, and the code in [20](20-ERROR-CODE-REGISTRY.md) and [07](07-SECURITY-THREAT-MODEL.md).

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
been seen to fail without it, so it may not be relied on (`SEC-016`). **Partial** means a fixture or test exercises it, but the guard has not been seen to fail with it removed.

## Layers

A deck passes through these in order. Each layer charges its own budget and inherits
the one beneath it.

1. **Admission**, `casual-slide-package`: the ZIP central directory, judged before any
   part is decompressed.
2. **Reading a part**, `casual-slide-package`: decompression held to the declared size
   and to the whole-package budget.
3. **XML**, `casual-slide-ooxml`: one bounded walker and one tree, which every XML part
   goes through. Under `strict`, DOCTYPE is refused outright (`CS-XML-0005`). By default the
   recovering reader skips it uninterpreted, so entity expansion never arises either way.
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
| `read_part_capped` | caller's `max` | A caller-chosen ceiling for one part, refused on the declaration with nothing inflated. `PresentationPackage::read_part_capped` exposes it for binary parts. `PresentationPackage::read_part` does not use it, so optional parts are bounded only by the budget (`PKG-04`) | CS-PKG-0009 | Tested |

## The recovering reader

`crates/casual-slide-package/src/salvage.rs` (`REC-01`), which reads what `Package::open` refuses. It holds to `PackageLimits`: `max_input_bytes` still refuses (the one limit that does, since it protects the machine); `max_entries` stops the scan and says so (`CS-REC-0109`); `max_total_uncompressed` is one budget for everything inflated, however many entries point at one stream; `max_expansion_ratio` and `max_path_bytes` set an entry aside as an opaque blob (`CS-REC-0107`, `CS-REC-0106`). Nothing allocates from a number in the file, and a scan only moves forward.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_SCRUBBED_NAME` | `200` | Characters of a damaged entry's name repeated in a report or a retained name | none: text only | Tested by `an_unsafe_name_never_becomes_a_part` |
| `MAX_CHOOSING_BYTES` | `256 << 20` | The most of one entry inflated into memory to choose between entries naming one part | none: that entry is not chosen on its content | Untested |
| `MAX_REFERENCE` | `32` | How far the recovering XML reader looks for a reference's `;`. Past it, the `&` is literal text | none: text only | Tested by `broken_nesting_is_closed` |

The recovering XML reader, `crates/casual-slide-ooxml/src/recover_xml.rs` (`REC-03`), holds to `OoxmlLimits` as `walk` does, with one difference: `max_xml_elements`, `max_xml_depth` and `max_xml_attributes` stop the reading of that part and keep what was read (`CS-REC-0307`), and a reading over `max_xml_part_bytes` is given up, so the part is set aside. `max_part_relationships`, `max_total_relationships` and `max_content_types` keep what fits.

## XML and graph: `OoxmlLimits`

`crates/casual-slide-ooxml/src/limits.rs`. Includes `PackageLimits` as `package`.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_xml_part_bytes` | `256 << 20` | One XML part handed to the walker or the tree, refused on its declared size before inflating (`SEC-P13`) | CS-XML-0006 | Tested; fixture `xml-part-over-size`, seen red with the cap removed (`SEC-016`) |
| `max_xml_elements` | `20_000_000` | Elements in one part | CS-XML-0001 | Tested; fixture `xml-too-many-elements`, seen red with the cap removed (`SEC-016`) |
| `max_xml_depth` | `256` | Nesting depth in one part | CS-XML-0002 | Tested; `hostile/deep-nesting.bin` |
| `max_xml_attributes` | `1024` | Attributes on one element | CS-XML-0003 | Tested; fixture `xml-too-many-attributes`, seen red with the cap removed (`SEC-016`) |
| `max_part_relationships` | `200_000` | Relationships declared by one part | CS-OPC-0002 | Tested; fixture `rels-per-part`, seen red with the cap removed (`SEC-016`) |
| `max_total_relationships` | `2_000_000` | Relationships across the package | CS-OPC-0002 | Tested; fixture `rels-total`, seen red with the cap removed (`SEC-016`) |
| `max_content_types` | `200_000` | Entries in `[Content_Types].xml` | CS-OPC-0003 | Tested; fixture `content-types-flood`, seen red with the cap removed (`SEC-016`) |
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
## Media: `MediaLimits`

`crates/casual-slide-import/src/media.rs`. Media is read only by `catalog_media`, on request, and never by opening a deck. Its bytes go to a hash and a header sniff, and are dropped: nothing is decoded, and an SVG is not even sniffed. Each default is Inferred, far above any deck here.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_bookmarks` | `4_096` | `bmk` in one `bmkLst`. Over it, the whole `p14:media` is left out, reported, and kept in the tree | CS-MED-0002 | Tested; `hostile/media-limit-bookmarks.bin`, at the bound and one past |
| `max_tracks` | `1_024` | `track` in one `trackLst`, the same way | CS-MED-0002 | Tested; `hostile/media-limit-tracks.bin`, at the bound and one past |
| `max_part_bytes` | `256 << 20` | Bytes one part may declare and still be read for a catalog, refused from the declaration with nothing inflated | CS-MED-0003 | Tested, with the bound lowered: `a_catalog_stops_at_its_bounds_and_says_so`, `a_binary_part_over_the_ceiling_is_refused_with_nothing_inflated` |
| `max_catalog_parts` | `100_000` | Parts one catalog lists | CS-MED-0008 | Tested, with the bound lowered: the package's `max_entries` is under it by default |
| `max_catalog_bytes` | `2 << 30` | Bytes one catalog reads, half the package's budget, which is cumulative (`PKG-01`). Checked between parts, so one part can overshoot by its size, itself capped below | CS-MED-0003 | Tested, with the bound lowered |
| `max_image_pixels` | `1 << 28` | Pixels an image may declare before it is reported. Reported and not refused: nothing here decodes; the renderer in Tier 2E must check it first (`SEC-C01`) | CS-MED-0004 | Tested; `hostile/media-image-bomb.bin`, at the bound and one past |
| `MAX_HEADER_SCAN` | `1 << 20` | Bytes looked through for a JPEG's size. Past it, the size stays unknown | none: left unknown | Tested in part: `hostile/media-lying-images.bin` covers cut and looping headers. Nothing crosses the bound itself |

A part in a catalog is read by `PresentationPackage::read_part_capped`, which refuses as `CS-PKG-0009` from the declared size; the catalog reports that as CS-MED-0003.
## Timing: `TimingLimits`

`crates/casual-slide-import/src/timing.rs`, nested in `SlideLimits` as `timing`. Nothing here refuses a deck or a part: crossing a bound leaves the whole `p:timing` or `p:transition` unread and in the tree, reported as `CS-TMG-0001` naming what crossed it (owner instruction 24), and an edit may replace it. The real decks committed hold 6 levels of time nodes at most, and 65 `cTn` across a whole deck (`T1I-01`).

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_node_depth` | `32` | Time nodes nested in time nodes, `mc:AlternateContent` counting as one. Three XML levels each, so it is met well before `max_xml_depth`. The writer holds a tree built by hand to it, as `CS-EXP-0008` | CS-TMG-0001 | Tested; `hostile/timing-limit-depth.bin` |
| `max_nodes` | `100_000` | Time nodes in one part's timing, the build list's templates included | CS-TMG-0001 | Tested; `hostile/timing-limit-nodes.bin` |
| `max_conditions` | `4_096` | Conditions in one list | CS-TMG-0001 | Tested; `hostile/timing-limit-conditions.bin` |
| `max_attr_names` | `1_024` | `attrName` in one `attrNameLst` | CS-TMG-0001 | Tested; `hostile/timing-limit-attr-names.bin` |
| `max_animate_values` | `65_536` | `tav` in one `tavLst` | CS-TMG-0001 | Tested; `hostile/timing-limit-animate-values.bin` |
| `max_builds` | `65_536` | Entries in one `bldLst` | CS-TMG-0001 | Tested; `hostile/timing-limit-builds.bin` |
| `max_templates` | `64` | `tmpl` in one `tmplLst`. Between the schema's 9 and this, they are kept | CS-TMG-0001 | Tested; `hostile/timing-limit-templates.bin` |
| `max_branches` | `64` | Branches in one `mc:AlternateContent` holding timing or a transition | CS-TMG-0001 | Tested; `hostile/timing-limit-branches.bin` |

Each fixture is seen to read with its bound raised by one (`raising_each_limit_by_one_reads_what_it_refused`). `hostile/timing-long-value.bin` is `CS-TMG-0002` under a lowered `max_value_bytes`. Past `max_xml_depth` a part fails in the XML reader before this one sees it, shown with that limit lowered: that slide is left out and reported as `CS-IMF-0012`, the deck opens, and `REC-03` and `REC-04` will read more of it. Nothing here evaluates a path or a formula, runs a command, or plays anything. Infinite repeats and a timeline's time budget are Phase 8's (`SEC-C10`).

## Charts: `ChartLimits` (Tier 1J)

`crates/casual-slide-import/src/chart.rs`, nested in `SlideLimits` as `chart`. Over a bound, that one part is set aside, reported under the bound's code and `CS-IMF-0012`, and kept byte for byte; **the deck opens** (owner instruction 24). Each is beyond what Office's interface makes. An embedded workbook is never read, so it spends nothing here and has no limit here (`SEC-P09`). No count in the file sizes anything: `ptCount` is compared, never allocated by.

| Field | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_chart_parts` | `8_192` | Chart-family parts read in one deck | CS-CHT-0001 | Tested: with the bound lowered: a deck crossing the default is too large to commit; guard removed, `more_chart_parts_than_the_bound_are_set_aside_and_the_rest_read` red (`SEC-021`, 2026-11-08) |
| `max_series` | `4_096` | Series in one part | CS-CHT-0002 | Tested: `hostile/chart-limit-series.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_points` | `1_048_576` | Cached points in one part | CS-CHT-0003 | Tested: `hostile/chart-limit-points.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_data_points` | `262_144` | `dPt` and `cx:dataPoint` in one part | CS-CHT-0004 | Tested: `hostile/chart-limit-data-points.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_data_labels` | `262_144` | `dLbl` and `cx:dataLabel` in one part | CS-CHT-0005 | Tested: `hostile/chart-limit-data-labels.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_levels` | `4_096` | `lvl` in one part | CS-CHT-0006 | Tested: `hostile/chart-limit-levels.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_anchors` | `65_536` | Anchors in one chart drawing | CS-CHT-0007 | Tested: `hostile/chart-limit-anchors.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_extensions` | `262_144` | `ext` in one part | CS-CHT-0008 | Tested: `hostile/chart-limit-extensions.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |
| `max_group_depth` | `64` | Groups nested in one chart drawing, the one recursion in these schemas | CS-CHT-0009 | Tested: `hostile/chart-limit-group-depth.bin`; guard removed, `a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens` red (`SEC-021`, 2026-11-08) |

What is damaged but not over a bound is read as far as it goes and reported: `hostile/chart-bad-structure.bin`. What is rooted wrongly is set aside: `hostile/chart-wrong-root.bin`; what is not XML likewise, built in `crates/casual-slide-export/tests/chart.rs`, as every `open` deck in the corpus is held to parse part by part. A workbook that could not be read if anyone tried is untouched: `hostile/chart-workbook-corrupt.bin`.

## Diagrams, equations and ink: `GraphicLimits`

`crates/casual-slide-import/src/diagram.rs`, nested in `ImportLimits` as `graphic`. Over a bound, a part of a diagram or an ink part is set aside and reported, and the deck opens; an equation in a slide's text stays in the tree and is reported (`CS-GFF-0008`).

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `graphic` | `crate::diagram::GraphicLimits::default()` | The bounds below, in `ImportLimits` | See below | Tested: each bound below removed in turn and its test seen to fail (`SEC-021`) |
| `max_depth` | `128` | Elements nested below a part's root | CS-GFX-0001 | Tested: `hostile/graphics-limit-depth.bin`, `hostile/graphics-limit-math-depth.bin`; guard removed, `a_part_over_a_bound_is_set_aside_with_its_code` and `an_equation_over_a_bound_stays_in_the_tree_and_is_reported` failed (`SEC-021`) |
| `max_elements` | `4_194_304` | Elements read in one part, or across a slide's equations | CS-GFX-0002 | Tested: with the bound lowered, as a deck crossing the default is too large to commit; guard removed, `the_element_bound_and_each_bound_at_its_value` failed (`SEC-021`) |
| `max_points` | `65_536` | `dgm:pt` in one part, `SEC-C12` | CS-GFX-0003 | Tested: `hostile/graphics-limit-points.bin`; guard removed, `a_part_over_a_bound_is_set_aside_with_its_code` failed (`SEC-021`) |
| `max_connections` | `131_072` | `dgm:cxn` in one part | CS-GFX-0004 | Tested: `hostile/graphics-limit-connections.bin`; guard removed, `a_part_over_a_bound_is_set_aside_with_its_code` failed (`SEC-021`) |
| `max_layout_nodes` | `65_536` | `dgm:layoutNode` in one part | CS-GFX-0005 | Tested: `hostile/graphics-limit-layout-nodes.bin`; guard removed, `a_part_over_a_bound_is_set_aside_with_its_code` failed (`SEC-021`) |
| `max_drawing_shapes` | `65_536` | `dsp:sp` in one part | CS-GFX-0006 | Tested: `hostile/graphics-limit-drawing-shapes.bin`; guard removed, `a_part_over_a_bound_is_set_aside_with_its_code` failed (`SEC-021`) |
| `max_traces` | `262_144` | `inkml:trace` in one part | CS-GFX-0007 | Tested: `hostile/graphics-limit-traces.bin`; guard removed, `a_damaged_ink_part_is_set_aside_and_the_slide_stands` failed (`SEC-021`) |

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

- **Time.** The package asks a host-supplied predicate before every read of a decompressing part, and stops with `CS-PKG-0010` (`SEC-P10`, `Package::set_cancel`). Import does not yet pass one through, and parsing after decompression has no time bound of its own.
- **Embedded packages.** An embedded deck or workbook is retained, never opened; opening one
  will be a separate admission with an inherited budget (`SEC-P09`).
- **Layout** limits arrive with Tier 2D: glyphs per frame. Rendering's are below.
- **Legacy `.ppt`** has no reader; a Compound File is refused as `CS-OPC-0005` (`CFB-01`, `PPT-01`).

## Rendering (`casual-slide-render`, `SKEL-01`)

Past one of these the picture, shape or slide is not drawn, and the render report names it; nothing stops the deck. The picture bounds follow OpenCalc's `casual-calc-render`, checked against the header before any allocation.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_IMAGE_PIXELS` | `40_000_000` | Pixels a picture may declare before it is decoded | Report: image over the pixel limit | Partial: `a_png_declaring_too_many_pixels_is_refused_before_decoding` |
| `MAX_EMF_RECORDS` | `100_000` | Records read from an EMF looking for its one bitmap (`T2E-03`, `FH-05`); the bitmap itself is held to `MAX_IMAGE_PIXELS` | Report: EMF not decoded | Partial: `truncated_and_hostile_metafiles_never_panic` |
| `MAX_IMAGE_SIDE` | `20_000` | Pixels on one side of a JPEG, given to the decoder as its limit | Report: image over the pixel limit | Untested |
| `MAX_MEDIA_BYTES` | `64 * 1024 * 1024` | Bytes of one media part read for drawing | Report: media not supplied | Untested |
| `MAX_SIDE_PX` | `16_384` | Pixels on one side of a rendered slide | CS-RND-0001 | Untested |
| `MAX_GROUP_DEPTH` | `64` | Groups walked inside one another, beyond import's own bound | Report: group nested too deep | Untested |
| `MAX_SEGMENTS_PER_SHAPE` | `200_000` | Path segments drawn for one shape | Report: geometry over the segment budget | Untested |
| `MAX_TABLE_CELLS` | `100_000` | Grid cells of one table drawn | Report: table over the cell budget | Untested |
| `MAX_CHART_POINTS` | `4_096` | Points of one chart series plotted | Points past it are not plotted | Untested |
| `MAX_CHART_SERIES` | `256` | Series of one chart plotted | Series past it are not plotted | Untested |
| `MAX_DIAGRAM_DEPTH` | `32` | Groups nested in a SmartArt drawing converted | Deeper groups are not drawn | Untested |
| `MAX_DIAGRAM_SHAPES` | `10_000` | Shapes of one SmartArt drawing converted | Shapes past it are not drawn | Untested |

The browser bridge (`casual-slide-wasm`, `VS-01`) bounds what a page may ask it to draw. The width comes from the host, the aspect ratio from the deck.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_WIDTH_PX` | `4096` | Pixels wide a host may ask one slide to be drawn | CS-WSM-0002 | Untested: `refuses_a_width_of_zero_or_past_the_bound` runs on CI |
| `MAX_PIXELS` | `4096 * 4096` | Pixels in one drawn slide; past it the width is reduced, not refused | None: the picture is smaller | Untested: `the_pixel_bound_shrinks_the_width_rather_than_refusing` runs on CI |
| `MAX_NOTES_CHARS` | `100_000` | Characters of one slide's speaker notes handed to Presenter View (`VS-07`) | None: the notes are cut and end with an ellipsis | Tested: guard loosened, `notes_past_the_bound_are_cut_and_marked` failed (`tests/present.rs`) |
| `MAX_TRANSITION_MS` | `60_000` | Milliseconds one transition plays, from `p14:dur` (`VS-07`) | None: a longer one plays for a minute | Untested |
| `MAX_SLIDE_SIZE` | `51_206_400` | EMU of a slide side the Slide Size command may set; `MIN_SLIDE_SIZE`, `914_400`, is the least, `ST_SlideSizeCoordinate`'s range (`casual-slide-transaction`, `VS-12`) | CS-TXN-0011 | Tested: guard loosened, `a_slide_size_outside_the_schema_is_refused_and_changes_nothing` failed |
| `MAX_DURATION_MS` | `60_000` | Milliseconds an effect or transition is given by the Transitions and Animations tabs (`VS-10`, `casual-slide-transaction`) | None: a longer one is held to a minute | Untested |
| `MAX_NEW_TABLE_SIDE` | `75` | Rows or columns the writer's `NewShape::Table` writes (`casual-slide-export` writer, `VS-11`) | CS-SAV (writer `BadOperation`) | Untested: no test inserts a table past it |
| `MAX_GRID_SIDE` | `64` | Categories or series a chart's data sheet reads, writes or saves (`casual-slide-export` `new_chart`, `VS-11`); past it the rest is left out | none: truncated | Untested: no test edits a chart past it |
| `MAX_TABLE_SIDE` | `75` | Rows or columns a table edit may grow a table to, PowerPoint's own Insert Table bound (`casual-slide-transaction` `rich`, `VS-11`) | CS-TXN-0013 | Untested: no test grows a table past it |
| `MAX_PICTURE_BYTES` | `64 * 1024 * 1024` | Bytes of one picture Insert > Picture takes, as the writer's media bound (`VS-08`) | CS-WSM-0008 | Untested: no test inserts a picture past it |
| `MAX_NAME_CHARS` | `128` | Characters of the name a host gives a shape it inserts (`SHAPE-01`) | None: the name is cut; control characters are dropped and the rest escaped | Untested |
| `MAX_FRAME_BYTES` | `48 * 1024 * 1024` | Bytes of one collaboration message, sent or accepted; a room's seed deck travels in one, base64 (`casual-slide-transaction::collab`, `VS-09`) | The server closes the connection | Untested |
| `MAX_OUTSTANDING` | `1` | Chunks a collaborating client has in flight: stop and wait (`VS-09`, `COL-02`) | None: edits wait for the acknowledgement | Tested: raised to 32, `two_clients_editing_at_once_through_the_server_end_with_the_same_deck` failed |
| `MAX_PARTICIPANT` | `4095` | Participants a room numbers, each minting node numbers and `cNvPr/@id`s from its own range of 2^20 (`casual-slide-wasm` `collab`, `COL-04`) | CS-COL-0004 | Untested |
| `MAX_WIRE_TEXT` | `1 << 20` | Characters of one inserted text on the wire (`VS-09`) | CS-COL-0004 | Untested |
| `MAX_WIRE_XML` | `1 << 20` | Bytes of one node's XML on the wire, which is then read under the reader's own limits (`VS-09`) | CS-COL-0004 | Untested |
| `MAX_LINE_WIDTH` | `20_116_800` | EMU of an outline weight Shape Outline sets, PowerPoint's 1584 points (`SHAPE-01`) | None: a wider one is set to the bound | Untested |

## Comments, sections, tags and marks (`ReviewLimits`, Tier 1M)

Past one of these the part is set aside and named, or the list left out, and the deck opens (owner instruction 24): the bytes stay, and the report says `CS-RVW-0001`.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `max_comments_per_part` | `100_000` | `cm` in one part | CS-RVW-0001 | Tested: `hostile/comment-limit-comments.bin` (legacy) and a lowered bound (modern); each guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` and `a_modern_comment_or_author_list_past_its_bound_is_set_aside` failed (`SEC-021`) |
| `max_authors` | `100_000` | `cmAuthor` or `author` in one list | CS-RVW-0001 | Tested: `hostile/comment-limit-authors.bin` (legacy) and a lowered bound (modern); each guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` and `a_modern_comment_or_author_list_past_its_bound_is_set_aside` failed (`SEC-021`) |
| `max_replies_per_comment` | `100_000` | `reply` under one comment | CS-RVW-0001 | Tested: `hostile/comment-limit-replies.bin`; guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed (`SEC-021`) |
| `max_reactions_per_holder` | `10_000` | `rxn` and `instance` under one holder | CS-RVW-0001 | Tested: `hostile/comment-limit-reactions.bin` and a lowered bound; each of the three guards (the holder's children, one reaction's children, and `rxn` with `instance` counted together) removed in turn, `reactions_past_their_bound_either_way_refuse_the_part` failed (`SEC-021`) |
| `max_task_events` | `100_000` | `event` in one task | CS-RVW-0001 | Tested: `hostile/comment-limit-task-events.bin`; guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed (`SEC-021`) |
| `max_monikers` | `1_024` | Monikers in one anchor | CS-RVW-0001 | Tested: `hostile/comment-limit-monikers.bin`; guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed (`SEC-021`) |
| `max_review_parts` | `100_000` | Comment parts, tag parts and custom XML items in one deck | CS-RVW-0001 | Tested: at a lowered bound, as 100,001 parts is past `CS-PKG-0002` first; each of the four guards (legacy comment, modern comment, tag and custom XML part counts) removed in turn, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens`, `a_modern_comment_part_past_the_part_count_is_set_aside` or `tag_parts_and_custom_xml_items_past_the_part_count_are_set_aside` failed (`SEC-021`) |
| `max_sections` | `10_000` | `section` in one list | CS-RVW-0001 | Tested: `hostile/section-limit-sections.bin`; guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed (`SEC-021`) |
| `max_section_slides` | `100_000` | `sldId` across one list | CS-RVW-0001 | Tested: `hostile/section-limit-slides.bin`; guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed (`SEC-021`) |
| `max_tags` | `100_000` | `tag` in one part | CS-RVW-0001 | Tested: `hostile/tags-limit-tags.bin`; guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` and `a_part_exactly_at_its_bound_is_read` failed (`SEC-021`) |
| `max_marks_per_part` | `100_000` | Zooms, marks and `custDataLst` entries in one slide-like part. The rest stay in the tree | CS-RVW-0001 | Tested: `hostile/marks-limit-part.bin` and a lowered bound; the element-collecting guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed; the mark walker's guard (`Marks::charge`) removed, `designer_tags_past_the_mark_bound_stay_unread_and_are_reported` failed (`SEC-021`) |
| `max_schema_refs` | `10_000` | `schemaRef` or `schema` in one part | CS-RVW-0001 | Tested: `hostile/custom-limit-schema-refs.bin` and a lowered bound; the `itemProps` guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed; the schema library guard removed, `a_schema_library_past_its_bound_is_refused` failed (`SEC-021`) |
| `max_labels` | `1_024` | Sensitivity labels in one deck | CS-RVW-0001 | Tested: `hostile/custom-limit-labels.bin` and a lowered bound; the `LabelInfo` guard removed, `every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens` failed; the custom-properties guard removed, `labels_in_custom_properties_stop_at_their_bound` failed (`SEC-021`) |

## Tools

`tools/f4/validate.py` and `tools/ooxml-sdk/` read the same untrusted decks and are bounded
the same way or tighter: at most 100,000 entries, 256 MiB per part, an expansion ratio of
1000, 5,000,000 elements and depth 256 per part, and DOCTYPE refused. Neither runs on
`fixtures/hostile/`.

## Changing a limit

A change to a default here is a change to a protected area ([GOVERNANCE](../GOVERNANCE.md)).
It needs the reason, a hostile fixture at the new ceiling seen to fail one past it, the row in
this page, and the code in [20](20-ERROR-CODE-REGISTRY.md) and [07](07-SECURITY-THREAT-MODEL.md).

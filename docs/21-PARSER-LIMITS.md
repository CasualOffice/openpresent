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
| `max_findings` | `10_000` | Distinct findings kept; one more says the list was cut | none: report capped | Tested; `import-findings-flood` |

Colour and theme import is the exception: it never refuses a part. What it cannot believe
is left absent, reported, and kept as bytes.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
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
| `max_slides` | `100_000` | Slides indexed | CS-SLD | Tested by tightening; no hostile fixture (`T1D-03`) |
| `max_parts` | `250_000` | Slide parts of all kinds | CS-SLD | Tested by tightening; no hostile fixture (`T1D-03`) |
| `max_layout_ids` | `10_000` | Entries in a master's layout list | CS-SLD | Tested by tightening; no hostile fixture (`T1D-03`) |

## Geometry

`crates/casual-slide-geometry/src/lib.rs`, module `limits`, enforced by `RawGeometry::compile`.
The largest preset has 244 guides; real freeforms have many more points, which is why that
cap is the generous one.

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_GUIDES` | `4096` | `av` plus `gd` guides | none yet | Tested; `tests/hostile.rs` |
| `MAX_PATHS` | `1024` | Paths | none yet | Tested |
| `MAX_PATH_POINTS` | `262_144` | Points over all paths | none yet | Tested |
| `MAX_HANDLES` | `256` | Adjust handles | none yet | Tested |
| `MAX_SITES` | `4096` | Connection sites | none yet | Tested |
| `MAX_FORMULA_LEN` | `1024` | Bytes in one formula | none yet | Tested |
| `MAX_NAME_LEN` | `256` | Bytes in one guide name | none yet | Tested |
| `MAX_EXTENT` | `27_273_042_316_900` | Shape extent in EMU, `ST_PositiveCoordinate`'s maximum; larger values are clamped, not refused, in `eval.rs` | none: clamped | Untested by a hostile extent; a compile-time assertion holds it under the arithmetic's limit |
| `MAX_KEPT` | `32` | Diagnostics kept per compile or evaluation; the count stays exact | none: report capped | Tested |
| `MAX_SUBJECT` | `64` | Bytes of **file-supplied text** kept in one diagnostic | none | Tested; but see below |

`GeometryError` has messages and no stable codes, against the rule that every refusal
carries one (`GEO-04`). And a geometry diagnostic carries up to 64 bytes of text from the
file, which the log contract in [20](20-ERROR-CODE-REGISTRY.md) forbids: the same defect
as `SEC-015`, in a crate that row does not name.

## Export

| Limit | Default | Bounds | Code | Status |
| --- | --- | --- | --- | --- |
| `MAX_WRITE_DEPTH` | `16` | Nesting of a variant written back, in `casual-slide-export/src/properties.rs`; the model can hold what import refused only if built by hand | CS-EXP-0008 | Tested |

## Not yet bounded

Each of these is a known gap with an owner, not an oversight:

- **Time.** No limit bounds the time a deck can take within its byte budgets (`SEC-P10`, Tier 1A).
- **Embedded packages.** An embedded deck or workbook is retained, never opened; opening one
  will be a separate admission with an inherited budget (`SEC-P09`).
- **Text, layout and rendering** limits arrive with Tiers 1F and 2D: runs per paragraph, glyphs
  per frame, image pixel dimensions before decode.
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

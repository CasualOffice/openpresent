# 22 — The presentation model

## Shape

```
Presentation
├ slide size, defaults, sections, custom shows, core properties
├ Themes        colour scheme, font scheme, format scheme, per master
├ Masters       background, shape tree, text styles (title, body, other), colour map
│  └ Layouts    background, shape tree, placeholder definitions
├ Slides        ordered; layout ref, hidden flag, background, shape tree,
│               transition, timing tree, notes, comments
└ Media and embedded parts: by content hash, decoded lazily
```

A **shape tree** holds these node kinds: `Shape`, `Picture`, `Group`, `Connector`,
`GraphicFrame` (table, chart, SmartArt, OLE), `Media`, `Ink`. Every node has a transform
(offset, extent, rotation, flips), fill, line, effects, and optionally a text body. A
**text body** holds properties and paragraphs; a paragraph holds properties and runs,
breaks, and fields.

Units stay as the file has them, usually EMU and centipoints. Charts and timing carry doubles and coordinates admit units such as `1in`, so source lexemes are kept (`MODEL-01`).

## Inheritance is resolved, never stored

A property value in the model is what the file said, or absent. Effective values come
from `-style`, walking the chain:

```
run → paragraph → shape text body → shape list style → shape's p:style/fontRef
    → matching placeholder on the layout → the theme's txDef, lnDef, spDef
    → matching placeholder on the master → master text styles
    → presentation default text style
```

Colours resolve through scheme colour → slide or layout colour-map override → master
colour map → theme, then their transforms (`lumMod`, `tint`, `alpha`, and so on).
Placeholder matching follows PowerPoint's rule by index and type; that rule gets its own
fixtures because an error misplaces a whole slide.

The ribbon reads effective values through the same resolver, so "what the ribbon says the
font is" and "what is drawn" cannot differ. Results are cached per object and invalidated
by the op's changed-object set: a master edit invalidates its layouts and slides.

**Theme objects that hold little.** `EffectReference` is `a:effect` in an effect list: `@ref` as a token as written, and nothing resolved. `ThemeManager` is the root of a `themeManager` part, which holds nothing, so it is a unit struct: what an import of that part finds beyond the empty root is reported. `ThemeOverride` is a `themeOverride` part's colour, font and format schemes, each absent if the file left it out.

**A format scheme and object defaults are what a shape holds.** `FormatScheme` holds `Fill`, `Line` and `StyleEffect` (`effects`, `scene3d`, `sp3d`), read by the same functions as a shape's, so `fillRef` and `lnRef` resolve to the type a shape would state for itself. `ShapeDefinition` holds `spPr` as `ShapeProperties`, `bodyPr`, `lstStyle` and `style`. `set_format_scheme`, `set_override_format_scheme` and `set_object_defaults` patch those nodes alone and read back what was asked. A default's `extLst` stays in the part, named by `uri`, and `xfrm` and `@bwMode` in its `spPr` stay and are reported. A style list with an entry past a bound is not patched, as the model's entries no longer pair with the part's by place.

## Identity

Each object has a stable internal `ObjectId`, independent of the file's `spid`, which is
only unique per slide and which PowerPoint can renumber. `spid` is kept for writing back
and for what timing trees and connectors reference. Slides, layouts and masters have ids
too. Ids are what ops and the wire carry.

## Tier 1D: slides and shape trees as built

**Model** (`casual-slide-model`, `slide.rs`, `shape.rs`). The six slide-like parts (`sld`, `sldLayout`, `sldMaster`, `notes`, `notesMaster`, `handoutMaster`) are one `SlidePart` with a kind. A field the kind lacks stays `None`. A part holds `cSld` (name, background, shape tree), `clrMap` or `clrMapOvr`, `showMasterSp`, `showMasterPhAnim`, `show`, the layout attributes, `hf`, `sldLayoutIdLst`, and which of `txStyles` and `notesStyle` are present. A `ShapeNode` is `sp`, `grpSp`, `cxnSp`, `pic`, `graphicFrame` or `contentPart`. It holds `cNvPr` (spid, name, alt text, hidden, hyperlinks with their `ahyp:hlinkClr`, `a16:creationId`, `adec:decorative`; these extensions are read and kept; a change to `a16:creationId` or `adec:decorative` is refused as `CS-SLE-0001`, and `hlinkClr` is written, changed in place or removed with its holders when it is the list's last entry, `CI-24`), the kind's locks, `txBox`, connector ends, `nvPr` (placeholder, media kind, `isPhoto`, `userDrawn`), transform, `bwMode`, `useBgFill`, `p:style`, the frame's graphic URI and the content part's `r:id`. Everything else a node holds is preserved, not modelled, and named in the report as `CS-IMF-0010`. That covers chart content, transitions and timing. Geometry, fills, lines, effects and 3D are Tier 1E's, below.
**Model** (`casual-slide-model`, `slide.rs`, `shape.rs`). The six slide-like parts (`sld`, `sldLayout`, `sldMaster`, `notes`, `notesMaster`, `handoutMaster`) are one `SlidePart` with a kind. A field the kind lacks stays `None`. A part holds `cSld` (name, background, shape tree), `clrMap` or `clrMapOvr`, `showMasterSp`, `showMasterPhAnim`, `show`, the layout attributes, `hf`, `sldLayoutIdLst`, and which of `txStyles` and `notesStyle` are present. A `ShapeNode` is `sp`, `grpSp`, `cxnSp`, `pic`, `graphicFrame` or `contentPart`. It holds `cNvPr` (spid, name, alt text, hidden, hyperlinks with their `ahyp:hlinkClr`, `a16:creationId`, `adec:decorative`; these extensions are read and kept; a change to `a16:creationId` or `adec:decorative` is refused as `CS-SLE-0001`, and `hlinkClr` is written, changed in place or removed with its holders when it is the list's last entry, `CI-24`), the kind's locks, `txBox`, connector ends, `nvPr` (placeholder, media kind, `isPhoto`, `userDrawn`), transform, `bwMode`, `useBgFill`, `p:style`, the frame's graphic URI and the content part's `r:id`. Everything else a node holds is preserved, not modelled, and named in the report as `CS-IMF-0010`. That covers table and chart content. A part's transitions and timing are Tier 1I's, below. Geometry, fills, lines, effects and 3D are Tier 1E's, below.

**Identity.** An `ObjectId` is the part's ordinal in the deck's structure plus the node's place in a depth-first walk of that part's shape tree. Node 0 is the part and node 1 is `spTree`. Ids depend on neither `spid` nor load order. A duplicate `spid` is reported as `CS-IMF-0005` and moves nothing. Ids last for a session and are never written to the file. A node inside `mc:AlternateContent` gets its id in the branch markup compatibility selects.

**Structure and lazy reading** (`casual-slide-import`, `slide.rs`). `index_deck` builds the structure from relationships the package already holds, and reads no slide. The walk is fixed: masters and their layouts, slides with their layout and notes, the notes and handout masters, then one step from each layout to its master. Every step must reach a part whose content type is the kind it names. A relationship cycle or a wrong-kind target is therefore reported as `CS-IMF-0007`, and never followed (`SEC-X07`). A missing layout is reported as `CS-IMF-0008`, and the slide still reads. `LazySlides` reads a part when it is first asked for; a 500-slide deck with 499 corrupt slides indexes and loads its first slide.

**Limits** (`SlideLimits`). Each of these refuses with a stable code:

| Code | Refused when | Default |
| --- | --- | --- |
| `CS-SLD-0001` | Shape-tree nodes in one part exceed the limit | 100 000 |
| `CS-SLD-0002` | Groups nest deeper than the limit | 64 |
| `CS-SLD-0003` | Slides in the structure exceed the limit | 100 000 |
| `CS-SLD-0004` | Slide-like parts, or entries in one `sldLayoutIdLst`, exceed the limit | 250 000 parts, 10 000 entries |
| `CS-SLD-0005` | No part has the ordinal asked for | — |

**Export** (`casual-slide-export`, `slide.rs`). The part is read again with a map from each id to its element, and only differing attributes and children are touched. A new child goes in at its schema position, whatever its siblings' namespaces. An edit outside what this tier writes is refused with one of these codes, and nothing is saved:

| Code | Refused when |
| --- | --- |
| `CS-SLE-0001` | A node is added, removed or reordered; a background or layout list changes; a hyperlink or placeholder is added or removed; a connection or `p:style` on a node that cannot hold one |
| `CS-SLE-0002` | The edit is to a node inside `mc:AlternateContent`, whose other branches would go stale (BEH-25) |
| `CS-SLE-0003` | The edit makes a `cNvPr/@id` collide with another in the part |
| `CS-SLE-0004` | The part's ordinal is not in the deck's structure |

Every patched part is read back before saving.

**Resolution** (`casual-slide-style`). The crate gives effective values with the level each came from: placeholder match, transform, colour map, background, header and footer flags, which master and layout shapes are drawn, the hidden flag, and which master text style applies. The rules and their evidence are in the crate's documentation. The slide-to-layout match by `idx` and the notes match by `type` are documented in `MS-OI29500`, Part 1 Section 19.3.1.36. The layout-to-master match by type, then by class, is Inferred.

## Tier 1E: geometry, fills, lines, effects and 3D as built

**Model** (`casual-slide-model`, `geometry.rs`, `fill.rs`, `line.rs`, `effect.rs`, `d3.rs`, `drawing.rs`). `ShapeNode::properties` is a `ShapeProperties`: `geometry`, `fill`, `line`, `effects`, `scene3d`, `sp3d`, `hidden` (the `a14` extensions in `extLst`), and for a picture its `p:blipFill`. Every field is what the file said or `None`. An absent list and an empty one differ, as `<a:effectLst/>` overrides inheritance and no `effectLst` does not. `Fill`, `Line`, `EffectProperties` and `Blip` are what a theme's format scheme (`T1C-02`), a table cell and a run need too, and `read_fill`, `read_line`, `read_effects`, `read_blip_fill` and `read_preset_text_shape` read them from any element. One `Effect` serves effect lists, DAGs and blips, as `EG_Effect` does. A DAG's reference is a name and never a pointer. Custom geometry keeps its coordinates and formulas as text, as they are numbers or guide names.

**Import** (`casual-slide-import`, the same names). Geometry is checked against `casual-slide-geometry` as it is read: a preset's token and adjust names against its tables, and a custom shape is compiled, with what was repaired reported as `CS-DRW-0004`. Nothing is evaluated at a size, and nothing here says how a fill, line or effect looks (owner: no painting or rendering until Phase 1 closes). Over a `DrawingLimits` bound the whole property is left out and reported as `CS-DRW-0001`, and the tree keeps it, so the deck opens and saves unchanged and a later edit replaces it. A token the schema does not have is reported and left out. Extension items are matched by what they hold and not by `uri`, as real decks spell one `uri` in several cases and without braces. `a14` is read in both its released namespace and the older `.../drawing/2007/7/7/main`, which `fixtures/real/sdk-animation.pptx` uses for the same element; only the released one is written new. A colour's findings in these properties keep their `CS-CLR` codes.

**Export** (`casual-slide-export`, `drawing.rs`). The model, before and after, becomes a small description of the elements it stands for, and only what differs touches the tree. An edited attribute changes that attribute. A child is matched to its element by name and occurrence, whatever the file's order. A choice that changed (a solid fill to a gradient) replaces that element and any repeat of it, in schema order. A list whose order changed is rebuilt. Unknown attributes, foreign children and other `ext` entries stay. A percentage follows the spelling it replaces, or the conformance class. Range is the reader's: the part is read back, and a value the reader would not believe is refused as `CS-EXP-0007`. One check is the writer's (`DRAW-01`): a new `ST_PositiveCoordinate` above 2147483647 (`MS-OI29500` 2.1.1330), on an effect, bevel, extrusion, contour, path or extent, or a new dash stop of zero or less (2.1.1292), is refused as `CS-EXP-0002`, while a value the file already holds stays. An extension is added only with a `uri` from a real deck (`hiddenFill`, `hiddenLine`, `hiddenEffects`, `useLocalDpi`), or the sketch's, which is Inferred; others are patched where a file has them, and adding one is `CS-EXP-0002`.

**Not in this tier.** `p:bgPr` is read by the slide reader as a background and stays retained; its fill is also read as a value with `read_fill`, into `BackgroundProperties::fill`, so a renderer can draw it (`SKEL-01`), and the element is still written back from the tree; a theme's format scheme and object defaults are read with these functions (`T1C-02`); a table cell's fills and borders are read with these functions (Tier 1G), and a run's already are (Tier 1F). `a:txSp` and `useSpRect` live in a locked canvas's graphic data, which no slide reader enters.

**Evidence.** `MS-OI29500` v20260818, by section at each use: 2.1.1270, 2.1.1272, 2.1.1274, 2.1.1276, 2.1.1287, 2.1.1291, 2.1.1292, 2.1.1294, 2.1.1295 to 2.1.1300, 2.1.1303, 2.1.1304, 2.1.1311, 2.1.1312. Where they bear on drawing and not on what is stored, the model's documentation records them for Tier 2B and 2C, and nothing acts on them yet. The sketch extension's `uri` is Inferred.

## Tier 1F: text as built

**Model** (`casual-slide-model`, `text.rs`). A `p:sp` holds a `TextBody`: `bodyPr`, `lstStyle` and paragraphs; a paragraph holds `pPr`, its runs, breaks and fields in order, and `endParaRPr`. `Presentation` holds `defaultTextStyle`, a master its `txStyles`, a notes master its `notesStyle`, all `TextListStyle`. Every attribute of `CT_TextBodyProperties`, `CT_TextParagraphProperties` and `CT_TextCharacterProperties` is a field, and absent stays absent. A body also holds its `prstTxWarp` with `avLst`, its `scene3d`, and the `EG_Text3D` choice of `sp3d` or `flatTx`, as Tier 1E's types. All 41 autonumber schemes are typed. Fonts are as the file states them: typeface, panose, pitch family, charset. Nothing here measures, substitutes or lays anything out, and `normAutofit` is stored, not recomputed. A shape's `lstStyle` and a master's `txStyles` are edited whole (`SetListStyle`, `SetTextStyles`, [24](24-OPERATIONS-AND-COLLABORATION.md)): Slide Size scales every stated `sz` and `buSzPts` with the slide (`UX-015`); `defaultTextStyle` in `presentation.xml` is not scaled yet.

**Lexemes.** Where a type's written form is a union, `Written<T>` keeps the text beside the value: `fontScale="62.5%"` reads as 62500 and stays `62.5%`. `Written` compares by value, so another spelling of the same value is not an edit and the tree's bytes stay. A new value is spelled in the form the element has, or `N%` in Strict. `buSzPct` has only the `N%` form in the vendored Transitional schema too, so a new one is spelled that way, while a real deck's `100000` is read and kept.

**What stays in the tree.** A run's fill, line, effects and underline line and fill, and a body's warp and 3D, are Tier 1E's types, read and written by its readers and writer. `extLst` stays in the tree, and are reported as `CS-IMF-0010`. Something else inside a paragraph, such as an equation's `mc:AlternateContent`, keeps its place as `TextItem::Retained` and is reported. A table cell's text is Tier 1G's.

**Limits** (`TextLimits`, [21](21-PARSER-LIMITS.md)). `CS-TXT-0001` to `0005`. Text never reaches a finding or a message.

**Export** (`casual-slide-export`, `text.rs`). Compared with what the tree says, and only what differs is touched. Paragraphs, runs and tab stops match by position, and the tail grows or shrinks; a retained item is neither moved nor removed. A new element goes in at its schema position. An item the paragraph holds cannot be made as `TextItem::Retained`, such as an inline equation's `mc:AlternateContent` (Tier 1K), as the model holds a name and not its content. So a body with either can be edited in place and not rebuilt once removed. A change leaving out what the schema requires (`bodyPr`, the last `p`, `a:t`, a new hyperlink's `r:id`) is refused too, both as `CS-EXP-0009`, with nothing saved. A shape in an alternate branch is `CS-SLE-0002`, as in Tier 1D.

**Resolution** (`casual-slide-style`, `text.rs`). `Chain::text_chain` gives a shape's levels. Each answer names one of `TextLevel`.

| Family | Chain | Evidence |
| --- | --- | --- |
| Paragraph | `pPr`, then each list style at the paragraph's `lvl`: the shape's, layout placeholder's, the theme's `txDef`, `lnDef`, `spDef`, master placeholder's, master text style, `defaultTextStyle` | `MS-OI29500` 2.1.1388; the order among the three defaults is Inferred from 2.1.1379; the layout's place and the default's are Inferred |
| Run | `rPr` or `endParaRPr`, then `defRPr` at each level of the same list styles | `MS-OI29500` 2.1.1399; the same Inferred order. Leaving out `pPr/defRPr` is Documented: 2.1.1398 (a) says that where the standard takes a run with no `rPr` from `defRPr`, Office instead resolves it as `rPr` describes, and 2.1.1399 (c) lists the ancestors a run inherits from as `lstStyle` children, the theme's, then the master's, never a paragraph's `pPr`. LibreOffice applies it, so decks written by python-pptx's `paragraph.font` differ there (`FH-04`) |
| Run's fill and fonts | As a run, with the shape's `p:style/fontRef` straight after its own `lstStyle`. A colour answers as a solid fill, `idx` as a theme font, and `none` gives no font. A theme's empty `ea` (as Office themes write it) gives way to its `font` for the run's language's script: `ja` `Jpan`, `ko` `Hang`, `zh` `Hans` or `Hant`. Not on a picture or a connector | `MS-OI29500` on 20.1.2.2.37 (a), (c); its place over the layout is Inferred; the script font from ISO 29500 20.1.4.1.16, the language's choice of it Inferred |
| Body | `bodyPr`, the layout placeholder's, the theme's `txDef`, `lnDef`, `spDef`, the master placeholder's | `MS-OI29500` 2.1.1379; the layout's place is Inferred |
| Left out | `defPPr` (2.1.1383, 2.1.1387); `lvl` inside `lvlNpPr` (2.1.1406 b) | Documented |
| Units | Spacing, each bullet group, fill, effects, underline line and fill, autofit, and `tabLst` resolve whole | Inferred |
| Defaults | Only what `MS-OI29500` states: `sz` 1800, `baseline` 0, `marL` and `indent` 0, `defTabSz` 914400, `eaLnBrk` and `hangingPunct` true, `latinLnBrk` false; `grpFill` in text is `noFill` (2.1.1298) | 2.1.1399, 2.1.1406 |

**Open until `PV-01`.** Whether PowerPoint uses `otherStyle` for a shape that is no placeholder. Whether all three object defaults apply to every shape, as 2.1.1379 reads, or "corresponding" narrows them by kind of shape. A default's `style` and `spPr` take no part: the notes name `bodyPr` and `lstStyle`.

## Tier 1J: charts as built

**Generated.** `tools/chart-codegen/generate.py` walks the vendored schemas once and writes three views: `casual-slide-model/src/chart/<ns>.rs`, one struct per type; `casual-slide-import/src/chart/<ns>.rs`, its reader; `casual-slide-export/src/chart/<ns>.rs`, what its values say the tree should hold. `check-repo.py` fails if any differs from what the tool writes. The hand-written part of each crate is its `src/chart.rs`.

**Model.** `ChartPart` by root: `c:chartSpace`, `cx:chartSpace`, `c:userShapes`, `cs:chartStyle`, `cs:colorStyle`. A repeated choice stays one list in the file's order, because that is the order charts draw in: `PlotArea::charts`, `PlotArea::axes`. A graphic frame's `ShapeNode::chart` holds the relationship. `ExtItem` is what `c14`, `c15` and `c16` extensions hold. DrawingML is Tier 1E's `ShapeProperties`, 1F's `TextBody` and 1C's `Color`; any other DrawingML, such as a user shape's non-visual properties, a style part's `defRPr` and `bodyPr`, `clrMapOvr`, and colour transforms in a colour part, is `Retained`: named in the model, kept in the tree, reported as `CS-IMF-0010`, and removable but not creatable. A value in a union with percent and number spellings is kept as written. `AxisId` is signed (`T1J-04`). `mc:AlternateContent` under `c:chartSpace` holding `c14:style` is `AlternateStyle`; one of another shape is reported and kept. `cdr14:contentPart` is `AlternateContentPart`.

**Import.** `read_deck_charts`, from `import_slide_parts`: `ImportedSlides::charts`. A part is read for what its root says. A part that cannot be read, is over a bound in `ChartLimits`, or has the wrong root is set aside, reported, kept byte for byte, and **the deck opens**. An embedded workbook is recorded (`ChartWorkbook`), and no byte of it is read (`SEC-P09`).

**Export.** `patch_chart_part`, `save_chart_parts`: edits patch named nodes, by one function for every type, and read back. `rewrite_from_model` writes from nothing, is never used to save, and exists so that tests hold the writer to every row. Adding or removing a chart part, or changing which chart a frame holds, is refused: those are Phase 6's.

## Tier 1K: SmartArt, equations and ink as built

**Model** (`casual-slide-model`, `diagram.rs`, `math.rs`, `ink.rs`, each with a generated `schema.rs`). SmartArt is the five linked parts of one diagram as typed values; its `dsp:drawing` is what will be drawn, and a layout definition is read as data and never run (`SEC-C12`). An equation is OMML as PowerPoint holds it, in `a14:m`, kept beside its fallback, neither drawn nor regenerated. Ink is the `p:contentPart`'s `inkml:ink` with its context and recorded actions; trace points stay as written. A part that cannot be read is set aside and stays in the package untouched. Bounds are `GraphicLimits` ([21](21-PARSER-LIMITS.md)).

**Known and absent.** 2016 ink brush property names and values are kept as InkML text, and `BrushProperty::extended_name` and `ink_effect` read them as their types (`T1K-04`). The generated modules name no `a:EG_TextBullet`, since a diagram's text is Tier 1F's `TextBody`, bullet included (`T1K-05`). The bounds have fixtures but were not seen failing with their guards removed (`SEC-021`); the `graphics` fuzz target ran 60 s clean (`FUZZ-02`).

**Status marks.** `DGM`, `MATH` and `INK` in `docs/spec/checklist.tsv` hold what `tools/spec-audit/audit.py`'s rule allows, by a run over this tier's decks: 1076 rows `y` in model, import and export, every `extLst` `r` where a fixture holds one, and the blanks with their reasons in `docs/spec/open-rows.tsv`.

## Tier 1G: tables as built

**Model** (`casual-slide-model`, `table.rs`). A graphic frame holds `ShapeNode::table`: `tblPr`, the grid, rows, cells. A cell's text is Tier 1F's `TextBody`, and its fill, borders and `cell3D` Tier 1E's types. `TableStyleList` is the `tableStyles` part, and `TableStyle` is also what `tblPr/tableStyle` holds. Absent stays absent, and a list present and empty is not an absent one. `Table::merges` gives the merges, what does not hold of them, and each covered cell's origin. A row height is what was stored, and nothing here lays anything out (owner instruction 20).

**Import** (`casual-slide-import`, `table.rs`). What `MS-OI29500` says Office requires and the schema does not is **reported and not repaired**: `CS-TBF` in [20](20-ERROR-CODE-REGISTRY.md). The exception is a span under 1 or leaving the table, which is left out, as no consumer should have to bound it. A cell's place in its row is its column (2.1.1424 c). `import_slide_parts` reads the part from the main part's relationship and returns it as `ImportedSlides::table_styles`, and each `tableStyleId` naming nothing is `CS-TBF-0008`. Over a bound, `CS-TBL`. `a16:rowId`, `a16:colId` and every `extLst` stay in the tree and are reported as `CS-IMF-0010`.

**Export** (`casual-slide-export`, `table.rs`). Rows, cells and columns match by position, and only what differs is touched. A table, row, cell or column added or removed is refused as `CS-EXP-0009`: those are edit operations, which move ids and merges together. `save_table_styles` writes the part alone. Adding a `tableStyles` part to a deck with none is refused as `CS-SLE-0001`, and is the writer tier's. A style reference is written as the model states it, and nothing keeps it naming a listed style.

**Resolution** (`casual-slide-style`, `table.rs`). `table_style` says where a table's style comes from, and `StyleSource::NotInDeck` is an answer and not an error. `CellChain` answers fill, bold, italic, text colour, font, margins, anchor and overflow from the cell, then the part styles. Positions are grid positions and never sizes.

| Rule | Evidence |
| --- | --- |
| Part styles apply whole table, bands, last and first column, last row, its corners, first row, its corners | `MS-OI29500` 2.1.1265 |
| An invalid `tableStyleId` applies no style, if it is neither built in nor in the file | `MS-OI29500` 2.1.1423 |
| `tcPr` over the style; flags turn on the parts of the same name; corners need both flags; bands count past the header and skip a header or total; a covered cell answers as its origin | Inferred |

**Known and absent.** **We hold no definitions of Office's built-in styles**, and real decks refer to one with an empty list (`z19828779-education-ar`, `z17310490-vierfelder-de`), so such a table resolves to no style here and is drawn styled by PowerPoint. A style's borders are read and not resolved to a cell's edges, as which of two neighbours' borders shows is Tier 2E's (BEH-14, 2.1.1424 e, f), and so is row growth (2.1.1427). `MS-OI29500` 2.1.1415's `dkEdge` and `flood` are recorded on the model and not resolved.

**Status marks.** `TABLE` in `docs/spec/checklist.tsv` holds what `tools/spec-audit/audit.py --check` allows: 87 rows `y` in model, import and export, every `extLst` `r`, and 17 with a blank, which are rows the audit cannot establish and not rows left unread. `every_table_row_in_the_checklist_is_read_or_reported` holds all 113. The reasons: a type that declares an `extLst` cannot be `y`, as `extLst` is retained (9 rows, all three columns); a root has no field to declare it (`tbl` and `tblStyleLst`, model); and a type or group built on a themeable fill, effect or colour is not yet held by its own rows (6 rows, export).

**A disagreement in the notes.** 2.1.1427 b says a row holds more than 1 cell, and 2.1.1425 allows one column. A single-column table is not reported.
## Tier 1H: pictures, SVG, audio and video as built

**Model** (`casual-slide-model`, `picture.rs`, `media.rs`, additions in `fill.rs`, `shape.rs`, `text.rs`). A reference is a relationship id and nothing else. `Blip` adds `svg` (`asvg:svgBlip`, of which the blip is the fallback), `image_formula` and `oembed`. `ShapeNode` adds `prefer_relative_resize`, and `poster()`, the blip of the `p:pic` a clip sits on. `AppNonVisual::media` is `Media`, the `a:EG_Media` choice with what it holds, and `media_extension` is `p14:media`: its references, `trim`, `fade`, `bmkLst`, and `p173:tracksInfo`. `Hyperlink::sound` is `snd`. `Bullet::Picture` holds a whole `Blip`. Times stay strings as written: the schema's `ST_UniversalTimeOffset` is a string.

**By content hash.** `MediaCatalog` lists each picture and clip part with its SHA-256, declared content type, sniffed format, header size, and every relationship reaching it, and each external target. It is made on request by `catalog_media`, never by opening a deck (`docs/28`). SHA-256 is written in `casual-slide-import`, to take no new dependency.

**What is never done.** Nothing decodes a picture or parses a clip. An SVG is hashed and read by nothing else: the XML walker never sees it, it is neither sniffed nor measured, and no value from it reaches the model or a report. A link, a linked clip, a caption track and an `oEmbedUrl` are never fetched, and are reported.

**Retained and reported.** `pic:pic` (a Word picture in a graphic frame), `a:pic` in `lockedCanvas`, and `pic14:*` inside them are kept in the tree and reported through the element holding them, `pic` or `lockedCanvas`. The `extLst` of `a:picLocks`, `p:cNvPicPr`, `a:audioFile`, `a:videoFile`, `a:quickTimeFile` and `a:audioCd` is kept and reported, as is any other entry in `nvPr/p:extLst`, `p14:media/p14:extLst` and `a:blip/a:extLst`.

**Export** (`casual-slide-export`, `media.rs`, and `fill.rs`, `drawing.rs`, `slide.rs`, `text.rs`). As Tier 1E: the model before and after become small descriptions, and only what differs touches the tree. `p14:media` is in `nvPr/p:extLst` and `tracksInfo` in `p14:media/p14:extLst` (the schema qualifies it; the SDK confirmed this against a first attempt at `p:extLst`), next to whatever else is there. A new `svgBlip`, `imgProps` or `p14:media` takes the `uri` Office gives it. `imageFormula` and `oembed` have no documented `uri`, so the model keeps the one the file wrote, and neither can be made where no file said one.

**Replacing media.** `replace_media` changes the reference in the model and the tree, and says what follows: a relationship to add, unless one already reaches the target (`INV-RI-10`), with a new id by `INV-RI-03`; the old one to release, only if no `r:*` attribute anywhere in the part still names it, retained XML and alternate branches counted; and through `parts_released` and the catalog, which media parts lose their last relationship from anywhere in the deck (`INV-SP-02`). A shared part is never overwritten. Half of an SVG and its fallback is refused. **Writing `.rels` and removing parts is Tier 1N's** and is not done here, so until then only a replacement reusing an existing relationship can be saved as a deck.

**Rows.** `tools/spec-audit` marks PIC and MEDIA under Tier 1H. A row the code handles but the audit's rule cannot show stays blank for a named reason. `M`: `picLocks`, where the model's `locks` is not named after the row, and a global element, which is no field. A simple type is held by the model, so only `M` applies. A type holding `extLst` (`CT_AudioFile`, `CT_VideoFile`, `CT_QuickTimeFile`, `CT_AudioCD`, `CT_PictureLocking`, `CT_NonVisualPictureProperties`, `CT_Blip`) never earns `y`, by the rule, and `CT_PictureLocking/AG_Locking` waits on `SHAPE`'s rows. `r` marks what is kept and reported: every `extLst` here, `pic:*`, `a:CT_GvmlPicture*`, `pic14:*`, and `a:blip` and `a:videoFile` as global elements in an extension list.

**Limits** (`MediaLimits`, [21](21-PARSER-LIMITS.md)) and codes `CS-MED-0001` to `0009`, `CS-MDE-0001` and `0002` ([20](20-ERROR-CODE-REGISTRY.md)).

**Evidence.** `MS-OI29500` v20260818, by section: 21.3.2.2 `dpi` is not supported by Office; 20.1.2.2.31 which locks Office ignores on a picture; 20.1.3.2 and 20.1.3.6 Office ignores `contentType` and does not round trip it. Each changes no stored value. `MS-PPTX` "Media Part", "Track Part", "Media", "Media Extensions" and "TracksInfo Extensions" for relationship types and `uri`s. `MS-ODRAWXML` "Pictures" for the SVG's PNG fallback. **Inferred:** `imageFormula` and `oembed` under `a:blip/a:extLst`, as no document here names the parent and no deck has one; `pic14:*` under `mc:AlternateContent` in `pic:pic`; the relationship types `a:audioFile`, `a:videoFile` and `a:quickTimeFile` may use; which content types promise which image format; every default bound. No real deck here holds audio or video, so those rows are proven on synthetic fixtures only (`F-007`).
## Tier 1I: timing and transitions as built

**Model** (`casual-slide-model`, `timing.rs`, `transition.rs`). `SlidePart::timing` and `SlidePart::transition` are each a `Slot`, on a slide, layout or master. `Slot::Plain` is the element in the part. `Slot::Alternate` is `mc:AlternateContent`, with every branch as values, its `Requires` as written, and `selected`, the branch `casual-slide-ooxml`'s resolver chose: its understood list is ECMA-376 alone, so a `Choice` requiring `p14` gives way to its `Fallback`, and `resolved()` answers that. `Slot::Unread` is what could not be read: it stays whole in the tree. `TimeNode` is every member of `CT_TimeNodeList`, and `TimeNode::Alternate` the same construct inside a list. Every attribute of `cTn`, `cBhvr`, each behaviour, condition, target and build is a field, and absent stays absent: neither the schema's defaults nor PowerPoint's are filled in. `TransitionEffect` is the 21 base effects, the 19 from the 2010 namespace, `p15:prstTrans` and Morph, with `p14:dur` as written. What a behaviour says (`@from`, `@path`, `@fmla`, `@cmd`, `@filter`) is a string, kept, and never evaluated. `BEH-23`, what any of this does when it runs, is Phase 8's.

**Identity.** A target is a `spid`, the file's `cNvPr/@id`, and never an `ObjectId`. `SlidePart::animated_node` finds the node it means, whose `ObjectId` is what ops carry, in the shape tree, which holds the branch the resolver selects. `SlidePart::retarget_shape` changes a node's `spid` and its targets and builds together. A shape only in a branch not taken is still a shape: both the reader's check and the writer's scan the tree.

**Import** (`casual-slide-import`, the same names). A value that cannot be used is reported as `CS-IMF-0001` or `0002` and left out. A repeat of what may appear once is `CS-IMF-0004`, and the first stands. Anything else is `CS-IMF-0010` or `0011`, and stays. What timing names is checked and never acted on: `CS-TMG-0003`, `0004` and `0005`, `CS-IMF-0005`, `0006` and `0008`. Over a `TimingLimits` bound, or where the reader fails, the slot is `Unread`, reported as `CS-TMG-0001` or `0002`, and the rest of the part reads (owner instruction 24, `SEC-C10`). `p14:dur` is also read in the older namespace `fixtures/real/sdk-animation.pptx` holds.

**Export** (`casual-slide-export`, the same names). Tier 1E's: the model, before and after, becomes a description, and only what differs touches the tree. A percentage follows the spelling it replaces, or Strict's `N%`. A new element goes in at its schema position, and `p:transition` before `p:timing` whichever is in `mc:AlternateContent`. An edit to one branch touches that branch: the model holds both, so nothing is stale that a caller cannot see. New `mc:AlternateContent` declares what `Requires` names on its `Choice`. Refusals are in [20](20-ERROR-CODE-REGISTRY.md). A file's own dangling target does not stop another edit; a new one is `CS-SLE-0005`, a change of `spid` without its targets included.

**Rows.** Every row is marked by `tools/spec-audit`, or has a reason. Left blank: `CT_SlideTiming` and `CT_SlideTransition`, whose `extLst` is retained (`r`); the unions `ST_TLTime`, `ST_TLTimeNodeID`, `ST_TLTimeAnimateValueTime`, `ST_TransitionEightDirectionType`, `ST_AnimationDgmBuildType`, `ST_AnimationChartBuildType`, as a simple type with no values of its own has nothing for the audit to hold; `<morph>`, a root row, whose attribute and values are marked; and the 15 rows of `x:drawing/2018/animation`, `animPr`, which is not in `p:timing` and is a 3D model's playback, Tier 1L's. The values of the members of a union are `M` alone, as the audit asks for an attribute of exactly that type: the fixtures hold each, and `every_row_of_timing_and_trans_is_in_a_fixture_or_has_a_named_reason` and the rewrite of those fixtures carry them. The 2010 effects and `p15:prstTrans` are `EXT` rows, whose marks are Tier 1M's; the code and fixtures for them are here.

**Evidence.** `MS-OI29500` v20260818, read as data, by section: 2.1.1128 (`sld`), 2.1.1138 (`anim`), 2.1.1145 (`audio`), 2.1.1149 (`bldP`), 2.1.1152 (`cBhvr`), 2.1.1156 (`cond`), 2.1.1157 (`cTn`), 2.1.1173 (`seq`), 2.1.1176 (`spTgt`), 2.1.1177 (`subSp`), 2.1.1181 (`tnLst`), 2.1.1186 (`wheel`), 2.1.1192 (`ST_TLTime`). The other notes in 2.1.1139 to 2.1.1204 were not read (`SPEC-02`), so nothing here rests on them. What they say of PowerPoint's defaults is for Phase 8: `calcmode` lin, `valueType` num, `additive` base, `accumulate` none, `xfrmType` pt, `dur` 0, `restart` always, `display` true, `afterEffect` false, `nodePh` false, `cond/@delay` 0. **Inferred:** that a shape in a branch not taken satisfies a target; that `presetBounceEnd` is enough to keep a node in a branch; `p15:prstTrans`'s names in `fixtures/generated`; and placing the older `dur` namespace with the released one, from one real deck. No committed real deck holds a 2010 effect, `prstTrans` or Morph, so those are shown against the schemas and validators only, until `PV-01`.

## Tier 1L: OLE, VML and 3D models as built

**Model** (`casual-slide-model`, `retained.rs`). An OLE object, an ActiveX control or a 3D model is kept as bytes and never opened (`SEC-C07`): only who it is and which image stands in for it are modelled, and nothing it names is followed. VML drawing parts are retained whole. **Not yet:** VML is not schema-checked in F4 (`F4-04`).

## Tier 1M: comments, sections, zoom, labels and retained data as built

**Model** (`casual-slide-model`, `comment.rs`, `section.rs`, `custom_data.rs`). `DeckComments` holds both comment forms: `commentAuthors` and `comments` parts as `CommentAuthorList` and `CommentList`, and `authors` and modern parts as `AuthorList` and `ThreadedCommentList`, with replies, reactions, tasks and the anchor's monikers as read. `Presentation::sections` is `p14:sectionLst`, and `SectionList::check_against` says how it differs from `sldIdLst`. `PartMarks` holds a slide-like part's zooms, classification marks, Designer marks, `commentRel` and `custDataLst` references. `DeckCustomData` holds tags and custom XML properties. `DeckLabels` holds sensitivity labels (`ENT-04`). Comment text, names, user ids, tags and titles are document data, and reach no finding.

**Import** (`comment.rs`, `section.rs`, `custom_data.rs`). `import_review` reads all of it and cannot fail. It is asked for and is not part of opening a deck, as it reads every slide. A part that cannot be read, is wrongly rooted, or crosses a `ReviewLimits` bound is set aside: named in `set_aside` and the report, and kept byte for byte (owner instruction 24). An over-long section list is reported and left out, and stays in the tree. An extension is matched by what it holds and not by its `uri`. Change tracking, revision info and custom XML item bodies are retained whole, and only their name and kind are reported as `CS-RVW-0012`: `pc:chgInfo` and its monikers are not parsed. A section list out of step opens, is kept, and is reported (`INV-SL-05`).

**Export** (`comment.rs`, `section.rs`, `custom_data.rs`). The model before and after become small descriptions, and only what differs touches the tree. `save_review` writes only parts that differ. Comments, replies, authors and sections match by `id`, others by place, so removing one leaves the others' elements alone. Refused as `CS-EXP-0009`: reordering such a list, changing a comment's anchor, a new part where the deck has none, adding a zoom or mark, or a `commentRel` or `custDataLst` reference. A section list added or a section removed by a model through `save_deck` is refused by preflight (`INV-SL-01`, `INV-SL-03`), since the writer's section operations are how those are asked for, and `patch_presentation` writes them for those operations.

**Known and absent.** No deck in `fixtures/real` holds a modern comment, a section, a zoom, a sensitivity label or a Designer mark. They hold author lists, change records, revision info and custom XML, so what the rest looks like comes from the `MS-PPTX` schemas, and PowerPoint's use is Unverified until `PV-01`. Inferred, each marked in the code: a mention as `a:fld` of type `mention`; `threadingInfo`'s `uri`; the sensitivity label part and `MSIP_Label_*` properties. **The `uri`s of `reactions` and `taskDetails` are not known here**, so one can be edited where a file has it, and adding the first is refused until a real deck or `PV-01` says what to write.

**Rows.** `tools/spec-audit` marks all 876. Modelled rows are `y`, whole retained parts and other tiers' extension elements `r`. The moniker types a change record holds through `xsd:any` have no global element, so the audit's walk cannot reach them: blank in `docs/spec/open-rows.tsv`, and `moniker_types_are_in_the_fixtures_and_kept` holds that they are present.

## The fidelity contract

Named by dimension. "Lossless" is not used.

| Dimension | Contract | Gate |
| --- | --- | --- |
| Unedited save | Output bytes equal input bytes | Corpus byte compare |
| Edited save, untouched parts | Each part's bytes unchanged; only touched parts and the bookkeeping parts they require are rewritten | Per-part hash compare |
| Edited part, unknown content | Unmodelled elements, attributes and `extLst` retained in place, including `mc:AlternateContent` choices | Semantic diff on the rewritten part |
| Semantic | Import → export → import yields an equal model | Fixed-point test |
| Rendering | Within per-fixture pixel threshold of reference | Fidelity tool |
| Loss | Whatever cannot be kept is counted and named | Report gate; a silent drop fails CI |

**Bookkeeping parts** deserve care: content types, the presentation relationship list and
`presentation.xml` change when slides are added, and `app.xml` holds counts. Rewriting them
is fine and is documented, so the part-hash gate has an explicit allow-list per edit kind.

## Retained vs. drawn

| Construct | Plan |
| --- | --- |
| SmartArt | Edit (`DGM-01`): render the stored drawing part; layout engine in 2J; data parts retained until then |
| Charts | Model and plot; embedded workbook retained verbatim; edit its data through OpenCalc's SDK |
| Equations (OMML) | Edit (`DGM-01`): math layout in 2K; fallback image retained until then |
| OLE, VBA, ActiveX | Retain; draw the fallback image; never execute |
| Linked media and images | Never fetched automatically; the host decides |
| 3D models, Zoom slides, Morph | Render fallback where one exists; named in the report |

## Limits

Threats and controls are in [07](07-SECURITY-THREAT-MODEL.md).

A presentation admits the package limits from OpenCalc's docs/21, plus per-construct ceilings
in the same contract: shape count per slide, nesting depth of groups, paragraph and run counts,
image pixel count before decode, timing-tree depth, and custom-geometry path count. A hostile
deck must be rejected or truncated with a report entry, never allowed to allocate freely.
Values are set in Phase 0 and asserted by fixtures.

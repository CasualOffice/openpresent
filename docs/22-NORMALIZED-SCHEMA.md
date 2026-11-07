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
run → paragraph → shape text body → shape list style
    → matching placeholder on the layout → matching placeholder on the master
    → master text styles → presentation default text style
```

Colours resolve through scheme colour → slide or layout colour-map override → master
colour map → theme, then their transforms (`lumMod`, `tint`, `alpha`, and so on).
Placeholder matching follows PowerPoint's rule by index and type; that rule gets its own
fixtures because an error misplaces a whole slide.

The ribbon reads effective values through the same resolver, so "what the ribbon says the
font is" and "what is drawn" cannot differ. Results are cached per object and invalidated
by the op's changed-object set: a master edit invalidates its layouts and slides.

## Identity

Each object has a stable internal `ObjectId`, independent of the file's `spid`, which is
only unique per slide and which PowerPoint can renumber. `spid` is kept for writing back
and for what timing trees and connectors reference. Slides, layouts and masters have ids
too. Ids are what ops and the wire carry.

## Tier 1D: slides and shape trees as built

**Model** (`casual-slide-model`, `slide.rs`, `shape.rs`). The six slide-like parts (`sld`, `sldLayout`, `sldMaster`, `notes`, `notesMaster`, `handoutMaster`) are one `SlidePart` with a kind. A field the kind lacks stays `None`. A part holds `cSld` (name, background, shape tree), `clrMap` or `clrMapOvr`, `showMasterSp`, `showMasterPhAnim`, `show`, the layout attributes, `hf`, `sldLayoutIdLst`, and which of `txStyles` and `notesStyle` are present. A `ShapeNode` is `sp`, `grpSp`, `cxnSp`, `pic`, `graphicFrame` or `contentPart`. It holds `cNvPr` (spid, name, alt text, hidden, hyperlinks, `a16:creationId`, `adec:decorative`), the kind's locks, `txBox`, connector ends, `nvPr` (placeholder, media kind, `isPhoto`, `userDrawn`), transform, `bwMode`, `useBgFill`, the frame's graphic URI and the content part's `r:id`. Everything else a node holds is preserved, not modelled, and named in the report as `CS-IMF-0010`. That covers chart content, transitions and timing. Geometry, fills, lines, effects and 3D are Tier 1E's, below.

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
| `CS-SLE-0001` | A node is added, removed or reordered; a background or layout list changes; a hyperlink or placeholder is added or removed |
| `CS-SLE-0002` | The edit is to a node inside `mc:AlternateContent`, whose other branches would go stale (BEH-25) |
| `CS-SLE-0003` | The edit makes a `cNvPr/@id` collide with another in the part |
| `CS-SLE-0004` | The part's ordinal is not in the deck's structure |

Every patched part is read back before saving.

**Resolution** (`casual-slide-style`). The crate gives effective values with the level each came from: placeholder match, transform, colour map, background, header and footer flags, which master and layout shapes are drawn, the hidden flag, and which master text style applies. The rules and their evidence are in the crate's documentation. The slide-to-layout match by `idx` and the notes match by `type` are documented in `MS-OI29500` 2.1.1127. The layout-to-master match by type, then by class, is Inferred.

## Tier 1E: geometry, fills, lines, effects and 3D as built

**Model** (`casual-slide-model`, `geometry.rs`, `fill.rs`, `line.rs`, `effect.rs`, `d3.rs`, `drawing.rs`). `ShapeNode::properties` is a `ShapeProperties`: `geometry`, `fill`, `line`, `effects`, `scene3d`, `sp3d`, `hidden` (the `a14` extensions in `extLst`), and for a picture its `p:blipFill`. Every field is what the file said or `None`. An absent list and an empty one differ, as `<a:effectLst/>` overrides inheritance and no `effectLst` does not. `Fill`, `Line`, `EffectProperties` and `Blip` are what a theme's format scheme (`T1C-02`), a table cell and a run need too, and `read_fill`, `read_line`, `read_effects`, `read_blip_fill` and `read_preset_text_shape` read them from any element. One `Effect` serves effect lists, DAGs and blips, as `EG_Effect` does. A DAG's reference is a name and never a pointer. Custom geometry keeps its coordinates and formulas as text, as they are numbers or guide names.

**Import** (`casual-slide-import`, the same names). Geometry is checked against `casual-slide-geometry` as it is read: a preset's token and adjust names against its tables, and a custom shape is compiled, with what was repaired reported as `CS-DRW-0004`. Nothing is evaluated at a size, and nothing here says how a fill, line or effect looks (owner: no painting or rendering until Phase 1 closes). Over a `DrawingLimits` bound the whole property is left out and reported as `CS-DRW-0001`, and the tree keeps it, so the deck opens and saves unchanged and a later edit replaces it. A token the schema does not have is reported and left out. Extension items are matched by what they hold and not by `uri`, as real decks spell one `uri` in several cases and without braces.

**Export** (`casual-slide-export`, `drawing.rs`). The model, before and after, becomes a small description of the elements it stands for, and only what differs touches the tree. An edited attribute changes that attribute. A child is matched to its element by name and occurrence, whatever the file's order. A choice that changed (a solid fill to a gradient) replaces that element and any repeat of it, in schema order. A list whose order changed is rebuilt. Unknown attributes, foreign children and other `ext` entries stay. A percentage follows the spelling it replaces, or the conformance class. Nothing is range-checked at write: the part is read back, and a value the reader would not believe is refused as `CS-EXP-0007`. An extension is added only with a `uri` from a real deck (`hiddenFill`, `hiddenLine`, `hiddenEffects`, `useLocalDpi`), or the sketch's, which is Inferred; others are patched where a file has them, and adding one is `CS-EXP-0002`.

**Not in this tier.** `p:bgPr` is read by the slide reader as a background and stays retained; `T1C-02` reads the format scheme's fills; a table cell's fills and borders are read with these functions (Tier 1G), and a run's already are (Tier 1F). `flatTx` and `prstTxWarp` are in a text body.

**Evidence.** `MS-OI29500` v20260818, by section at each use: 2.1.1270, 2.1.1272, 2.1.1274, 2.1.1276, 2.1.1287, 2.1.1291, 2.1.1292, 2.1.1294, 2.1.1295 to 2.1.1300, 2.1.1303, 2.1.1304, 2.1.1311, 2.1.1312. Where they bear on drawing and not on what is stored, the model's documentation records them for Tier 2B and 2C, and nothing acts on them yet. The sketch extension's `uri` is Inferred.

## Tier 1F: text as built

**Model** (`casual-slide-model`, `text.rs`). A `p:sp` holds a `TextBody`: `bodyPr`, `lstStyle` and paragraphs; a paragraph holds `pPr`, its runs, breaks and fields in order, and `endParaRPr`. `Presentation` holds `defaultTextStyle`, a master its `txStyles`, a notes master its `notesStyle`, all `TextListStyle`. Every attribute of `CT_TextBodyProperties`, `CT_TextParagraphProperties` and `CT_TextCharacterProperties` is a field, and absent stays absent. All 41 autonumber schemes are typed. Fonts are as the file states them: typeface, panose, pitch family, charset. Nothing here measures, substitutes or lays anything out, and `normAutofit` is stored, not recomputed.

**Lexemes.** Where a type's written form is a union, `Written<T>` keeps the text beside the value: `fontScale="62.5%"` reads as 62500 and stays `62.5%`. `Written` compares by value, so another spelling of the same value is not an edit and the tree's bytes stay. A new value is spelled in the form the element has, or `N%` in Strict. `buSzPct` has only the `N%` form in the vendored Transitional schema too, so a new one is spelled that way, while a real deck's `100000` is read and kept.

**What stays in the tree.** A run's fill, line, effects and underline line and fill are Tier 1E's types, read and written by its readers and writer. `scene3d`, `sp3d`, `flatTx`, a warp's `avLst`, `a:blip`'s content in a picture bullet beyond `r:embed`, and `extLst` stay in the tree, and are reported as `CS-IMF-0010`. Something else inside a paragraph, such as an equation's `mc:AlternateContent`, keeps its place as `TextItem::Retained` and is reported. A table cell's text is read the same way (Tier 1G).

**Limits** (`TextLimits`, [21](21-PARSER-LIMITS.md)). `CS-TXT-0001` to `0005`. Text never reaches a finding or a message.

**Export** (`casual-slide-export`, `text.rs`). Compared with what the tree says, and only what differs is touched. Paragraphs, runs and tab stops match by position, and the tail grows or shrinks; a retained item is neither moved nor removed. A new element goes in at its schema position. A picture bullet can be kept or removed and not made, and a change leaving out what the schema requires (`bodyPr`, the last `p`, `a:t`, a new hyperlink's `r:id`) is refused too, both as `CS-EXP-0009`, with nothing saved. A shape in an alternate branch is `CS-SLE-0002`, as in Tier 1D.

**Resolution** (`casual-slide-style`, `text.rs`). `Chain::text_chain` gives a shape's levels. Each answer names one of `TextLevel`.

| Family | Chain | Evidence |
| --- | --- | --- |
| Paragraph | `pPr`, then each list style at the paragraph's `lvl`: the shape's, layout placeholder's, master placeholder's, master text style, `defaultTextStyle` | `MS-OI29500` 2.1.1388; the layout's place and the default's are Inferred |
| Run | `rPr` or `endParaRPr`, then `defRPr` at each level of the same list styles | `MS-OI29500` 2.1.1399; leaving out `pPr/defRPr` is Inferred from 2.1.1398 and 2.1.1399 |
| Body | `bodyPr`, the layout placeholder's, the master placeholder's | `MS-OI29500` 2.1.1379; the layout's place is Inferred |
| Left out | `defPPr` (2.1.1383, 2.1.1387); `lvl` inside `lvlNpPr` (2.1.1406 b) | Documented |
| Units | Spacing, each bullet group, fill, effects, underline line and fill, autofit, and `tabLst` resolve whole | Inferred |
| Defaults | Only what `MS-OI29500` states: `sz` 1800, `baseline` 0, `marL` and `indent` 0, `defTabSz` 914400, `eaLnBrk` and `hangingPunct` true, `latinLnBrk` false; `grpFill` in text is `noFill` (2.1.1298) | 2.1.1399, 2.1.1406 |

**Known and absent.** `MS-OI29500` 2.1.1379, 2.1.1388 and 2.1.1399 put the theme's object defaults (`txDef`, `lnDef`, `spDef`) before the master, and a shape's `p:style/fontRef` is overridden by its text body. Neither is in the chain, because neither is on the model yet. Whether PowerPoint uses `otherStyle` for a shape that is no placeholder is open until `PV-01`.

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

**A disagreement in the notes.** 2.1.1427 b says a row holds more than 1 cell, and 2.1.1425 allows one column. A single-column table is not reported.

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

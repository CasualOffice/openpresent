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

**Model** (`casual-slide-model`, `slide.rs`, `shape.rs`). The six slide-like parts (`sld`, `sldLayout`, `sldMaster`, `notes`, `notesMaster`, `handoutMaster`) are one `SlidePart` with a kind. A field the kind lacks stays `None`. A part holds `cSld` (name, background, shape tree), `clrMap` or `clrMapOvr`, `showMasterSp`, `showMasterPhAnim`, `show`, the layout attributes, `hf`, `sldLayoutIdLst`, and which of `txStyles` and `notesStyle` are present. A `ShapeNode` is `sp`, `grpSp`, `cxnSp`, `pic`, `graphicFrame` or `contentPart`. It holds `cNvPr` (spid, name, alt text, hidden, hyperlinks, `a16:creationId`, `adec:decorative`), the kind's locks, `txBox`, connector ends, `nvPr` (placeholder, media kind, `isPhoto`, `userDrawn`), transform, `bwMode`, `useBgFill`, the frame's graphic URI and the content part's `r:id`. Everything else a node holds is preserved, not modelled, and named in the report as `CS-IMF-0010`. That covers geometry, fills, lines, effects, table and chart content, `blipFill`, transitions and timing. Text bodies joined in Tier 1F, below.

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

## Tier 1F: text as built

**Model** (`casual-slide-model`, `text.rs`). A `p:sp` holds a `TextBody`: `bodyPr`, `lstStyle` and paragraphs; a paragraph holds `pPr`, its runs, breaks and fields in order, and `endParaRPr`. `Presentation` holds `defaultTextStyle`, a master its `txStyles`, a notes master its `notesStyle`, all `TextListStyle`. Every attribute of `CT_TextBodyProperties`, `CT_TextParagraphProperties` and `CT_TextCharacterProperties` is a field, and absent stays absent. All 41 autonumber schemes are typed. Fonts are as the file states them: typeface, panose, pitch family, charset. Nothing here measures, substitutes or lays anything out, and `normAutofit` is stored, not recomputed.

**Lexemes.** Where a type's written form is a union, `Written<T>` keeps the text beside the value: `fontScale="62.5%"` reads as 62500 and stays `62.5%`. `Written` compares by value, so another spelling of the same value is not an edit and the tree's bytes stay. A new value is spelled in the form the element has, or `N%` in Strict. `buSzPct` has only the `N%` form in the vendored Transitional schema too, so a new one is spelled that way, while a real deck's `100000` is read and kept.

**What stays in the tree.** A run's fill says which it is (`TextFill`), and only a solid colour's content is modelled. `ln`, `uLn`, effects, `scene3d`, `sp3d`, `flatTx`, `avLst`, `a:blip`'s other content and `extLst` stay, are reported as `CS-IMF-0010`, and join when Tier 1E and 1H model them. Something else inside a paragraph, such as an equation's `mc:AlternateContent`, keeps its place as `TextItem::Retained` and is reported. A table cell's text is Tier 1G's.

**Limits** (`TextLimits`, [21](21-PARSER-LIMITS.md)). `CS-TXT-0001` to `0005`. Text never reaches a finding or a message.

**Export** (`casual-slide-export`, `text.rs`). Compared with what the tree says, and only what differs is touched. Paragraphs, runs and tab stops match by position, and the tail grows or shrinks; a retained item is neither moved nor removed. A new element goes in at its schema position. What another tier owns can be kept or removed and not made, and a change leaving out what the schema requires (`bodyPr`, the last `p`, `a:t`, a new hyperlink's `r:id`) is refused too, both as `CS-EXP-0009`, with nothing saved. A shape in an alternate branch is `CS-SLE-0002`, as in Tier 1D.

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

# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/), as OpenCalc's does. Semantic versioning
applies once a published line begins; until then everything is under **Unreleased**,
grouped by date. Each entry cites the tracker id in
[docs/14](docs/14-EXECUTION-TRACKER.md) that drove it.

Nothing has been released. Release tags and their dry-run default are described in
[docs/15](docs/15-CI-AND-RELEASE-GATES.md).

## Unreleased

### 2026-11-09

**Added: side panes, comments, notes and the Selection Pane (`UX-011`)**

- One task pane host at the right; Format Shape and the Animation Pane moved onto it.
- Comments pane: modern threaded comments on a slide or shape, replies, resolve, reopen, delete, markers, saved through the Tier 1M model; @mention text is sent nowhere.
- Speaker notes typed under the slide and saved to the notes slide, created when a slide has none.
- Selection Pane: list, rename, hide or show, reorder by drag; engine ops `SetComments`, `SetAuthors`, `SetNotes`, `SetNonVisual`.

**Added: the File tab and autosave (`FILE-01`, `SAVE-01`)**

- Rename in the title bar; File > Info properties and Advanced Properties (Summary, Statistics, Custom) saved into `docProps`, one undo step each.
- Save As `.pptx`, `.potx`, `.ppsx` (F12) through the save-as grid; Download a Copy; Print, one slide a page.
- Browser autosave to IndexedDB and a recovery bar, copied from OpenCalc's `editor.drafts.js`.

**Docs**

- PowerPoint parity audit of the live editor: 352 features, 35 working (9.9%), backlog `PAR-01` to `PAR-66` (`FIT-02`).
- Every document brought true to the code and git at `8011d28` (`DOC-011`).

### 2026-11-08

**Phase 1 closed.** Every tier, 1A to 1N and 1R, closed; the owner signed off on the phase1-exit run at `50ea80d` (`P1X-01`, instruction 53).

**Added: rendering (Phase 2, under instruction 43's waiver)**

- `casual-slide-text`: text bodies laid out into glyph runs, caret stops, copied from OpenCalc's text crate (`T2D1-01` to `T2D4-01`).
- `casual-slide-render`: slides to PNG (tiny-skia) and to PDF with real, extractable text (`SKEL-01`, `VS-04`); tables, classic charts, SmartArt from its drawing, OLE from its fallback, shadow, glow, autofit at its stored scale (`SKEL-03`); highlight (`UX-005`).
- Fidelity fixes from the gallery's scores: group scale, pattern and tiled fills, CJK, vertical text, hyperlink colour, EMF-wrapped bitmaps, symbol fonts, Gaussian blur (`FH-03`, `FH-04`, `FH-05`).

**Added: vertical slices 1 to 12 (instruction 44), each built and open with its leftovers**

- Slice 1, web viewer: `casual-slide-wasm` and `webapp/`, OpenCalc's shell cloned for PowerPoint (`VS-01`).
- Slice 3, desktop: Tauri shell, bundled on three OSs (`VS-03`).
- Slice 4, save and export: byte-identical unedited save, PDF, slide as PNG (`VS-04`, done).
- Slice 5, edit text: `casual-slide-transaction` from OpenCalc, caret, typing, IME, Bold/Italic/Underline, undo (`VS-05`, `VS-17`, `VS-19`).
- Slice 6, move and resize: select, drag, resize, rotate, nudge, delete (`VS-06`).
- Slice 7, present: slide show and Presenter View (`VS-07`).
- Slice 8, build slides: New Slide, Duplicate, Delete, Hide, reorder, insert text box and picture; starts on a blank deck (`VS-08`).
- Slice 9, collaborate: the collaboration seam (ADR-020), `server/casual-slide-collab-server`, and co-editing in the browser with presence (`VS-09`, `COL-01`, `COL-04`, `COL-06`).
- Slice 10, animate: Transitions and Animations tabs, Animation Pane, played in the show (`VS-10`).
- Slice 11, rich objects: insert and edit tables and charts, Edit Data (`VS-11`).
- Slice 12, design: themes, variants, slide size, background (`VS-12`).
- Slice 2, render depth, continues (`VS-02`).

**Added: the editor's shell and objects**

- PowerPoint for the web's ribbon, menus, Format Shape pane, status bar, KeyTips, Home tab formatting, clipboard, multi-select, Picture Format tab (`UX-003`, `UX-004`).
- Shapes gallery with all 187 presets, drawing, yellow handles, Shape Format tab (`SHAPE-01`).

**Added: site and fidelity measurement**

- Marketing site and editor on GitHub Pages at https://casualoffice.github.io/openpresent/ (`SITE-01`, `SITE-03`).
- Slide gallery beside LibreOffice, scored per slide: at `1ba6443`, 5.4% of pixels differ on mean (94.6% agree with LibreOffice 26.2.6.3), not a PowerPoint measure (`FH-01`, `FH-02`).

**Fixed**

- CI failures on `main`, `CI-10` to `CI-22`.

### 2026-11-07

**Added: closing Tiers 1E and 1F (`CLOSE-01`, `T1E-06`, `T1F-04`, `STYLE-01`, `DRAW-01`)**

- A text body's `prstTxWarp` with `avLst`, `scene3d`, `sp3d` and `flatTx`, and `p:style` on shapes, connectors and pictures, modelled, read and written. Connector ends can be edited. `DrawingLimits` is carried by `SlideLimits`. Older `a14` namespace read. Colour findings on slide parts keep `CS-CLR`.
- The text chain takes the theme's `txDef`, `lnDef` and `spDef`, and a shape's `fontRef`.
- The writer refuses a new `ST_PositiveCoordinate` over 2147483647 and a new dash stop of zero or less, as `CS-EXP-0002`.
- Fixtures: gradient and pattern fills on placed shapes, a body's warp and 3D, object defaults, `extLst` slots, path fill modes; six hostile decks; a silent-drop gate.
- Status marks for GEOM, FILL, LINE, EFFECT, D3 and TEXT, by `tools/spec-audit`, which now sees enum values read through their enum, `Node::a` chains, `// spec-audit: owner=` helpers and model doc comments, and walks tests that read the checklist by area (`STAT-01`).

**Added: text (`T1F-01`, `T1F-02`, `T1F-03`)**

- Text bodies on shapes, `defaultTextStyle`, master text styles and `notesStyle`, modelled, imported and patched in place: every run and paragraph property, spacing, tabs, bullets, all 41 autonumber schemes, fonts as stated, hyperlinks on runs. `normAutofit`, `spAutoFit` and `noAutofit` stored as written.
- `casual-slide-style` resolves text properties through run, paragraph, shape list style, layout placeholder, master placeholder, master text styles and presentation default.
- `TextLimits`, `CS-TXT-0001` to `0005`, `CS-EXP-0009`, and the `text_bodies` fuzz target.

**Added**

- Governance: `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `GOVERNANCE.md`,
  this changelog, and `NOTICE` with the Ecma notice and a third-party inventory (`GOV-01`).
- [docs/15](docs/15-CI-AND-RELEASE-GATES.md), CI and release gates, and
  [docs/21](docs/21-PARSER-LIMITS.md), the parser limits contract (`GOV-01`, `DOC-008`).
- Repository gates ported from OpenCalc, each with a seeded violation in a self-test:
  tracker row shape, doc references, crates never depending on a server (`CI-01`),
  fuzz lockfile, Conventional Commits. Post-merge and scheduled fuzz workflows (`GOV-01`).
- Microsoft's Open XML SDK validator in `tools/ooxml-sdk` (`F-006`).
- Real PowerPoint-written decks with recorded rights in `fixtures/real/`, and a runner over
  66 decks in `tools/real-decks` (`F-007`).
- Text cost and painting measurements (`SPK-01`); ADR-004 and ADR-005 accepted.
- Markup compatibility, part-name equivalence, the Strict namespace map, F4 schema
  validation and forced rewrite through the tree (`MCE-01`, `OPC-01`, `STRICT-01`,
  `F4-01`, `WR-01`).
- Document properties, the presentation part and view state: model, import, export
  (Tier 1B).
- Colour and theme: model, import, export, resolver (Tier 1C).
- Preset and custom geometry engine with all 187 shapes and 41 text warps (`GEO-03`).
- Spec audit, count check, fidelity pipeline and area decks (`SPEC-01`, `SPEC-03`,
  `SPK-02`).
- Part graph, hostile-input corpus, fuzz targets and CI (`F-003`, `F-004`, `F-005`).
- Byte-faithful XML tree (ADR-013).
- Foundation: design set, spec checklist of 8730 rows, and the package crate forked
  from OpenCalc with provenance (`DOC-001`, `SPEC-05`, `F-002`).
- Design documents: phases and tasks, owner instruction log, PowerPoint command
  inventory, UX study, competitor study, fit-gap, export and save design.

**Fixed**

- Defects in Ecma's preset shape definitions (`GEO-01`, `GEO-02`).
- Zip bomb dilution and XML read before the size check (`SEC-013`).

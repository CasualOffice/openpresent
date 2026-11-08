# 42 — Text, rendering and slide show

## Text engine

The pipeline, all in the engine:

```
runs → font resolution and fallback → itemize (bidi, script) → shape
     → line break → line boxes (spacing, tabs, bullets, columns) → autofit → glyph runs
```

The same result supplies drawing, caret positions, selection rectangles and hit testing, so
what is clicked is what is drawn.

**Built so far** (2026-11-08, `casual-slide-text`): font resolution and fallback, shaping, UAX #14 line breaking (a break ending a paragraph opens an empty last line, sized by the paragraph mark), line boxes with spacing and bullets (a line's ascent and descent come from each run's own substituted face, never a fallback that draws a character it lacks, as OpenCalc's `FaceSelector::primary`), and glyph runs. Caret geometry from `VS-05`: each laid-out line gives its top, bottom and every caret stop as a paragraph offset and an x (`CaretLine`), and `casual-slide-wasm` hit-tests and draws carets and selections from them, so the editor and the drawing share one layout. Underlines from `VS-18`: a glyph run carries its `u` kind and a line at the face's `post` underline position and thickness, trailing spaces excluded, which the scene strokes in the run's colour (`dbl` two lines, `heavy` twice as thick, dotted and dashed kinds as named; `words` and wavy kinds are reported as approximations). Not yet: bidi across runs, tabs, columns, autofit, and carets in vertical, rotated-body or right-to-left text (`VS-15`). Each is a row in [14](14-EXECUTION-TRACKER.md) under Tier 2D.

**Why this reverses OpenCalc's ADR-018.** That ADR kept shaping out of WASM because the browser
already draws complex scripts correctly and the editor never needs engine metrics to place a
caret. Here, the browser would give each collaborator a different wrap, and the export a third.
The cost is real, and `SPK-01` measures it before ADR-004 is Accepted.

**Fonts.** OpenCalc already bundles Carlito (Calibri metrics), Caladea (Cambria metrics),
Liberation (Arial, Times New Roman, Courier New metrics), Inter for UI and sliced CJK faces.
Metric-compatible substitution keeps line breaks true to PowerPoint for those families.
Others are the honest gap: **Aptos**, Office's current default, is the largest risk: modern decks reflow without a compatible face. Akasia (OFL) claims metric compatibility with it, which is self-reported and unverified (`TXT-03`). Options, in order: the host supplies the
real font through a font seam; a tuned substitute with stored advance overrides; report the
substitution in the compatibility report. Embedded fonts in a `.pptx` are honoured only if their
embedding rights allow it.

**Autofit.** `normAutofit` stores `fontScale` and `lnSpcReduction`. On an unedited body we use
the stored values, so a deck looks as saved. On edit we recompute. Matching PowerPoint's own
search exactly is not assumed; fixtures will show how close it gets.

**Input.** The canvas does not take text. A focused, off-screen `contenteditable` receives
`beforeinput` and composition events; the engine applies them as ops and paints the composition
underline itself. Candidate windows are positioned by moving that element to the caret. Find,
spell check and the screen-reader tree use a parallel accessible DOM of the current slide.
**Spell check** is a host seam, since the browser cannot check text in a canvas.

## Display list

As OpenCalc: serializable, painter's order, nested clips, and golden-testable. Its items are
different, because this is general vector art rather than grid decoration.

| Item | Carries |
| --- | --- |
| `Path` | Geometry, fill (solid, gradient, pattern as an 8 by 8 cell bitmap, picture stretched or tiled), stroke (dash, cap, join, arrowheads). A shape in a group is placed in slide units: the group's scale sizes its box, never its line widths or text (`FH-03`) |
| `GlyphRun` | Face, size, glyph ids and positions, fill, the characters drawn (so the PDF backend keeps text extractable, `VS-04`), and the source range for hit testing |
| `Image` | Frame, crop, recolour, part hash |
| `Group` | Transform, opacity, clip, effect list, stable object id, for caching |
| `Effect` | Shadow, glow, soft edge, reflection, blur |

Items carry object ids so the editor can invalidate and reuse them per object.

## Painting

| Backend | Use | Text |
| --- | --- | --- |
| Canvas2D (JS painter) | Editor, thumbnails, slide show | `Path2D` per glyph, cached by face and glyph id, positioned by transform |
| `-render` tiny-skia | Server thumbnails, PNG, the fidelity gate | Same outlines |
| `-render` PDF | Export, print | Embedded subset fonts, real text |

A stroke is never drawn thinner than one device pixel, as PowerPoint draws a 0.5 pt theme line
as a solid pixel at low zoom; a line end is sized in line widths from at least 2 pt, so a thin
line's arrowhead stays readable (`SHAPE-02`; an estimate, not measured against PowerPoint).

Glyph paths rather than `fillText` because `fillText` re-shapes and can disagree with the engine.
The cost is losing LCD antialiasing and hinting, which matter little at slide scale and not at all
in a rotated title, but do matter in small text at low zoom. `SPK-01` looks at this directly, on
both frame time and appearance. Measured in [64](64-SPK-01-TEXT-COST.md); ADR-005 is Accepted. **The fallback, if needed, is `fillText` per word** at engine positions, with `fontKerning='normal'`, `textRendering='geometricPrecision'` and `direction` from the engine. Per cluster draws wrong Arabic forms.

WebGL is held back for transitions that need it. Canvas2D first; effects that it cannot draw
cheaply (soft edge, reflection) rasterize once per object into an `OffscreenCanvas` and are reused.

**Virtualization.** The axis here is slides, not cells. Parsing is lazy per slide part; the thumbnail
rail paints what is visible from cached bitmaps; only the current slide and its neighbours keep display
lists. Images decode at the size they display, in a worker, with a bounded cache.

**Layering.** The editing surface composites three canvases: static content, the object being
manipulated, and handles and guides. Drag then repaints only what moves.

## Slide show

`p:timing` compiles to a timeline: a pure function from (click count, elapsed time) to per-object
overrides of transform, opacity, visibility, colour and motion path. The clock is supplied, so a
test samples any instant and compares it with a golden. Time never enters the model.

Each animated object is a cached layer composited per frame, so the frame cost is a function of
the number of moving objects and not the slide's complexity. Transitions composite two rendered
slides. Morph needs object matching between slides and comes last.

Presenter view is a second window on the same session, synchronized by `BroadcastChannel`, with the
notes, next slide, timer and pointer. Fullscreen and window placement use the
Window Management API where the browser has it, falling back to a manually moved window.

## Budgets

**Proposed, none measured, except the Phase 1 row marked measured.** Phase 2's render targets (time, memory, wasm size, determinism) are R5 to R8 in [06](06-ROADMAP-AND-DELIVERY.md) §Phase 2 plan. The spikes replace these with evidence, and CI then asserts them on a named
environment, as OpenCalc's docs/30 does.

| Measure | Target |
| --- | --- |
| Drag frame on a 200-shape slide | under 16 ms |
| Keystroke to paint, 2,000-character body | under 16 ms |
| Slide switch, cached | under 50 ms |
| First paint of a 500-slide deck | under 1 s after bytes arrive |
| Open and read every slide of a 500-slide deck; unedited save; one slide edit, unoptimized build (Phase 1, asserted) | under 10 s, 100 ms and 10 s. **Measured** 2026-11-08 on a 10-core laptop at load 6 to 9, so upper bounds: 0.5 to 0.7 s, under 1 ms, 1.7 to 2.4 s (`tests/phase1_exit.rs`, `tools/phase1-evidence.json`) |
| Animation, 30 simultaneous objects | 60 fps |
| Text stack in WASM, before fonts, Brotli | 313 KiB measured as an increment over a bare bundle ([64](64-SPK-01-TEXT-COST.md)); whole-engine budget set in Phase 3 |

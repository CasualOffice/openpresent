# 42 — Text, rendering and slide show

## Text engine

The pipeline, all in the engine:

```
runs → font resolution and fallback → itemize (bidi, script) → shape
     → line break → line boxes (spacing, tabs, bullets, columns) → autofit → glyph runs
```

The same result supplies drawing, caret positions, selection rectangles and hit testing, so
what is clicked is what is drawn.

**Why this reverses OpenCalc's ADR-018.** That ADR kept shaping out of WASM because the browser
already draws complex scripts correctly and the editor never needs engine metrics to place a
caret. Here, the browser would give each collaborator a different wrap, and the export a third.
The cost is real, and `SPK-01` measures it before ADR-004 is Accepted.

**Fonts.** OpenCalc already bundles Carlito (Calibri metrics), Caladea (Cambria metrics),
Liberation (Arial, Times New Roman, Courier New metrics), Inter for UI and sliced CJK faces.
Metric-compatible substitution keeps line breaks true to PowerPoint for those families.
Others are the honest gap: **Aptos**, Office's current default, has no metric-compatible open
face that we know of, so modern decks will reflow. Options, in order: the host supplies the
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
| `Path` | Geometry, fill (solid, gradient, pattern, picture), stroke (dash, cap, join, arrowheads) |
| `GlyphRun` | Face, size, glyph ids and positions, fill, and the source range for hit testing |
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

Glyph paths rather than `fillText` because `fillText` re-shapes and can disagree with the engine.
The cost is losing LCD antialiasing and hinting, which matter little at slide scale and not at all
in a rotated title, but do matter in small text at low zoom. `SPK-01` looks at this directly, on
both frame time and appearance. **If it looks wrong, ADR-005 changes**, to `fillText` per cluster
at engine-given positions, keeping engine layout.

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

**Proposed, none measured.** The spikes replace these with evidence, and CI then asserts them on a named
environment, as OpenCalc's docs/30 does.

| Measure | Target |
| --- | --- |
| Drag frame on a 200-shape slide | under 16 ms |
| Keystroke to paint, 2,000-character body | under 16 ms |
| Slide switch, cached | under 50 ms |
| First paint of a 500-slide deck | under 1 s after bytes arrive |
| Animation, 30 simultaneous objects | 60 fps |
| WASM bundle before fonts, Brotli | decided by `SPK-01` |

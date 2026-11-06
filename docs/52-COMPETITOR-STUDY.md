# 52 — Competitor study: who has tried to clone PowerPoint

What others found when they tried this, and what it means for
[02](02-ARCHITECTURE.md), [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md), [24](24-OPERATIONS-AND-COLLABORATION.md),
[28](28-PACKAGE-AND-PART-GRAPH.md) and [51](51-SPEC-CHECKLIST-AND-FIDELITY.md).

## How to read this

**Every source was checked on 2026-11-07.** Citations such as `[S12]` point to the [source register](#source-register),
which gives the URL, the page's own date and how it was read.

| Mark | Meaning |
| --- | --- |
| **D** Documented | The vendor, maintainer, specification or a named third party states it. Vendor claims stay claims, and third-party reports are labelled as such |
| **O** Observed | Seen directly in this study: a repository tree, a source identifier, a downloaded file, or a rendered DOM from a public page |
| **I** Inferred | My reasoning from D or O. It can be wrong, and nothing here has measured it |

**Limits of this study.** Read these before relying on any table.

- **No PowerPoint comparison was run.** Nothing here measures fidelity. Every fidelity statement is a report by someone else.
- **LibreOffice Bugzilla, the LibreOffice wiki release notes and the Collabora SDK site refused automated access** (Anubis bot check). I did not try to get around that. Bug statements below come from search-result excerpts, marked "excerpt only": their current status is unknown and many are years old.
- **Google Slides was observed once**: one public deck, signed out, in view-only mode, in headless Chrome. Editing mode was not observed.
- **The Univer Slides demo did not finish loading** within 60 s in headless Chrome ([O]; it stayed on "Loading Univer Slides"), so its rendering is taken from its documentation.
- **WPS is close to a blank.** No technical source was found on its rendering, text layout or collaboration model. Those cells say "not found" and were not filled in.
- **Zoho's rendering source dates from 2011.** Its current state is unknown.
- **Source code was inspected for structure only**: file names, identifiers and comments, by `grep`. No algorithm in ONLYOFFICE or LibreOffice source was studied, and none is described here. See [Licences](#licences-and-clean-room-practice).

## The short version

1. **Engine-owned layout is the common choice.** Every product aiming at PowerPoint fidelity whose text path could be examined keeps line breaking out of the browser's hands. That supports ADR-004. None of them shows that it must be in WASM.
2. **None of the four editors whose text path could be established paints glyph paths interactively.** ONLYOFFICE paints FreeType bitmaps, Google Slides uses SVG `<text>` at engine-given positions, Collabora sends bitmap tiles, PPTist uses DOM text. This is the strongest challenge in the study: `SPK-01` should test a third arm, not only the `fillText` fallback.
3. **No precedent was found for a byte-faithful XML tree.** The evidence for why it is wanted is strong; the evidence that it works at scale does not exist. LibreOffice has only now started doing a narrow version, for charts.
4. **Character OT has one documented precedent for text, and a working competitor without it.** ONLYOFFICE ships presentation co-editing with object-level marking and gives up redo. Whether users would pay for more is unmeasured.
5. **`docs/42` may be out of date on Aptos.** A project called Akasia, first released in June 2026, claims metric compatibility under OFL. The claim is self-reported and unverified.

## Per product

### Summary

| | Rendering | Text layout | Ribbon | Collaboration | Licence |
| --- | --- | --- | --- | --- | --- |
| **ONLYOFFICE** | Canvas, own painting | Own, in JS, with HarfBuzz and FreeType in WASM | Yes, tabs | Change sets plus object locks; Fast and Strict modes | AGPL-3.0 with additional terms |
| **Collabora Online** | Server-rendered tiles on a browser canvas; WebGL slideshow | LibreOffice core on the server | Yes, Notebookbar | One server process per document | Primarily MPL-2.0 |
| **LibreOffice Impress** | Native desktop | Own, with HarfBuzz | Optional; six layouts | Not researched for desktop | MPL-2.0 |
| **Google Slides** | SVG in the DOM | Engine positions, browser draws glyphs | No; menu bar and toolbar | Not documented for Slides; Docs uses OT | Proprietary |
| **Zoho Show** | SVG, VML, Canvas (2011 source) | Own JS engine "Petal" (2011 source) | No; contextual toolbars | Real time; model not found | Proprietary (I) |
| **WPS Presentation** | Not found | Not found | Not found | "Real time" claim only | Proprietary, free tier |
| **PPTist** | DOM plus SVG | Browser, ProseMirror | No | None found | AGPL-3.0 or paid licence |
| **Univer Slides** | Canvas2D | Own, using browser `measureText` | Not checked | Commercial server SDK | Engine Apache-2.0; Slides commercial |
| **reveal.js** | DOM | Browser | Not an editor | None | MIT |

### ONLYOFFICE Presentation Editor

**Rendering: canvas, for fidelity reasons stated by the vendor.**
- D: the team decided in 2010 to "render it themselves" because "plain HTML simply could not deliver the formatting precision and cross-browser consistency the team required" [S1].
- D: the vendor is candid about the price: "HTML5 Canvas was not designed for document editing", and "Canvas-based rendering is computationally intensive", needing years of optimization [S1].
- O: slide logic lives in `sdkjs/slide`, with `Drawing/HtmlPage.js`, `Transitions.js`, `TransitionsGL.js`, `MorphTransition.js`, `anim-pane.js` [S2]. Transitions have both a Canvas2D and a GL file (I, from the names).

**Text: own layout, with shaping in WASM.**
- O: `common/libfont/engine` contains `fonts.wasm` at 3,614,021 bytes uncompressed, plus a `fonts_ie.js` fallback of 6.9 MB [S3]. `textshaper.js` calls `HB_*` functions and `manager.js` uses `FT_*` functions [S3]. `core/3DPARTY.md` lists HarfBuzz (MIT) and FreeType (FTL) [S4].
- O: `word/Drawing/Graphics.js` draws text by loading glyphs from the font manager and reading `pGlyph.oBitmap`; `common/libfont/rasterheap.js` holds a raster heap [S5]. I: glyphs are rasterized by the engine and blitted, not drawn with `fillText` or paths. A grep for `fillText` in that file found none; other draw paths exist in the suite and were not audited.
- D: users get a font-hinting setting with three values: "As Windows", "As OS X" (no hinting) and "Native" [S6]. I: small-text appearance mattered enough to expose.
- O: `common/Drawings/Format/DrawingContent.js` extends the word processor's `CDocumentContent` [S5]. I: slide text shares the word processor's layout engine, so one investment serves three products.
- O: `Shape.js` carries `FontScale`, `LnSpcReduction` and a text-fit delta [S5]. I: stored autofit values are honoured and recomputed. The exact logic was deliberately not studied.
- O: input goes through `common/text_input2.js`, which has both textarea and `contenteditable` element types, with a comment that a single textarea is not enough [S5]. This is the same shape as the plan in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md).

**PPTX import: conversion to a private binary.**
- O: `core/X2tConverter` converts formats and `core/OOXML/Binary/Presentation` holds `Converter`, `PPTXWriter`, `FontCutter`, `FontPicker` [S7]. `slide/api.js` uses `BinaryPPTYLoader` and writes a `PPTY;v10;` header [S5]. `OOXML/PPTXFormat` is a typed C++ object model [S7]; `Slide.h` and `CSld.h` have typed nullable members and no match for `extLst` or `unknown` [S7]. I: a load and save passes through typed models and a binary format, so preservation depends on what those models hold. Whether other classes retain extensions was not audited.
- D: until 2013 it used proprietary formats, then "committed to OOXML" [S1]. I: the OOXML model was fitted to an existing engine rather than designed around it.

**Fidelity and feature coverage.**
- D: animation support took about four years: requested July 2018, "very complicated" in November 2021, "most of animation" in v7.1, May 2022 [S8].
- D: changelog milestones [S9]: SmartArt without conversion to groups (7.1.0), Morph (7.5.0), "Own rendering of some SmartArt objects instead of recorded images in the file" (8.2.0), viewing animations with text (9.0.0), 20 new transitions (9.4.0), Master Slide settings moved to their own tab (9.1.0; a Slide Master mode already appears in 8.1.1 entries), Outline, Notes and Handout views and animated or video export (10.0.0).
- D: re-save and animation bugs keep appearing in the changelog: "error when saving/opening the PPTX file with animation applied" (8.3.3), editor stopping on some animations (9.0.3), crashes converting some PPTX files (9.0.4) [S9]. Third party: a forum user reported that 50% opacity was lost on save in v8, confirmed by the team [S11].
- I: **sixteen years after the 2010 decision, the changelog is still adding views such as Outline (10.0).** Feature parity is a long road, even for a project that has worked on it continuously.

**UI: ribbon-style tabs.** O: `web-apps/apps/presentationeditor/main/app/view` has `Toolbar.js`, `Animation.js`, `Transitions.js`, `SlideMasterTab.js`, `ViewTab.js` [S12]. D: transition settings moved to a toolbar tab in 7.0 [S10].

**Collaboration: locks and change sets, not character OT as far as shown.**
- D: Fast mode shows changes live and is the default; Strict hides them until saved. Edited objects get dashed outlines; in Strict, green for yours, red for others [S13]. D: in Fast mode undo covers only your own change and **redo is unavailable because it "may cause conflicts"** [S14].
- O: `slide/Editor/CollaborativeEditing.js` has locks by `c_oAscLockTypeElemPresentation` `Object` and `Slide`; `common/docscoapi.js` has `askLock`, `saveChanges` and `changesIndex` [S5].
- I: the unit of conflict in a presentation is mainly the object or slide. How two users typing in one text box merge in Fast mode was not established.

**Licence.** D, O: AGPL-3.0 with additional terms; no trademark rights; non-code elements including icons under CC BY-SA 4.0 [S5][S15].

### LibreOffice Impress

**Rendering and text.** Native desktop. O: `vcl/source/gdi/CommonSalLayout.cxx` has HarfBuzz calls [S16]. `oox` maps `normAutofit` values to `TextFitToSizeFontScale` properties [S16]. I: PPTX is converted on import into LibreOffice's own drawing model.

**PPTX strengths.**
- D: PPTX is read and written; 26.8 adds round-trip retention of chart types it cannot display, described for OOXML files, with PPTX not named in that post [S17][S18]. D: 26.8 adds slide sections and multiple page sizes in one document [S17].
- O: `sd/qa/unit` has `export-tests-ooxml1` to `5`, `import-tests-smartart` and others [S16]. I: a large regression suite is one of its real assets.

**Documented gaps.**
- **SmartArt.** D, 2019: "PowerPoint 2010+ writes a DrawingML fallback" which Impress shows without doing layout; legacy files with no fallback depend on partial layout support; "If editing is ever wanted, own layout support is needed!" [S19]. Excerpt only: line breaks in SmartArt boxes differ from PowerPoint (tdf#148828) [S20].
- **Autofit.** Excerpt only, status unknown: font 55.7 against PowerPoint's 54 with a word split (tdf#122023); text still overflowing after shrink (tdf#157773); the two "interpret shrink on overflow differently", with PowerPoint adding an empty line (tdf#136228); the shrink option not recognized on import (tdf#107694) [S20].
- **Theme colours.** D, 2022: in Impress, chose to precompute colours on theme change rather than store references, and fills and lines were not yet covered [S21]. I: this trades away what ADR-006 keeps. Later releases may have changed it.
- **Round-trip validity.** D: 26.2.2's listing has `PPT->PPTX invalid value in a:ext`, `invalid values in a:off element`, `OLE objects on different slides get overwritten on save`, `Paragraph styles ... saved as direct formatting`, `Text appears right aligned instead of centered`, and `elements with 3D rotations don't render any longer` [S22]. Third party: "Several bugs that caused Microsoft Office to reject round-tripped documents have also been fixed" (excerpt only) [S23].
- **Malformed input.** D: 26.8.1's listing has crashes on `p:pos` with no preceding `p:cm`, and on an `a:tableStyleId` GUID not defined in `tableStyles.xml` [S24].
- **Master export.** Excerpt only: one master with two layouts saved as two masters [S25].
- **Chart loss.** D: regional map charts keep the type but "very little of its content remains intact" [S18].

**UI.** D: six layouts including Tabbed, Tabbed Compact and Groupedbar. "Choice is the point"; classic stays first-class; Tabbed is closest to the ribbon [S26].

**Licence.** O: core ships `COPYING.MPL`, Mozilla Public License 2.0 [S16]. GitHub's metadata labels the repo GPL-3.0 [S16]; I take MPL-2.0 from the file the project ships and from this study's brief; legal should confirm per file.

### Collabora Online

**Rendering: LibreOffice on the server, tiles in the browser.**
- D: one Kit process per document [S27 excerpt]; 2016 flow is UI requests tile, LOKit renders and compresses to PNG, sent back [S28 excerpt]; "multiple collaborators in a document access this one process" with 256×256 tiles [S29 excerpt].
- D, 2025 vendor deck: client keeps 150 to 250 tiles as canvases (30 to 60 MB); "JS 'GC' is not your friend"; a watchdog warns if the main loop exceeds 50 ms [S30].
- **Slideshow: moved from animated SVG to WebGL.** D, 2025: "Up to 16x" faster load for large decks, with no method given; 4K, 3D transitions; pre-fetching in the direction of travel starting 100 ms after a slide switch [S30]. O: `browser/src/slideshow` has `LayersCompositor.ts`, `LayerRenderer.ts`, `Transition2d.ts`, `Transition3d.ts`, `SlideCache.ts`, and `SlideRenderer.ts` uses `WebGLTexture` [S31].
- I: Collabora's fidelity is exactly LibreOffice's, and every gap above applies. Layout is identical for all collaborators because there is one layout.

**Collaboration.** D: core is used for "import, export, modification, rendering and collaboration" [S32]. I: the server's single document gives one order of events and no client-side merge. Not stated explicitly by any source found.

**Interoperability work.** D: validity-testing about 243,000 converted files, aiming at zero errors, with a progress chart [S33]. D: 26.04 (June 2026) adds Follow-Me slideshow, slide sections, better font embedding [S33].

**UI.** D: Notebookbar tabs, shared with LibreOffice through JSDialog [S33][S34 excerpt]. Default mode not established.

**Licence.** D: "primarily under the MPLv2", other parts under other licences; branding Apache-2.0 [S35].

### Google Slides

**Rendering: SVG with engine-positioned text.**
- O: the rendered DOM of a public deck in view-only mode has **no `<canvas>`, 15 `<svg>` elements (some are icons) and 63 `<text>` elements** [S36]. Slide content sits in an `<svg>` under groups with ids such as `editor-<objectId>-paragraph-0`, with `sketchy-text-content-text` groups holding the `<text>` nodes, and filmstrip thumbnails are also SVG [S36].
- O: each text run or word is its own `<text>` with `text-rendering="geometricPrecision"` and an unrounded position, for example `x="68.15942974541133"` for `{Name` and `x="317.97192974541133"` for `Here}` [S36].
- I: **Google computes layout itself, down to word position, and lets the browser draw glyphs at those positions.** This is the same shape as the fallback named in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md), `fillText` per cluster at engine positions, except by word and in SVG.
- O: `/export/svg` for the same deck contains only `path`, `image`, `clipPath`, `g`: **text as glyph outlines, no `<text>`**, for the one slide returned [S37]. So glyph paths exist in Google's portable output, not in its editor.
- D: Google moved **Docs**, not Slides, to canvas in 2021, for "performance" and "consistency in how content appears across different platforms"; the post does not mention Slides [S38]. Third party: an accessibility "side DOM" accompanied it [S39 excerpt].

**Autofit.** D: the API exposes the same pair as OOXML: `fontScale` and `lineSpacingReduction`, multiplied at render [S40 excerpt]. D: "Any text Autofit settings on the shape are automatically deactivated by requests that can impact how text fits in the shape" [S41]. I: Google avoids promising recomputation through its API.

**PPTX limits.**
- D: "Up to 100 MB for presentations converted to Google Slides" [S42]. Third party: a 300 MB increase announced then paused (user report, unverified) [S43 excerpt].
- D: Office editing writes "to the original Microsoft Office file" [S44]. Google's guide lists no lost features [S44].
- Third party, **dated May 2021**, "true at time of testing" [S45]: missing fonts become Arial, permanently; text transparency lost; WordArt becomes plain text; animation delays dropped; timings rounded to one decimal; motion paths and Morph lost; footers unsupported; most shape effects lost except shadow and reflection; pattern fills become solid; a 3 pt line became 4 pt; SmartArt animation lost; live charts become non-editable images; triggers lost; no table cell split.

**UI.** O: `docs-menubar` and a simple toolbar, no ribbon [S36]. **Collaboration.** D: Docs uses OT with a separate collaboration protocol (2010) [S46 excerpt]. Slides is not documented in anything found. **Licence.** I: proprietary hosted service; [S44] mentions a Workspace account and trial.

### Zoho Show

- **Rendering and text.** D, **September 2011**: "Rendering engines include VecDer (SVG, VML and Canvas) and Petal (JavaScript-based text and layout engine)" [S47]. I: Zoho also concluded fifteen years ago that it needed its own layout engine. Current state unknown.
- **Best compatibility statement of any vendor found.** D: Zoho publishes a per-feature list [S48]:
  - Not supported: WordArt, math equations, narration, transition sound, 3D format and rotation, glow, soft edges, artistic filters, stock and surface charts, trendlines, error bars, letter spacing, **bidirectional text, column text, vertical text**.
  - Partial: SmartArt "Converted as Group Shapes"; gradients linear and radial only; gradient lines "Imported as Solid colors"; radar and 3D charts as images; missing fonts "fall back to default font".
  - Supported: masters and layouts; pattern fills; entrance, emphasis, exit and path animations; triggers; text animation by word and letter; chart animation by series.
- **Limits.** D: imports up to 500 MB, including password-protected PPTX [S49].
- **Vendor claim.** D: "Charts import as fully editable elements, not screenshots" [S50]; no limitations admitted in that piece.
- **UI.** D: toolbars change with selection, "built around the content" [S50]. **Collaboration.** D: co-editing available [S50]; 2011 mentions a "web messaging framework" [S47]; model not found. **Licence.** I: proprietary SaaS; no public source repository was found.

### WPS Presentation

Found: a free basic tier with paid plans, and a real-time co-editing claim covering Presentation [S51 excerpt]. Kingsoft Docs is described as HTML5 and relying on a "self-developed Office kernel" (annual-report summary, third party) [S52 excerpt]. A University of Bergen help page calls it "99 % compatible" with no method [S53 excerpt], and G2 reviewers report compatibility and font problems [S54 excerpt].

**Not found:** rendering approach, text layout, documented PPTX gaps, UI, collaboration model. Nothing here supports any conclusion about WPS's architecture.

### PPTist

- **Rendering: DOM plus SVG.** O: a public demo DOM has **0 `<canvas>`, 56 `<svg>`, ProseMirror editors and `base-element` nodes** [S55]; `BaseShapeElement.vue` uses `<svg>` [S55]; `package.json` depends on ProseMirror, `html-to-image` and ECharts [S55].
- **Text: browser.** I, from the above: wrapping belongs to the browser, so it can differ between clients.
- **Fidelity, by its own author.** D: import "~85%+", export "~95%+", "100% fidelity cannot be guaranteed"; **"I hope you use PPTist to build a presentation product different from Office PPT, rather than just an editing relay station for Office PPT"** [S56]. I: the most popular DOM attempt (9,467 stars, O) tells you not to use it for what OpenSlide is doing. The percentages have no stated method.
- **PPTX.** O: uses `pptxtojson` for import and `pptxgenjs` for export [S55]. So both directions rebuild from JSON, and retention is not attempted (I).
- **UI.** O: top menu, left and right panels; no ribbon [S55]. **Collaboration.** O: README has no mention [S55]. **Licence.** D: **AGPL-3.0, "prohibits closed-source commercial use"**, or a paid independent licence [S56]. `pptxtojson` itself is MIT (O) [S55].

### Univer

- **Rendering.** D: "implemented based on Canvas2D", inspired by Konva, Fabric and Babylon [S57]; a Skeleton layer holds calculated layout and does coordinate conversion [S57]; slides are a declared unit model [S57].
- **Text.** O: `engine-render/.../shaping-engine/font-cache.ts` uses `getMeasureText` and `getTextSizeByDom`, with `hdmxData` in its types [S58]; the package has hyphenation pattern loaders [S58]. I: own line layout on browser metrics. That is a middle option: no shaping bundle, but metrics can vary by browser and OS. Draw calls were not checked.
- **Slide status.** D: "current Slides and Boards capabilities are separately distributed commercial products"; PPTX exchange too [S59]. O: `packages/slides` and `slides-ui` exist in the open repository [S59].
- **An idea worth noting.** D: "Slide layout lint adds structured findings for issues such as off-page or overlapping text" for agents [S59].
- **Licence.** D: engine Apache-2.0; commercial plugins, Server SDK and collaboration separately licensed [S59].

### reveal.js and similar

D: an HTML presentation framework, MIT [S60]. It is an authoring-in-code format with a separate hosted editor, and it makes no attempt at PPTX fidelity (I). It is a useful bound only: when the goal is not PowerPoint, the DOM is enough.

### Aspose.Slides and Open XML SDK: what is hard

**Aspose.Slides** is commercial (I, from metered licensing links [S61]), a "standalone engine" not needing PowerPoint [S61].
- D: "rendering accuracy may vary slightly compared to PowerPoint, especially when using custom or missing fonts" [S62 excerpt].
- D: separates **replacement** (deliberate), **substitution** (when unavailable) and **fallback** (per missing glyph); fallback rules are runtime settings, not stored in the file [S63 excerpt].
- D: substituting a font "can change text width, height, and wrapping" [S64].
- D: release notes and tickets show the same long tail: "autofit in Aspose.Slides looks different than in PowerPoint" (2017) [S65 excerpt]; rotated SmartArt parts cropped, fixed in 25.5 [S66 excerpt]. A commercial engine sold for this purpose still files these.

**Open XML SDK and Microsoft specifications.**
- D: SDK preprocessing "Preprocessing affects what gets saved. When you save a file, the only markup that is saved is that which remains after preprocessing"; the default `NoProcess` does none [S67].
- D: `normAutofit` omitted means no autofit in the standard, but **"Office does not follow this and instead uses a hierarchy of text styles"** [S68]. A schema-driven implementation gets this wrong.
- D, third party: "PowerPoint always reduces line spacing by as much as 20% to maximize the font size used"; later edits "re-fit the text automatically, up to the original (full-size) font size"; the algorithm is undocumented, so re-fit can make text jump [S69].
- D, third party: repair prompts are "generic and do not pinpoint the exact line, element, or file" [S70].

## Feature coverage against PowerPoint

Only what sources state. A blank means not established, **not** "absent".

| Feature | ONLYOFFICE | LibreOffice / Collabora | Google Slides (2021 report) | Zoho Show | PPTist |
| --- | --- | --- | --- | --- | --- |
| Masters and layouts | Yes; own tab in 9.1.0 [S9] | Yes [S21] | Footers unsupported [S45] | Yes [S48] | |
| SmartArt editable | Yes, without group conversion [S9] | Fallback drawing; partial layout [S19] | | As groups [S48] | No mention |
| Morph | Yes, 7.5 [S9] | | Lost [S45] | | |
| Animation | "Most", 7.1 [S8] | | Presets only, paths lost [S45] | Broad, with paths and triggers [S48] | Yes, element [S56] |
| Charts native | `ChartEx.js` exists [S3 tree] | Newer types retained, not drawn [S18] | Become images [S45] | Most; radar and 3D as images [S48] | 8 types [S56] |
| WordArt | `TextArtSettings.js` exists [S12] | | Plain text [S45] | No [S48] | |
| Glow, soft edge, 3D | | 3D rotation issue [S22] | Lost [S45] | No [S48] | |
| Equations | `MathEquation` folder [S7] | | | No [S48] | LaTeX [S56] |
| RTL, vertical, columns | RTL work on the roadmap across editors, presentation status not established [S10]; columns yes [S10] | | | No [S48] | |
| Export video | Yes, 10.0 [S9] | | | | |

## What this means for our decisions

### ADR-004: engine-owned text layout in WASM. **Supported on engine-owned; unproven on WASM.**

For:
- O, D, I: ONLYOFFICE and Collabora own layout, Google does by inference from its DOM, and Zoho said so in 2011. Their reason is ours: "cross-browser consistency" [S1], "consistency in how content appears across different platforms" [S38].
- D: the one DOM attempt examined advises against fidelity use [S56].
- O: ONLYOFFICE ships HarfBuzz and FreeType in WASM [S3]. So a shaping WASM in an editor is known to ship at about 3.6 MB uncompressed, a useful outside reference for the `SPK-01` bundle row.

Against, or not shown:
- I: nobody examined has the whole pipeline in WASM. ONLYOFFICE has layout in JS, shaping in WASM. So keystroke-time costs across the boundary have no precedent here, and `SPK-01` must measure them.
- O: Univer shows a cheaper middle: own line breaking on browser metrics [S58]. It was not shown to be wrong, but it gives up the same-wrap guarantee (I).
- D: nobody claims to match PowerPoint's autofit search. [S69] calls the algorithm undocumented. **Owning layout does not by itself give PowerPoint's line breaks.** Fonts and undocumented rules decide that.

### ADR-005: glyph paths on canvas. **Challenged. No editor whose text path could be established does this.**

| Product | How interactive text reaches the screen | Mark |
| --- | --- | --- |
| ONLYOFFICE | FreeType bitmaps from the engine, blitted | O + I [S5] |
| Google Slides | SVG `<text>` at engine positions | O [S36] |
| Collabora | Server-rendered bitmap tiles | D [S28][S30] |
| PPTist | DOM text | O [S55] |
| Google SVG export | Glyph paths, not interactive | O [S37] |

- I: this is not proof that paths are wrong. These choices may reflect history, such as Canvas2D `Path2D` performance in 2010, and none is a measured comparison. No published comparison of `Path2D` glyphs against `fillText` was found at all.
- D: ONLYOFFICE exposes a hinting setting [S6]. That says the look of small text was a real complaint or concern (I), which is exactly what [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) worries about.
- D, old community reports: LCD antialiasing on canvas text needs an opaque context, `alpha: false` [S72 excerpt]. I: paths would not get LCD treatment either way; layered canvases with transparency also lose it for `fillText`. Three layers, as planned, interact with this.
- **Recommendation: `SPK-01` should compare three arms, not two.** (a) `Path2D` per glyph. (b) `fillText` per cluster or word at engine positions, Google's shape. (c) Engine-rasterized glyph bitmaps in an atlas, ONLYOFFICE's shape, which keeps engine control and could allow hinting. Judge on 11 pt to 14 pt text at 100% zoom on a 1× display and at thumbnail size, as well as frame time. Paths can stay for rotated and very large text, and for SVG export, whichever arm wins.
- I: arm (b) has a font-identity risk. The browser must draw with the same font file the engine shaped with, loaded through `FontFace`, and must not re-shape differently.

### ADR-007 and ADR-013: part-level retention and a byte-faithful XML tree. **Need strongly supported; method without precedent.**

For:
- D: TDF on discarding what is not understood: "The file opens, the chart disappears", "the loss occurs silently" [S18].
- D: LibreOffice's newest work adopts retention, and TDF claims it is "the only office suite to adopt this approach" [S18]. That claim is vendor marketing, and it concerns chart definitions only.
- D: Google's font change becomes permanent [S45]; PPTist and LibreOffice rebuild, with reports above of invalid XML, lost formatting and rejected files [S22][S23]. ONLYOFFICE keeps fixing re-save bugs [S9].
- D: the SDK warns that what gets dropped stays dropped [S67].
- D: `grab-bags` in LibreOffice's PPTX work, mentioned in 2022 [S21], show that retained fragments were tried. [28](28-PACKAGE-AND-PART-GRAPH.md) rejects these as a long tail of special cases. No evidence here contradicts that.

Against, or open:
- I: **no product found patches a source-preserving XML tree in place.** We would be first among those examined, so nobody can tell us how it fails.
- I: **stale derived content is the main risk.** Retaining bytes retains things that become wrong once a neighbour is edited: the SmartArt drawing part after its data changes [S19], `mc:Fallback` branches, chart caches beside an embedded workbook, stored `fontScale` after an edit [S69], `docProps/thumbnail`. Keeping them is faithful and also displays old content in other readers. Each needs a rule: rewrite, drop, or flag.
- I: retention does not stop structural invalidity. Reports of repair prompts concern values and references [S22][S70], and a tree patch can still write those. F4 and F5 in [51](51-SPEC-CHECKLIST-AND-FIDELITY.md) remain necessary.
- O: Collabora's scale, about 243,000 files [S33], sets the bar for how big a corpus finds this.

### ADR-006: inheritance never flattened. **Supported.** 

D: LibreOffice's precompute choice left fills and lines uncovered at the time [S21]; a master export bug is reported (excerpt) [S25]; ONLYOFFICE's changelog carries a run of Slide Master fixes in 8.1.1 [S9]; Google lacked footers and secondary colours in 2021 [S45]. I: masters are a repeated source of trouble for others, which favours keeping inheritance as the central fact. None of these sources links the trouble to flattening directly.

### ADR-008, ADR-009, ADR-010: server-ordered OT, per-property, character OT in text. **Supported on order and properties; demand for character OT unproven.**

For:
- D: Figma's model is close to our property rule: server tracks the latest value per property per object, "the server can define the order of events", so edits to different properties do not conflict [S73]. Its reason for skipping OT is that it is "a design tool, not a text editor", and same-text edits do not merge there [S73]. We have text, so the reasoning does not carry over to our text bodies.
- D: Google Docs combines OT with a separate server protocol [S46]. 
- D: ONLYOFFICE gives up redo in Fast mode because it "may cause conflicts" [S14]. I: this is the cost of not transforming history, though that source does not give the mechanism.
- I: Collabora's single process avoids the problem and pays with a server process per document [S27] and needs a live connection. That rules it out for our WASM-first design.

Against:
- D: ONLYOFFICE ships presentation co-editing on object marking [S13]. I: for slides, where people mostly work on different objects, object-level may already satisfy most users, so character OT in one text box could be a lot of cost for a rare case. Nobody has published data on how often two people type in one slide text box.
- D: Figma's fractional indexing for child order [S73] is simpler than insert-after anchors, and worth comparing in the z-order and slide-order ops.

### LibreOffice as a reference (F7). **Supported, in the role [51](51-SPEC-CHECKLIST-AND-FIDELITY.md) gives it.**

- D, excerpt: its layout differs from PowerPoint on autofit and SmartArt [S20]; regressions occur, such as 3D rotation [S22]; so "a difference is a question, not a verdict" is right.
- I: it cannot judge inheritance or retention, because it converts on import. It cannot judge SmartArt layout where it uses the fallback drawing [S19]. Do not count F7 agreement on those families at all.
- I: its bug tracker is a free catalogue of behaviours to test. Titles are facts we can reuse as test ideas without touching code. Automated access is blocked, so a person has to read it or use the openly served `whatsnew` listings [S22][S24].
- I: a second opinion from Aspose would add a commercial view, at a licence cost and with its own gaps [S62][S65]. Nothing here justifies buying it before `PV-01`, the real PowerPoint.

### ADR-012: ribbon. **Supported, not unanimous.**

O, D: ONLYOFFICE and Collabora have tabs; LibreOffice offers tabs as one of six and says "Choice is the point" [S26]; Google and Zoho do not [S36][S50]. I: products aimed at PowerPoint migrants converge on tabs, while products with their own audience do not. This matches our target.

### [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md): specific notes

- **Aptos.** The page says no metric-compatible open face is known. D, self-reported: **Akasia**, SIL OFL 1.1, "metric-compatible" with Aptos, built from OFL families, about 99% of the character repertoire, tag dated June 2026, last seen July 2026 [S74 excerpt]. A LibreOffice bug from 2024 found Source Sans metrics "nowhere near equivalent" [S75 excerpt], so a real claim here would be new. **Not verified**: the Codeberg page refused automated fetches, and no independent test was found. Compare its `hmtx` with real Aptos under `SPK-01` or a sibling task, and do a rights review, before relying on it.
- **Slide show WebGL.** [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) holds WebGL back for transitions that need it. O, D: Collabora moved its whole slideshow to WebGL [S30][S31]; ONLYOFFICE has `TransitionsGL.js` [S2]. I: expect to need it sooner than Morph.
- **Input and accessibility.** O, D: hidden input element [S5], side DOM [S39]. Supported.

## What would change our mind

| Decision | Evidence that would change it | Where it would come from |
| --- | --- | --- |
| ADR-004 engine in WASM | Keystroke-to-paint over the 16 ms budget on a 2,000-character body on a mid-range laptop, after profiling; or bundle size past the limit `SPK-01` sets and no lazy path. Then move layout to JS on engine metrics, keeping shaping in WASM (ONLYOFFICE's split) | `SPK-01` |
| ADR-004 at all | Browser `measureText` line breaks proving identical across Chrome, Safari, Firefox, Windows and macOS, with the same web fonts, on the corpus. Then Univer's cheaper option is defensible | Add as a `SPK-01` experiment |
| ADR-005 paths | Arm (b) or (c) visibly better at 11 pt to 14 pt on 1× displays in a blind comparison, or a frame-time win at 200 shapes | `SPK-01` |
| ADR-005 keep paths | Paths acceptable in that comparison and cheaper than the alternatives. Then this study's challenge is dismissed | `SPK-01` |
| ADR-013 tree | Memory per open part blocking a 500-slide deck under lazy parsing, or stale-derived-content rules growing into the same long tail the design meant to avoid. Fall back to fragments only then, before Tier 1D | Corpus, Tier 1D |
| ADR-009 character OT | Usage data showing same-body concurrent typing is rare, plus a prototype showing object-level locks are acceptable to users. Then ship object locking first and keep the op set ready. **The opposite** is also a trigger: if text OT plus deterministic layout diverges in `TP1`, stop and redesign | Pilot telemetry, `TP1` |
| Server-ordered at all | A hard offline-first requirement from enterprise customers, which server ordering serves poorly | [10](10-ENTERPRISE-REQUIREMENTS.md) |
| LibreOffice as reference | A cheaper way to reach `PV-01` that removes the need for it; or F7 false-alarm rates so high that triage costs more than it finds | F7 logs |
| Aptos substitution | Independent verification of Akasia's metrics; or a host font seam covering most customers | Font task |
| This study's reading of Google | Observing editing mode, or Google moving Slides to canvas as it did for Docs. Then re-run the observation | Repeat [S36] with a signed-in session, by a person |

## What to copy: ideas only

### Licences and clean-room practice

- **ONLYOFFICE (`sdkjs`, `web-apps`, `core`): AGPL-3.0** with additional terms; non-code assets including icons CC BY-SA 4.0; no trademark rights [S5][S15]. **PPTist: AGPL-3.0** too [S56]. Reading for ideas is one thing, but **no code, no assets, no translated logic** may enter this repository. I suggest that whoever implements a behaviour not read these repositories' implementation of it, and that a person with legal authority confirm this policy.
- **LibreOffice and Collabora: MPL-2.0**, with exceptions per file [S16][S35]. File-level copyleft still applies to anything copied. Do not copy code. Their test documents have their own rights; [51](51-SPEC-CHECKLIST-AND-FIDELITY.md) requires a rights review before any third-party fixture is committed. Regenerate the case rather than taking the file.
- **Univer: Apache-2.0** for the open engine, and commercial for Slides [S59]. **`pptxtojson`: MIT** [S55]. **reveal.js: MIT** [S60]. Permissive, but we have no need for their code, and [08](08-ADR-REGISTER.md) requires an ADR for any new engine dependency.
- **Aspose, Google, Zoho, WPS:** proprietary. Behaviour only. This is not legal advice.

### Ideas, ranked by value for cost

| # | Idea | Seen at | Why |
| --- | --- | --- | --- |
| 1 | **Publish a per-feature compatibility page** with supported, partial, not supported, generated from `spec/` and the fixtures | Zoho [S48] | Nobody else is this clear, and our checklist can make it true by construction |
| 2 | **Name silent loss as the enemy** and report what was retained but not drawn | TDF [S18] | Fits the compatibility report |
| 3 | **Show SmartArt from the saved drawing, edit from own layout**, in that order | LibreOffice, ONLYOFFICE [S19][S9] | Fidelity early; layout engine later. Needs the stale-part rule |
| 4 | **Coloured dashed outlines with a name on hover** for who is editing an object | ONLYOFFICE [S13] | Cheap, and needed even with OT |
| 5 | **Three-word font vocabulary**: replacement, substitution, fallback; runtime policy not stored in the file | Aspose [S63] | Clears up a muddled area in the host seam and report |
| 6 | **Validity-test at the scale of hundreds of thousands**, with a public progress chart | Collabora [S33] | F4 and the validator are cheap to scale. Rights limit what we can obtain, though, so start with what is lawful |
| 7 | **Layout lint for agents**: off-page and overlapping text, as structured findings | Univer [S59] | Directly useful in our MCP server |
| 8 | **Client watchdog** warning when the main loop passes 50 ms; **explicit canvas memory management** | Collabora [S30] | Cheap; keeps the budgets honest |
| 9 | **Pre-fetch in the direction of travel** starting about 100 ms after a slide switch | Collabora [S30] | Fits slide virtualization |
| 10 | **Delayed image loading on open** | ONLYOFFICE 8.2 [S10] | Already implied by lazy parts; its presence on a roadmap shows it mattered |
| 11 | **Decide what programmatic ops do to autofit.** Google turns autofit off on API edits that affect fit | Google [S41] | An explicit rule for SDK and MCP ops, instead of an accident |
| 12 | **SVG export with glyph paths** | Google [S37] | Portable and independent of fonts |
| 13 | **Fractional indexing** for child order, to compare with insert-after anchors | Figma [S73] | Possibly simpler; compare, not adopt |
| 14 | **State the fidelity limits plainly in the README** | PPTist [S56] | Sets expectations, and fits our "no claim without `PV-01`" rule |

**Do not copy:** per-mode differences in collaboration, where mixed Fast and Strict users do not see each other's changes [S14]; giving up redo [S14]; SmartArt-as-groups as the only path [S48]; a global fidelity percentage with no method [S53][S56].

## Hardest fidelity problems, as fixture candidates

Ranked by how many independent products report them, how visible the failure is, and how long others took. The ranking is my judgement (I); the reports are as cited. `BEH` ids refer to [51](51-SPEC-CHECKLIST-AND-FIDELITY.md); "new" means no row covers it yet as far as I can tell.

| # | Problem | Who reports it | Fixtures to write | Row |
| --- | --- | --- | --- | --- |
| 1 | **Missing fonts and the reflow cascade** | Google [S45], Zoho [S48], Aspose [S62][S64], LibreOffice [S75], [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) | Aptos default deck; Calibri with and without Carlito; title near a wrap boundary; CJK and Latin mixed with fallback; embedded font with and without rights; substitution reported | BEH-13, BEH-08 |
| 2 | **Autofit: stored scale, recomputation, inherited switch** | LibreOffice [S20], python-pptx [S69], Aspose [S65], Microsoft [S68], Google [S41], ONLYOFFICE [S9] | `normAutofit` absent on the slide but set in the layout; stored scale with no edit; one character added; `lnSpcReduction` at 10% and 20%; wrap off; hard breaks; empty trailing paragraph; autofit plus a substituted font | BEH-10, BEH-21 |
| 3 | **SmartArt** | LibreOffice [S19][S20], ONLYOFFICE [S9], Zoho [S48], Aspose [S66], Google [S45] | Each layout family with a drawing part; legacy without one; data changed with stale drawing; rotated parts; text wrapping inside nodes; animation by node; SmartArt in a group | New |
| 4 | **Animation timing tree** | ONLYOFFICE (four years) [S8][S9], Google [S45], LibreOffice [S76 excerpt] | Delays with two decimals; triggers; motion paths; by-paragraph, by-word; chart by series; SmartArt by node; animation on a placeholder inherited from the layout; save without loss | BEH-23 |
| 5 | **Round-trip validity and repair prompts** | LibreOffice [S22][S23], PptxGenJS [S70], Microsoft Q&A [S77 excerpt] | Extreme and negative `a:off` and `a:ext`; PPT-origin values; two OLE objects on different slides; duplicated ids; relationship targets; each passing F5 | BEH-22, F4, F5 |
| 6 | **Newer chart types and chart retention** | TDF [S18], Zoho [S48], Google [S45] | Each `cx` chart with `AlternateContent`; region map; radar; 3D; trendlines; error bars; cached values against embedded workbook; edit data then save | New |
| 7 | **Master, layout and theme inheritance across export** | LibreOffice [S21][S25][S22], Google [S45] | One master with two layouts survives as one; footers from layout; date placeholder; theme override part; paragraph style not turned into direct formatting; `a:off` in a placeholder that inherits | BEH-01, BEH-02, BEH-04, BEH-19 |
| 8 | **Text alignment and paragraph behaviour** | LibreOffice [S22] (excerpt [S78], meta-bug on PPTX paragraphs), PowerPoint adding a line [S20] | Centred text from layout; spacing in points against percent; first-line metrics by font | BEH-09, BEH-02 |
| 9 | **Non-Latin and vertical layout** | Zoho [S48], ONLYOFFICE ongoing RTL work [S9][S10] | Arabic mixed with digits; vertical CJK; multi-column; `upright`; kinsoku | BEH-06, BEH-08, BEH-12 |
| 10 | **Effects and 3D** | Zoho [S48], Google [S45], LibreOffice [S22], Aspose [S66] | Glow, soft edge, reflection; shadow size; 3D rotation; bevel; each shown and retained even when it cannot be drawn | BEH-17, new for 3D |
| 11 | **Transitions and `mc:AlternateContent`** | Google [S45], ONLYOFFICE [S9] | Morph with fallback; Office 2013 and later transition with `AlternateContent`; the `Ignorable` list preserved | BEH-18, BEH-24 |
| 12 | **WordArt and text effects** | Google [S45], Zoho [S48] | Text transform, gradient text, text transparency, outline | New |
| 13 | **Fills and picture recolour** | Zoho [S48], Google [S45] | Path gradient; gradient line; pattern fill; set-transparent-colour; duotone; picture fill on a shape | BEH-15, BEH-16 |
| 14 | **Rounding drift** | Google 3 pt to 4 pt, inches [S45] | Thin lines; EMU values needing exact round-trip; nested group offsets | BEH-22, BEH-05 |
| 15 | **Tables** | Google [S45], LibreOffice [S24] | Split and merged cells; `tableStyleId` not in `tableStyles.xml` | BEH-14 |
| 16 | **Malformed but real input** | LibreOffice [S24], [S22] | `p:pos` with no `p:cm`; unknown table style GUID. Must never crash | Hostile set, [07](07-SECURITY-THREAT-MODEL.md) |
| 17 | **Media and OLE** | LibreOffice [S22][S79 excerpt], Zoho [S48] | Audio classification on export; narration; video types; OLE preview image kept | New |

Worth adding as **behaviour rows** if the owner agrees: SmartArt drawing-part staleness, `cx` chart retention, stale derived content in general, 3D retention, and WordArt. I did not change [51](51-SPEC-CHECKLIST-AND-FIDELITY.md).

## Source register

All checked **2026-11-07**. **How read:** Fetch means the page was fetched and read through a summarizer, quotes as returned. API means GitHub API or raw file. Run means headless Chrome with a scratch profile or a direct download. Excerpt means a search-result excerpt only, because the page was not opened or refused access; treat these as weakest.

| ID | Source | Page date | URL | How read |
| --- | --- | --- | --- | --- |
| S1 | ONLYOFFICE, "The first document editor" | 2026-06 | https://www.onlyoffice.com/blog/2026/06/the-first-doc-editor | Fetch |
| S2 | `sdkjs/slide` tree | Pushed 2026-10-15 | https://github.com/ONLYOFFICE/sdkjs/tree/master/slide/Drawing | API |
| S3 | `sdkjs/common/libfont`, `common/Drawings/Format` trees | Same | https://github.com/ONLYOFFICE/sdkjs/tree/master/common/libfont | API, raw |
| S4 | `core/3DPARTY.md` | Pushed 2026-10-15 | https://github.com/ONLYOFFICE/core/blob/master/3DPARTY.md | Raw |
| S5 | `sdkjs` files: `word/Drawing/Graphics.js`, `common/libfont/rasterheap.js`, `common/Drawings/Format/DrawingContent.js`, `Shape.js`, `common/text_input2.js`, `slide/api.js`, `slide/Editor/CollaborativeEditing.js`, `common/docscoapi.js` | Same | https://github.com/ONLYOFFICE/sdkjs | Raw, `grep` of identifiers and headers |
| S6 | ONLYOFFICE, Presentation Editor advanced settings | Undated | https://helpcenter.onlyoffice.com/docs/userguides/presentation_editor/AdvancedSettings.aspx | Fetch |
| S7 | `core`: `X2tConverter`, `OOXML/Binary/Presentation`, `OOXML/PPTXFormat` | Pushed 2026-10-15 | https://github.com/ONLYOFFICE/core/tree/master/OOXML | API, raw |
| S8 | DocumentServer issue 348 | 2018 to 2022 | https://github.com/ONLYOFFICE/DocumentServer/issues/348 | Fetch |
| S9 | DocumentServer changelog | 10.0.0 at top | https://github.com/ONLYOFFICE/DocumentServer/blob/master/CHANGELOG.md | Raw |
| S10 | DocumentServer roadmap; 9.1 release | 2025-10 for 9.1 | https://github.com/ONLYOFFICE/DocumentServer/blob/master/ROADMAP.md | Fetch, excerpt for 9.1 |
| S11 | ONLYOFFICE forum, transparency in animation | v8 era | https://forum.onlyoffice.com/t/bug-presentation-transparency-lost-in-animation/8049 | Excerpt |
| S12 | `web-apps` presentation views | Pushed 2026-10-15 | https://github.com/ONLYOFFICE/web-apps/tree/master/apps/presentationeditor/main/app/view | API |
| S13 | ONLYOFFICE, co-editing presentations | Undated | https://helpcenter.onlyoffice.com/docs/userguides/presentation_editor/CollaborativeEditing.aspx | Fetch |
| S14 | ONLYOFFICE, Fast vs Strict | 2020-07 | https://www.onlyoffice.com/blog/2020/07/freedom-to-choose-your-collaboration-fast-vs-strict-co-editing-modes-in-onlyoffice | Fetch |
| S15 | DocumentServer repo metadata | Pushed 2026-10-26 | https://github.com/ONLYOFFICE/DocumentServer | API |
| S16 | LibreOffice core: `COPYING.MPL`, `vcl`, `oox`, `sd/qa/unit` | Pushed 2026-11-06 | https://github.com/LibreOffice/core | API, raw |
| S17 | TDF, LibreOffice 26.8 announcement | 2026-08-26 | https://blog.documentfoundation.org/blog/2026/08/26/libreoffice-26-8/ | Fetch via monthly index |
| S18 | TDF, 26.8 and interoperability | 2026-09-17 | https://blog.documentfoundation.org/blog/2026/09/17/libreoffice-26-8-and-interoperability/ | Fetch |
| S19 | Vajna, Improving SmartArt import | 2019-02-02 | https://speakerdeck.com/vmiklos/improving-smartart-import-in-libreoffice-impress | Fetch |
| S20 | LibreOffice bugs 122023, 136228, 157773, 107694, 148828 | Various | https://www.libreoffice.org/bugzilla/show_bug.cgi?id=122023 (others by id; tree at https://www.libreoffice.org/bugzilla/showdependencytree.cgi?id=156920) | Excerpt; Bugzilla refused fetch |
| S21 | Vajna, Document themes in Impress | 2022-02-05 | https://speakerdeck.com/vmiklos/document-themes-in-libreoffice-impress-and-elsewhere | Fetch |
| S22 | LibreOffice 26.2.2 what's new | 2026 | https://whatsnew.libreoffice.org/26.2.2/ | Run (listing titles; fixed or open status not stated) |
| S23 | Linuxiac, LibreOffice 26.2.2 | 2026 | https://linuxiac.com/libreoffice-26-2-2-office-suite-released/ | Excerpt; fetch 404 |
| S24 | LibreOffice 26.8.1 what's new; TDF 26.8.1 post | 2026-10-02 | https://whatsnew.libreoffice.org/26.8.1/ | Run, Fetch |
| S25 | LibreOffice bug list, PPTX export reports | 2020 | https://lists.freedesktop.org/archives/libreoffice-bugs/attachments/20200703/25b7936d/attachment-0001.htm | Excerpt; summarized across that query's results, so the exact page was not isolated |
| S26 | TDF, There is no best interface | 2026-08-21 | https://blog.documentfoundation.org/blog/2026/08/21/there-is-no-best-interface/ | Fetch |
| S27 | Collabora SDK, architecture | Undated | https://sdk.collaboraonline.com/docs/architecture.html | Excerpt; site refused fetch. Also https://mautic.collaboraoffice.com/asset/213:fosdem23-tor-lillqwist-cool-wasmpdf (excerpt) |
| S28 | Online Deep Dive 2016 | 2016 | https://conference.libreoffice.org/assets/Conference/Brno/Online-DeepDive-2016.pdf | Excerpt; fetch 404 |
| S29 | FOSDEM 2022, LibreOfficeKit recent developments | 2022 | https://mautic.collaboraoffice.com/asset/175:fosdem2022janholesovskylibreofficekit-recentdevelopmentspdf | Excerpt, plus Run |
| S30 | Vajna, Collabora Online 25.04 performance | 2025-06-04 | https://speakerdeck.com/vmiklos/collabora-online-25-dot-04-performance-improvements | Fetch |
| S31 | Collabora Online mirror, `browser/src/slideshow` | 2026 | https://github.com/CollaboraOnline/online.mirror/tree/main/browser/src/slideshow | API, raw |
| S32 | Same PDF as S29, text extracted | 2022 | https://mautic.collaboraoffice.com/asset/175:fosdem2022janholesovskylibreofficekit-recentdevelopmentspdf | Run |
| S33 | Collabora, CODE 26.04 release | 2026-06-08 | https://www.collaboraonline.com/blog/code-26-04-release/ | Fetch |
| S34 | Collabora, FOSDEM 2022 talks | 2022 | https://www.collaboraonline.com/blog/talks-and-slides-by-collabora-at-fosdem-2022/ | Excerpt |
| S35 | Collabora Online mirror README | 2026 | https://github.com/CollaboraOnline/online.mirror/blob/main/README.md | Raw |
| S36 | Google Slides, public deck "Baby album", rendered DOM and screenshot | Observed 2026-11-07 | https://docs.google.com/presentation/d/1EAYk18WDjIG-zp_0vLm3CsfQh_i8eXc67Jo2O9C6Vuc/edit | Run |
| S37 | Same deck, SVG export | Observed 2026-11-07 | https://docs.google.com/presentation/d/1EAYk18WDjIG-zp_0vLm3CsfQh_i8eXc67Jo2O9C6Vuc/export/svg | Run |
| S38 | Google, Docs canvas rendering update | 2021-05 | https://workspaceupdates.googleblog.com/2021/05/Google-Docs-Canvas-Based-Rendering-Update.html | Fetch |
| S39 | The New Stack, Docs switches to canvas | 2021 | https://thenewstack.io/google-docs-switches-to-canvas-rendering-sidelining-the-dom/ | Excerpt; summarized across that query's results, so the exact page for "side DOM" was not isolated |
| S40 | Slides API, shapes reference | Undated | https://developers.google.com/workspace/slides/api/reference/rest/v1/presentations.pages/shapes | Excerpt |
| S41 | Slides API, add shapes and text | Undated | https://developers.google.com/workspace/slides/api/guides/add-shape | Fetch |
| S42 | Google Drive Help, files you can store | Undated | https://support.google.com/drive/answer/37603?hl=en | Fetch |
| S43 | Google Cloud Community, 100 MB thread | Undated | https://www.googlecloudcommunity.com/gc/Workspace-Q-A/100-mb-size-limit-google-presentation/m-p/405725 | Excerpt |
| S44 | Google, Switch from PowerPoint to Slides | Undated | https://support.google.com/docs/answer/9310378?hl=en | Fetch |
| S45 | BrightCarbon, Convert PowerPoint to Google Slides | 2021-05-12 | https://www.brightcarbon.com/blog/convert-powerpoint-google-slides/ | Fetch |
| S46 | Google Drive Blog, making collaboration fast | 2010-09 | https://drive.googleblog.com/2010/09/whats-different-about-new-google-docs.html | Excerpt; fetch returned no article body |
| S47 | Zoho Show 3.0 press release | 2011-09-27 | https://www.newswire.com/news/zoho-announces-zoho-show-3-0-181594 | Fetch |
| S48 | Zoho, PowerPoint compatibility | Undated | https://www.zoho.com/show/powerpoint-compatibility.html | Fetch |
| S49 | Zoho, import and export | Undated | https://help.zoho.com/portal/en/kb/show/web-app/create-presentations/articles/import-and-export-presentations-zohoshow | Fetch |
| S50 | Zoho, Show vs PowerPoint | 2026-04-30 | https://www.zoho.com/show/chronicles/zoho-show-vs-microsoft-powerpoint.html | Fetch |
| S51 | WPS, What is WPS Office | 2026 | https://www.wps.com/academy/what-is-wps-office/about-wps/1863074/ | Excerpt |
| S52 | Yicai, Kingsoft Office annual report summary | 2025 | https://yicaiglobal.com/bulletin/253000000533636052 | Excerpt |
| S53 | UiB help, WPS | Undated | https://hjelpekort.app.uib.no/en/KI-3455.html | Excerpt |
| S54 | G2, WPS Presentation reviews | Undated | https://www.g2.com/products/wps-presentation/reviews | Excerpt |
| S55 | PPTist demo DOM; repo files; `pptxtojson` metadata | Pushed 2026-11-06 | https://pipipi-pikachu.github.io/PPTist/ and https://github.com/pipipi-pikachu/PPTist and https://github.com/pipipi-pikachu/pptxtojson | Run, API, raw |
| S56 | PPTist README | 2026 | https://github.com/pipipi-pikachu/PPTist/blob/master/README.md | Fetch, raw |
| S57 | Univer, rendering architecture | Undated | https://docs.univer.ai/blog/rendering | Fetch |
| S58 | Univer `engine-render` | Pushed 2026-11-06 | https://github.com/dream-num/univer/blob/dev/packages/engine-render/src/components/docs/layout/shaping-engine/font-cache.ts | Raw |
| S59 | Univer README and `packages` tree | Same | https://github.com/dream-num/univer/blob/dev/README.md | Raw, API |
| S60 | reveal.js README | Pushed 2026-09-30 | https://github.com/hakimel/reveal.js | Raw, API |
| S61 | Aspose.Slides product overview | Undated | https://docs.aspose.com/slides/java/product-overview/ | Fetch |
| S62 | Aspose.Slides, convert to JPG FAQ | Undated | https://docs.aspose.com/slides/net/convert-powerpoint-to-jpg/ | Excerpt |
| S63 | Aspose.Slides, font replacement and fallback | Undated | https://docs.aspose.com/slides/net/font-replacement/ and https://docs.aspose.com/slides/net/create-fallback-font/ | Excerpt |
| S64 | Aspose.Slides, manage autofit | Undated | https://docs.aspose.com/slides/net/manage-autofit-settings/ | Fetch |
| S65 | Aspose.Slides for .NET 17.8 release listing | 2017 | https://www.componentsource.com/es/product/aspose-slides-net/releases/2383006 | Excerpt |
| S66 | Aspose forum, rotated SmartArt cropped | 2025 | https://forum.aspose.com/t/rotated-parts-of-smartart-are-cropped-when-converting-to-image-in-java/311275 | Excerpt |
| S67 | Microsoft, Introduction to markup compatibility | 2025-01-08 | https://learn.microsoft.com/en-us/office/open-xml/general/introduction-to-markup-compatibility | Fetch |
| S68 | [MS-OE376] normAutofit | 2025-03-18 | https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/37891df4-0cfe-414a-a4bc-ff02dd817f4f | Fetch |
| S69 | python-pptx, autofit analysis | Undated | https://python-pptx.readthedocs.io/en/latest/dev/analysis/txt-autofit-text.html | Fetch |
| S70 | PptxGenJS, needs-repair errors | Undated | https://gitbrent.github.io/PptxGenJS/docs/needs-repair-errors | Fetch |
| S71 | Withdrawn: a report of equations lost on PPTX save, whose page could not be isolated. Not used | | | |
| S72 | WHATWG thread; Stack Overflow archive | 2013; to 2023 | https://lists.whatwg.org/pipermail/whatwg-whatwg.org/2013-April/081589.html | Excerpt |
| S73 | Figma, How multiplayer works | 2019-10-16 | https://www.figma.com/blog/how-figmas-multiplayer-technology-works/ | Fetch |
| S74 | Akasia | 2026-06 to 07 | https://codeberg.org/bloudraad/akasia | Excerpt; fetch returned 404 |
| S75 | LibreOffice bugs 159950, 162872 | 2024 | https://lists.freedesktop.org/archives/libreoffice-ux-advise/2024-September/046999.html | Excerpt |
| S76 | LibreOffice dependency tree 106547 | Undated | https://www.libreoffice.org/bugzilla/showdependencytree.cgi?id=106547 | Excerpt |
| S77 | Microsoft Q&A, why does this PPTX need repair | Undated | https://learn.microsoft.com/en-gb/answers/questions/2281200/why-does-this-pptx-need-repaired | Excerpt |
| S78 | LibreOffice meta-bug 107899, PPTX paragraphs | 2020 | https://lists.freedesktop.org/archives/libreoffice-bugs/2020-November/332088.html | Excerpt; fetch 403 |
| S79 | Linuxiac, LibreOffice 26.2.1 | 2026 | https://linuxiac.com/libreoffice-26-2-1-released-with-over-70-bug-fixes/ | Excerpt; fetch 404 |

# 06 — Roadmap and delivery

**This is an enterprise product, not an MVP.** The rules for opening and closing are in
[05](05-PHASES-AND-TASKS.md). Each phase below is a charter: what it gives, what opens it, what is in and
out, and what closes it.

**Order follows the owner's priority (2026-11-07):** fidelity is the most critical thing, so import, export and
modelling come first, then layout and rendering, then the editor. **Web and desktop are both first-class** (`DEC-06`).

**PowerPoint verification is a release gate, not a phase blocker** (`DEC-04`, `PV-01`). Until it passes, phases close against LibreOffice, schemas and Microsoft's validator, and nothing may claim to match PowerPoint.

**No calendar dates.** Row counts measure surface, not effort. Phase 1's history calibrates throughput: Phase 2's ETA is in hours of work, sized from it (instruction 41).

## Where things stand

| Phase | State | Note |
| --- | --- | --- |
| D Design | **Open** | Blocks nothing structurally. Owner decisions remain |
| 0 Foundation | **Closed** 2026-11-07 | Signed off by the owner. Seeded violations were run locally on macOS, not on GitHub's runners. Fonts measured in `SPK-01` left the repo and return in Phase 2 |
| 1 Model, import, export | **Open** | See the Phase 1 plan below |
| 2 Layout and render | Not opened | Plan proposed below, for the owner's approval (instruction 41). **No rendering, painting or font work until Phase 1 closes** (owner instruction 20) |
| 3 to 10 | Not opened | — |

### Waivers

| ID | Criterion not met | Risk | Accepted | If it fails |
| --- | --- | --- | --- | --- |
| W-1 | Phase 1 opened before Phase 0 closed: CI not run, no PowerPoint-written decks | Code written against decks only we and LibreOffice wrote; failures on other platforms found late | Owner's "proceed", 2026-11-07 | Fix in tier; Phase 1 still cannot close without these Resolved 2026-11-07: Phase 0 closed. |

## Phase D — Design

- **Gives:** an agreed design, so that later phases need no do-over.
- **Opens:** project start.
- **In:** architecture, layering, model and op design, threat model, enterprise requirements, feature study, spec checklist, process.
- **Out:** product code.
- **Closes when:**
  - [ ] Every ADR needed by Phases 0 to 2 is Accepted or has a named spike deciding it.
  - [ ] Each `DEC-*` is answered, or the owner has marked it as not blocking and named the phase that needs it.
  - [ ] The doc audit's P0 and P1 findings are fixed (`DOC-003`), and the gap analysis is dispositioned (`DOC-004`).
  - [x] Owner sign-off, 2026-11-07.

## Phase 0 — Foundation

- **Gives:** a repository where a broken change cannot land unnoticed.
- **Opens:** Phase D's architecture ADRs Accepted.
- **In:** workspace, copied crates, package security, fixtures, CI, fuzz, spikes.
- **Out:** presentation semantics.
- **Closes when:**
  - [x] Package threats `SEC-P01`–`P07`, `P12`, `P13` have tests seen red. Evidence: `SEC-010`, `SEC-013`.
  - [x] Spec checklist generated, and fails on an unassigned type. Evidence: `SPEC-05`.
  - [x] CI green on three platforms: all 9 jobs at `ec7e3d8`. A seeded violation was seen failing each job, run locally on macOS ([15](15-CI-AND-RELEASE-GATES.md)), not on GitHub's runners.
  - [x] Fixture corpus has provenance recorded, including real PowerPoint-written decks as input. Evidence: `F-004`, `F-007`, 66 decks.
  - [x] `SPK-01` reported; ADR-004 and ADR-005 settled. Evidence: [64](64-SPK-01-TEXT-COST.md).
  - [x] `SPK-02` runs 5 decks end to end against LibreOffice; Microsoft's validator runs. Evidence: `SPK-02`, `F-006`.
  - [ ] Owner sign-off.

## Phase 1 — Model, import, export

- **Gives:** any `.pptx` opens and saves with nothing lost, and every construct is accounted for. **No UI, and nothing drawn.**
- **Opens:** Phase 0 closed, or a waiver. Currently W-1.
- **In:** every row in [spec/](spec/README.md), by tier below; behaviours BEH-01 to 04, 18, 21, 22, 24 in [51](51-SPEC-CHECKLIST-AND-FIDELITY.md).
- **Out:** rendering; UI; edit operations beyond what round-trip tests need. Legacy `.ppt` and `.odp` (Phase 10).

| Tier | Scope | Rows | Opens after |
| --- | --- | ---: | --- |
| **1A** | OPC: content types, relationships, part graph, properties, signatures retained, Strict and Transitional, repackager | 392 | — |
| **1B** | Presentation part, view state | 297 | 1A |
| **1C** | Colour, themes | 473 | 1A |
| **1D** | Masters, layouts, slides, notes, handouts, placeholders, shape tree, `-style` resolver | 564 | 1B, 1C |
| **1E** | Geometry, fills, lines, effects, 3D | 1082 | 1D for import; the geometry engine itself is standalone |
| **1F** | Text model | 317 | 1D |
| **1G** | Tables | 113 | 1F |
| **1H** | Pictures, SVG, media | 116 | 1E |
| **1I** | Timing, transitions | 522 | 1D |
| **1J** | Charts, 2016+ charts | 1764 | 1E, 1F |
| **1K** | SmartArt, equations, ink | 1263 | 1E, 1F |
| **1L** | OLE, VML, 3D models, retained with fallback | 951 | 1H |
| **1M** | Comments, review, sections, zoom, labels, tags, custom XML | 876 | 1D |
| **1R** | **Recovery: open and edit damaged decks, including what Microsoft refuses** ([29](29-RECOVERY.md)) | damage corpus | 1A |
| **1N** | **The writer:** part naming, rId allocation, content types, shared parts, id rules, app.xml, thumbnails, signature drop, save-as matrix ([36](36-EXPORT-AND-SAVE.md)) | invariants | 1A; grows with each tier |

### Phase 1 plan

Every task below is a row in [14](14-EXECUTION-TRACKER.md). Audit coverage is re-read from `tools/spec-audit/audit.py`, not from this page.

**Where it stands (2026-11-08).** Every tier is built and merged; **All Phase 1 tiers, 1A to 1N and 1R, are closed** (2026-11-08) (2026-11-08), the rest are not. Each tier's gate, item by item, and what it still lacks are in [14](14-EXECUTION-TRACKER.md) §Tier gates. None of the Phase 1 exit criteria below is met yet. Status columns are filled by `tools/spec-audit`; a row needs code, a test and a fixture.

**Two corrections this plan makes to the table above.**
- **1C cannot close before 1E and 1F.** A theme's format scheme is made of fills, lines and effects, and object defaults of text and shape properties. So 1C finishes as `T1C-02` after those tiers.
- **1N is not one tier at the end.** Each tier that adds parts adds writer rules, so 1N runs alongside from Wave 1 and closes last.

| Wave | Runs at once, because each owns separate files | Needs |
| --- | --- | --- |
| **0: close-out** | `STAT-01`, 1A and 1C leftovers, merge 1D | — |
| **1: the shape of a slide** | 1E shape properties · 1F text · 1N core | 1D |
| **2: objects** | 1G tables · 1H pictures and media · 1I timing and transitions · 1M comments, sections, zoom | 1E, 1F |
| **3: the big parts** | 1J charts · 1K SmartArt, equations, ink | 1E, 1F |
| **4: retained, then finish** | 1L OLE, VML, 3D · `T1C-02` · 1N close | 1H, all |
| **5: exit** | Corpus at target, 500-slide budget, hostile run, sign-off | all |

**Three or four sessions at a time**, because usage limits stopped larger rounds three times. A wave does not wait for its slowest member if the next wave's dependencies are met. Each session follows the `openslide-tier` skill: read the MS-OI29500 notes, then split, build, fuzz and commit increments.

**Order inside Waves 2 and 3 follows real decks** (`F-007`): what the decks use most goes first, since that finds wrong semantics soonest. Row count is not effort: charts are the largest tier by rows, while text will be the hardest later, in Phase 2.

**What can change this plan:**
- Real decks may refuse under the stricter XML reader, or exercise constructs not yet planned. That is why `F-007` comes first.
- Throughput from 1A to 1D sizes the rest. Dates follow once 1D has merged, not before.

**Built is not done (owner instruction 22).** Each tier stays with its session until its gate below is met, item by item.

**The foundation comes first (owner instruction 20).** Phase 1 is import, export and modelling. Nothing is drawn, and no font work happens, until it closes. `SKEL-01` waits for Phase 2.

**A tier closes when:**
1. Every in-scope row is `y` or `r` for model, import and export, or has a named reason, and the audit tool agrees.
2. Each `y` row has a fixture exercising it.
3. Unedited save is byte-identical. An edit changes only named nodes of named parts. **A forced-rewrite run passes every check below, since an unedited save proves nothing about the writer (`WR-01`).**
4. Import → export → import is a fixed point.
5. Edited XML validates through F4: markup compatibility resolved, then schemas, then Microsoft's validator.
6. The report names everything not modelled, and a silent drop fails a test.
7. New limits have hostile fixtures; each new parser has a fuzz target.

**Phase 1 closes when:**
- [ ] All tiers closed.
- [ ] Every exported corpus deck passes schema validation and Microsoft's validator, and LibreOffice reopens it. The PowerPoint check is `PV-01`.
- [ ] Corpus includes real decks from PowerPoint on Windows and Mac, and Google Slides, with provenance recorded. Keynote was dropped by the owner (instruction 36).
- [ ] A 500-slide deck opens within its budget, and a hostile corpus run shows no crash or hang.
- [ ] The damage corpus opens, or is refused only under [29](29-RECOVERY.md)'s rules; an edit to a recovered deck writes a valid file, except a part kept as it arrived after a failed checksum, which is reported (owner instruction 34).
- [ ] Owner sign-off.

## Phase 2 — Layout and render


**Start from OpenCalc (owner instruction 42).** Every Phase 2 task opens by naming OpenCalc's solution it copies, with provenance (fonts and the font contract, text shaping and line breaking, the display list, the PNG and PDF backends, chart drawing, the fidelity harness), and OpenCalc's recorded mistakes and fixes in that area. A task that builds something OpenCalc already has must say why. A problem OpenCalc fixed must not come back here.
- **Gives:** any deck renders headlessly to PNG and PDF, with fidelity measured, before a UI can hide problems.
- **Opens:** Phase 1 closed and signed by the owner (instruction 20); this plan approved and `DEC-16` to `DEC-20` answered (instruction 41). ADR-004 and ADR-005 are settled.
- **In:** tiers below; behaviours BEH-05 to 20. **Out:** UI, the Canvas2D painter, editing, animation playback, and the two layout engines named in `DEC-20`.

| Tier | Scope | Opens after |
| --- | --- | --- |
| **2A** | Slide composition, master and layout, background, theme | 1D |
| **2B** | Presets, custom paths, fills, lines, group transforms | 1E |
| **2C** | Effects, flattened 3D | 1E |
| **2D1** | Fonts, fallback, the font contract (`TXT-01`) | 1F, `SPK-01` |
| **2D2** | Shaping, bidi, line breaking | 2D1 |
| **2D3** | Paragraph layout: spacing, tabs, bullets, numbering | 2D2 |
| **2D4** | Body layout: anchoring, insets, columns, vertical text, rotated text | 2D3 |
| **2D5** | CJK rules: kinsoku, hanging punctuation, East Asian line breaks | 2D3 |
| **2D6** | Autofit, and WordArt warps | 2D4 |
| **2E** | Tables, pictures, SVG, EMF | 1G, 1H |
| **2F** | Classic charts | 1J |
| **2H** | 2016+ charts | 1J |
| **2I** | Ink, OLE and VML fallbacks | 1K, 1L |
| **2J** | SmartArt from its stored drawing. The layout engine for decks with none (`DGM-01`) is in only if `DEC-20` keeps it | 1K |
| **2K** | Equations: from the stored fallback picture; math layout only if `DEC-20` keeps it | 1K |
| **2G** | PDF with tags, reading order, alt text, links; JPEG; print layouts | 2A to 2K |

### Phase 2 plan

**Proposed 2026-11-08, for the owner's approval (instruction 41).** Nothing below starts before Phase 1 is signed.

**Scope in one line per area.** Out is everything not named.

| Area | In | Out |
| --- | --- | --- |
| Display list | `Path`, `GlyphRun`, `Image`, `Group`, `Effect` items as in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md); serializable, golden-tested, same bytes on every platform | Canvas2D painting (Phase 3) |
| Fonts | OpenCalc's faces copied with provenance; resolution by theme font, `+mj-lt`, script and fallback; every substitution reported (`TXT-01`) | Bundling Aptos, Calibri or any face we may not ship |
| Text | Shaping, bidi, line breaking, paragraphs, bullets, fields, body text, vertical and rotated text, CJK rules, Thai on demand (`TXT-07`), stored autofit, WordArt warps | Recomputing autofit after edits (Phase 5) |
| Geometry | All 187 presets and custom paths, drawn from the existing `casual-slide-geometry`; group transforms | — |
| Fills, lines, effects | Solid, gradient, pattern, picture; dash, caps, joins, arrows; shadow, glow, soft edge, reflection, blur; 3D flattened to 2D | True 3D (Phase 10, `D3-01`) |
| Pictures | Raster with crop, tile and recolour; SVG; EMF and WMF records that real decks use, the rest reported | Video and audio frames beyond the poster image |
| Tables | Grid, merges, borders, margins, style evaluation including built-in style definitions (`T2E-01`) | — |
| Charts | Classic chart families from their caches; 2016+ families | Recalculating from embedded workbooks |
| SmartArt, equations, ink, OLE, VML | Drawn from what the file stores: diagram drawing, fallback picture, ink trace, OLE preview | Layout engines, unless `DEC-20` keeps them |
| Backends | PNG through tiny-skia; PDF written by us as OpenCalc does, with subset fonts, real text, tags, links, alt text; JPEG; handouts and notes pages | Printing to a device |
| Fidelity harness | Reference renders, pixel compare, line-break compare, determinism compare, all on GitHub CI | Running anything on the owner's machine (instruction 38) |

**Targets.** Each is a number, a method and a corpus. Numbers are proposals (`DEC-19`): the first harness run may show LibreOffice itself cannot be met, and then the owner sets them. After that a threshold may be tightened per fixture, never loosened without the owner.

| # | Target | Proposed | How measured, on GitHub CI | Corpus |
| --- | --- | --- | --- | --- |
| R1 | Shapes, fills, lines, pictures, tables against the reference | ≤ 1% of pixels differ per slide; effects ≤ 3% | A pixel differs when its colour is more than ΔE 10 from every pixel within 1 px of it; 96 dpi | `fixtures/generated`, one construct per deck |
| R2 | Line breaks against the reference | Same line ends in ≥ 98% of paragraphs whose font is present or metric-compatible | Line ends read from the reference PDF's text positions, against our line boxes | Generated text decks, `real/`, `producers/` |
| R3 | Whole real slides against the reference | Reported per slide; not a gate | As R1 | `real/`, `producers/` |
| R4 | PowerPoint's own PDF export, if the owner supplies it (`DEC-17`) | ≤ 2% per slide, ≥ 99% of line ends | As R1 and R2 | Only those decks |
| R5 | Determinism | Byte-identical PNG and PDF on Linux x86_64, macOS arm64, Windows x86_64; identical display list in wasm32 | Hashes compared across the CI matrix | Every corpus deck |
| R6 | Time, 500-slide deck | Every slide to PNG at 1280×720 ≤ 60 s; to PDF ≤ 30 s; p95 per slide ≤ 250 ms | Timed release build on `ubuntu-latest`, asserted | Phase 1's 500-slide deck, plus a dense-text copy |
| R7 | Memory | Peak ≤ 1 GiB for R6; ≤ 300 MiB for one slide | `/usr/bin/time -v` | As R6 |
| R8 | wasm | Layout and display list build for `wasm32-unknown-unknown`; ≤ 1 MiB Brotli added, fonts excluded (`SPK-01` measured 313 KiB for text) | Size of the release build; output compared with native under Node | Generated decks |
| R9 | Hostile input | No panic, no hang; each slide renders or reports within 5 s | The hostile and damage corpora (6,203 decks) | `hostile/`, damage corpus |
| R10 | Nothing left out silently | Every construct not drawn is named in the render report | A test switches a drawer off and must see the report change | Generated decks |
| R11 | Behaviours | BEH-05 to 20 each have fixtures that pass | Named tests | Generated decks |
| R12 | No network | No network capability in the render path | A test, as the exit criterion says | — |

**What stands in for PowerPoint, and how far a claim goes.** No PowerPoint reference exists until `PV-01`. The reference is **LibreOffice from Ubuntu's apt on GitHub's runner**, run with no network as `phase1-exit.yml` does, its version recorded with each reference and every result labelled LibreOffice. R1 to R3 can therefore claim only "agrees with LibreOffice within N": a second opinion (F7), never a match with PowerPoint. Where we and LibreOffice differ, the row says whose behaviour it is; where LibreOffice is wrong, the fixture records that and is judged against the standard. R5 to R12 need no reference and can be claimed outright. R4 is the only F6 evidence, and it covers only the decks it was run on, and only PowerPoint's PDF export, not its screen.

**Decisions for the owner.** Five, each with a recommendation. Three need an ADR, written when the phase opens.

| ID | Question | Recommendation | ADR |
| --- | --- | --- | --- |
| `DEC-16` | Which fonts ship? | OpenCalc's set: Carlito, Caladea, Liberation, sliced Noto CJK, all open licences. Akasia stands in for Aptos only if `TXT-03` shows its widths match. No Microsoft font is shipped; a host can supply one through the font seam | ADR-016 |
| `DEC-17` | What is the reference until `PV-01`? | LibreOffice on GitHub's runner, labelled. **And, if you can: export the 12 committed real decks to PDF from PowerPoint**, about 15 minutes, which gives the only true comparison before release | ADR-017 |
| `DEC-18` | Which libraries draw? | As OpenCalc: tiny-skia for PNG, harfrust and skrifa for text, `image` with named codecs, our own PDF writer and subsetter. Add resvg for SVG and write our own bounded EMF reader | ADR-019 |
| `DEC-19` | Are the target numbers above right? | Yes as a start; the first harness run confirms or corrects them before any tier closes | — |
| `DEC-20` | Do SmartArt with no stored drawing and equations need our own layout in Phase 2? | **No.** Draw what the file stores, and report a placeholder when it stores nothing. Both engines move to Phase 6, where editing needs them. This changes `DGM-01`'s plan | — |

**Order and parallelism.** Three sessions at a time, three lanes. Text is the biggest and most serial work, so lane A starts on it at once and lane B after its first hour. Each task is a row in [14](14-EXECUTION-TRACKER.md) §Phase 2 tasks with its end point and time box. **The end point is: merged to `main`, its named tests green on GitHub CI, its BEH fixtures passing, and its checklist and tracker marks true.** A tier's gate items are a task's end point, not a later close-out. At the box, the session merges what passes, files a row for what is left and stops.

| Lane | Tasks in order, with time box in session-hours | Lane total |
| --- | --- | ---: |
| **A: text in** | `T2D1-01` fonts 1 · `T2D2-01` shaping, bidi, breaks 2 · `T2D5-01` CJK and Thai 1 · `T2F-01` classic charts 3 · `T2J-01` SmartArt and `T2K-01` equations 1 · `T2I-01` ink, OLE, VML 1 · `T2E-03` EMF and WMF 1.5 | 10.5 |
| **B: text out** | `SKEL-01` display list, crates, PNG 1 · `T2D3-01` paragraphs 2 · `T2D4-01` bodies 1.5 · `T2D6-01` autofit, warps 1.5 · `T2E-04` tables 1.5 · `T2H-01` 2016+ charts 2 · `T2G-02` tagged PDF, print 2 | 11.5 |
| **C: harness and shapes** | `FH-01` fidelity harness 1 · `T2A-01` composition 1 · `T2B-01` shapes, fills, lines 2 · `T2C-01` effects 2 · `T2E-02` pictures, SVG 1.5 · `T2G-01` PDF backend 1.5 | 9 |
| **All, last** | `P2X-01` exit run: every target measured and asserted | 1 |

Lanes A, B and C start at once. `T2D2-01` and `T2D3-01` meet at the line-box interface that `SKEL-01` fixes in its first hour; `T2B-01` waits for `SKEL-01` and `FH-01`, both one hour. Each later task starts after what it uses has merged in another lane: charts at hour 4, after paragraphs and shapes; 2016+ charts at hour 7.5, after classic charts. `T2G-02` builds tags and print layouts over whatever draws by then; the exit run covers what lands after it.

**ETA, from Phase 1's git history.** Phase 1's active build ran from the plan at 2026-11-07 17:14 to the last tier close at 2026-11-08 05:06: **about 12 hours**, 40 merged branches (the repository has 336 commits in all), **about 49 session-hours** between each branch's first and last commit, which undercounts. The waste in it, which this plan does not repeat: up to 10 sessions at once with load at 134 (rule 22); about 5 of the 12 hours spent closing tiers already built, because tasks had no end point (instruction 22); 1D held unmerged for 4.7 hours; gates run in every session; runs repeated. The owner judges Phase 1 needed under 4 hours. The boxes above are set at that pace.

| | Session-hours | Wall clock, three at once |
| --- | ---: | ---: |
| **Without waste** | 32 | **about 12 hours**: the longest lane, 11.5, then the exit hour |
| **With the risks below** | up to 45 | **up to 20 hours** |

It assumes: Phase 1 signed and `DEC-16` to `DEC-20` answered before the start; at most three sessions; every test on GitHub CI and nothing on the owner's machine (instructions 38, 39), so sessions compile locally and read results and diff images from CI; a merge needs only to compile, and CI on `main` is the gate (instruction 32); OpenCalc's text, render, PDF and chart code is copied, not rewritten; a CI run takes 20 to 30 minutes, which is not yet measured.

**Risks that move the ETA most.**

| Risk | Could add, session-hours | What limits it |
| --- | --- | --- |
| Line breaks disagree with LibreOffice for reasons that are LibreOffice's, or because Aptos has no matching face | 4 h | R2 counts only decks whose fonts are present or metric-compatible; each disagreement gets one look and a row, not a hunt |
| Every check waits on GitHub CI | 3 h | A fast Linux render job checks merges; the three-OS run checks `main` |
| Output differs between x86_64 and arm64 (`GEO-05`), in anti-aliasing or floating point | 3 h | `SKEL-01` runs the three-OS hash compare in its first hour, before anything is built on it |
| LibreOffice draws a construct wrongly, or not at all, so a threshold cannot be met by being right | 2 h | Such fixtures are judged against the standard and recorded; `DEC-17`'s PowerPoint PDFs settle the real decks |

**Closes when:**
- [ ] Each corpus deck is within its recorded threshold against its reference: a PowerPoint-exported PDF or image where the deck ships with one, else LibreOffice, labelled as such. Thresholds are those set under `DEC-19`.
- [ ] Each behaviour row has a passing fixture.
- [ ] Budgets in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) replaced with measurements and asserted, including R6 to R8 above.
- [ ] Output identical on Linux, macOS and Windows.
- [ ] No network capability in the render path, shown by a test.
- [ ] Owner sign-off.

## Phase 3 — Viewer

- **Gives:** a browser and a Tauri desktop window open a real deck, with thumbnails, views and zoom, and no editing.
- **Opens:** Phase 2 closed; `UX-001` done and the shell prototype approved by the owner.
- **In:** WASM bridge, Canvas2D painter, native desktop path, shell, rail, status bar, Normal, Sorter, Reading views, notes pane, accessible DOM, CSP.
- **Out:** every edit.
- **Closes when:**
  - [ ] Painter matches the PNG backend within threshold; web and desktop agree.
  - [ ] Browser smoke in CI on the published matrix; desktop bundles on three platforms.
  - [ ] 500-slide budget met.
  - [ ] WCAG 2.2 AA audit of the shell, with screen reader and keyboard-only passes.
  - [ ] XSS and link threats (`SEC-E01`, `SEC-E03`, `SEC-N03`) tested.
  - [ ] Owner sign-off.

## Phases 4 to 7 — Editing

**Common entry:** the previous editing phase closed, plus the prerequisites below. **Common exit:** the op set is inverse-tested and **TP1 holds for every op added (`OP-02`)**,
the UI status is filled for every in-scope checklist row, a task test by people who use PowerPoint passes for the phase's verbs, keyboard and screen
reader work for every new verb, and the owner signs.

| Phase | Gives | Opens after | In | Out | Extra exit criteria |
| --- | --- | --- | --- | --- | --- |
| **4 Objects** | Build a deck from shapes and pictures | 3 | Op set, undo, select, transform, guides, group, arrange, selection pane, insert shape and picture, slide ops, reuse slides, merge shapes, eyedropper, sections, format panes, Home, Insert, Shape Format, Picture Format, accessibility checker, **autosave and recovery** | Text editing | Drag budget on adversarial slide; a crash loses nothing (`SAVE-01`) |
| **5 Text** | Type and format, in any script | 4 | Caret, selection, IME, formatting, lists, autofit on edit, find and replace, spell seam, links, outline view, notes page view | Tables, charts | IME matrix; RTL and CJK; typing budget |
| **6 Rich objects** | Tables, charts, SmartArt, equations, media, ink | 5, 2F | Each, with Format tabs; chart data through the OpenCalc SDK | Slide show | Round-trip per construct after edit |
| **7 Design** | Themes, masters, layouts | 6 | Themes, variants, background, slide size, master and layout editing | — | Master edit propagates and round-trips |

## Phase 8 — Slide show

- **Gives:** a deck presents as it does in PowerPoint, with presenter view, on web and desktop.
- **Opens:** playback (8A) after 1I and Phase 3; authoring (8B) after Phase 4. Theme editing is not a dependency.
- **In:** timeline, all transitions, Morph, all animation classes, triggers, motion paths, animation pane, presenter view, custom shows, timings, kiosk, narration playback, zoom.
- **Out:** nothing recordable: narration, timings, screen recording and video or GIF export are in 8B ([96](96-POWERPOINT-FIT-GAP.md)).
- **Closes when:**
  - [ ] Deterministic timeline goldens for each timing construct, against the documented semantics in BEH-23.
  - [ ] 60 fps on the animation corpus on a named device.
  - [ ] Presenter sync test; owner sign-off.

## Phase 9 — Collaboration

- **Gives:** people co-edit one deck reliably.
- **Opens:** Phases 4 to 7 op sets stable; read-only audit of OpenCalc's server done.
- **In:** forked server, OT per [24](24-OPERATIONS-AND-COLLABORATION.md), presence, modern comments, version history.
- **Out:** WOPI and hosting product work.
- **Closes when:**
  - [ ] TP1 property tests over every op; a fuzz target on the wire format.
  - [ ] Two real browsers in CI; resume with no lost edit; leader failover tested.
  - [ ] Published load-test numbers on a named environment.
  - [ ] `SEC-S01`, `S02`, `S03`, `S05`, `S06` tested; owner sign-off.

## Phase 10 — Enterprise and hosts

- **Gives:** an enterprise can deploy and support it. **First stable release.**
- **Opens:** Phases 8 and 9 closed.
- **In:** all remaining [10](10-ENTERPRISE-REQUIREMENTS.md) rows, SDK, WOPI, MCP, `.odp`, import of legacy `.ppt` (`PPT-01`), 3D model rendering and editing (`D3-01`), Helm, release process.
- **Closes when:**
  - [ ] Each `ENT-*` row closed with evidence.
  - [ ] Third-party penetration test; findings closed.
  - [ ] WOPI validator passes; SBOM and signed releases published.
  - [ ] **`PV-01` passed: PowerPoint opens every exported corpus deck with no repair prompt, with visual and edit comparison recorded.**
  - [ ] No P0 or P1 open; legal review complete (`DEC-07`); owner sign-off.

## Claims

| Claim | Requires |
| --- | --- |
| "Supported" | A row at `y`, with a test seen to fail without the code |
| "Retained" | Bytes compared on a fixture |
| "Matches PowerPoint" | Compared against a PowerPoint-produced reference, never LibreOffice |

# 06 — Roadmap and delivery

**This is an enterprise product, not an MVP.** The rules for opening and closing are in
[05](05-PHASES-AND-TASKS.md). Each phase below is a charter: what it gives, what opens it, what is in and
out, and what closes it.

**Order follows the owner's priority (2026-11-07):** fidelity is the most critical thing, so import, export and
modelling come first. After Phase 1, work runs as **vertical slices** (owner instruction 44): thin features a user can use, each cut through every layer, then deepened. **Web and desktop are both first-class** (`DEC-06`).

**PowerPoint verification is a release gate, not a phase blocker** (`DEC-04`, `PV-01`). Until it passes, phases close against LibreOffice, schemas and Microsoft's validator, and nothing may claim to match PowerPoint.

**No calendar dates.** Row counts measure surface, not effort. Phase 1's history calibrates throughput: Phase 2's ETA is in hours of work, sized from it (instruction 41).

## Where things stand

| Phase | State | Note |
| --- | --- | --- |
| D Design | **Open** | Blocks nothing structurally. Owner decisions remain |
| 0 Foundation | **Closed** 2026-11-07 | Signed off by the owner. Seeded violations were run locally on macOS, not on GitHub's runners. Fonts measured in `SPK-01` left the repo and return in Phase 2 |
| 1 Model, import, export | **Closed** 2026-11-08 | Signed off by the owner (instruction 53) on the phase1-exit run at `50ea80d`. What each tier still lacks is in [14](14-EXECUTION-TRACKER.md) §Tier gates |
| 2 Layout and render | **Open** under instruction 43's waiver | Its tiers are the render backlog for the slices. Gallery agreement with LibreOffice 26.2.6.3 measured on CI at `1ba6443`: 5.4% of pixels differ on mean over 111 slides (94.6% agree); not a PowerPoint measure |
| Slices 1 to 13 | **None closed**; 1 to 12 built and merged, each open with what is left in its row; 13 not started | Replace Phases 3 to 10; see §After Phase 1, column "Today" |

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
  - [x] Owner sign-off, 2026-11-07 (instruction 21).

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
- [x] All tiers closed.
- [x] Every exported corpus deck passes schema validation and Microsoft's validator, and LibreOffice reopens it. The PowerPoint check is `PV-01`.
- [x] Corpus includes real decks from PowerPoint on Windows and Mac, and Google Slides, with provenance recorded. Keynote was dropped by the owner (instruction 36).
- [x] A 500-slide deck opens within its budget, and a hostile corpus run shows no crash or hang.
- [x] The damage corpus opens, or is refused only under [29](29-RECOVERY.md)'s rules; an edit to a recovered deck writes a valid file, except a part kept as it arrived after a failed checksum, which is reported (owner instruction 34).
- [x] Owner sign-off (2026-11-08).

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
| Display list | `Path`, `GlyphRun`, `Image`, `Group`, `Effect` items as in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md); serializable, golden-tested, same bytes on every platform | Canvas2D painting (slice 1) |
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

## After Phase 1: vertical slices

**What changed, 2026-11-08 (owner instruction 44).** Phases 3 to 10 were layers: finish the viewer, then all of editing, then the slide show, then collaboration. A user saw nothing usable until late, and a problem in one layer surfaced only when the next was built. They are replaced by **vertical slices**: each is a thin feature a user can see and use, cut through model, layout, render, UI and save at once, then deepened in later passes. Phase 2's tiers and targets stay as the render backlog that slices draw from. **Nothing about fidelity changes:** absent stays absent, edits patch named nodes (ADR-013), nothing is lost silently, untrusted input never panics, and no claim of a PowerPoint match before `PV-01`. Every exit criterion of the old Phases 3 to 10 is kept: each is moved to the slice or pass named below, never dropped.

**How a slice runs.**
- **Outcome:** one sentence a user would say, shown on a real deck from `fixtures/real`.
- **End point:** merged to `main`; its named tests green on GitHub CI, on web and desktop where both apply; the gallery (`FH-02`) or a CI browser run shows it working; tracker, checklist and docs true. Then the owner signs it; only the owner closes a slice.
- **Time box** is in session-hours at Phase 2's pace. At the box the session merges what passes, files a row for the rest and stops.
- **Reuse (instruction 42):** each starts from OpenCalc's working code, copied with provenance, and names what it copies. Building what OpenCalc has needs a reason in its row.
- **Thin first.** A slice does the common case end to end and reports what it does not do. Depth comes in later passes, each its own row with its own end point.

| # | Slice | User can | End point | Box | Copies from OpenCalc | Today (at `1ba6443`) |
| --- | --- | --- | --- | ---: | --- | --- |
| 1 | **Web viewer** | Open a `.pptx` in the browser and page through its slides, with thumbnails and zoom | Every `fixtures/real` deck opens and draws in a CI browser run; text, shapes and pictures drawn; anything not drawn is reported | 4 | `casual-calc-wasm` bridge, `webapp/` shell (`index.html`, `editor.core.js`, `file.worker.js`, `serve.py`), CSP, fonts | Built, live at the Pages `editor.html` (`VS-01`, `SITE-01`). Left: a CI browser run over every `fixtures/real` deck; engine size (`T2D1-02`) |
| 2 | **Tables, charts, effects in render** | See tables, charts and shadows as the deck has them | `T2E-04`, `T2F-01`, `T2H-01`, `T2C-01` gates met; R1 measured on their decks | 8 | `casual-calc-render` chart drawing, `casual-calc-layout`, `-text` | In progress (`VS-02`): tables, classic charts, SmartArt and OLE fallbacks, shadow and glow drawn (`SKEL-03`, `FH-03`, `FH-05`); `T2E-04`, `T2F-01`, `T2H-01`, `T2C-01` gates not met; what is left in `SKEL-03`, `SKEL-04`, `FH-04` |
| 3 | **Desktop window** | Open the same deck in a native window by double-click | Tauri bundles built on three OSs in CI; desktop and web draw the same | 2 | `desktop/` Tauri shell, file association, `casual-calc-host` | Built (`VS-03`); `desktop.yml` bundled on three OSs green at `50ea80d`. Left: `DESK-01`, `DESK-02` |
| 4 | **Save and export** | Download the deck unchanged, or as PDF | Unedited save byte-identical; PDF has real text (`T2G-01`); desktop Save works | 3 | `editor.files.js`, `download`, PDF writer and subsetter | Done (`VS-04`), for the owner to sign |
| 5 | **Edit text** | Click a text box, type, delete, bold, undo, save | Typed text round-trips and opens clean in the SDK validator; only the edited node changes; IME in one browser | 6 | `casual-calc-transaction` (undo), `casual-calc-selection`, `commands.js`, `editor.clipboard.js` | Built (`VS-05`, `VS-17`, `VS-19`); left in `VS-15`: CI browser run, IME in one browser |
| 6 | **Move and resize shapes** | Select, drag, resize, rotate, delete, undo | Op set inverse-tested; transform round-trips; drag budget on the adversarial slide | 4 | `editor.selection.js`, `editor.geometry.js`, transaction | Built (`VS-06`); multi-select from `UX-003`; left in `VS-14`: drag budget, CI browser run |
| 7 | **Present** | Play the deck full screen with keyboard, and a presenter view | Slide show on web and desktop; notes shown; transitions drawn as a simple fade and the rest reported | 3 | `commands.js` key handling, `editor.core.js` view switching | Built (`VS-07`); left in `VS-16`: run on web and desktop in CI, automatic advance, `showPr` |
| 8 | **Build slides** | Add, duplicate, reorder and delete slides; insert a shape, text box or picture | Each op inverse-tested and round-trips; autosave and recovery lose nothing (`SAVE-01`) | 5 | `editor.sheets.js` (tab ops pattern), `editor.drafts.js` (autosave), ribbon | Built (`VS-08`); autosave and recovery built (`SAVE-01`, in progress); left: Set Layout, CI browser run, `SAVE-09` |
| 9 | **Collaborate** | Two people edit one deck and see each other | Two browsers in CI converge; resume with no lost edit | 6 | `server/casual-calc-collab-server`, `collab.js`, `editor.presence.js`, `editor.versions.js` | Built: server and browser (`VS-09`, `COL-04`, ADR-020). Left: resume (`COL-05`), tokens (`COL-03`), more ops on the wire (`COL-07`), `COL-02`, `COL-08`; the CI two-page run is red on `main` (`CI-23`) |
| 10 | **Animate** | See entrance, exit and emphasis animations and transitions play | Timeline goldens for the common classes; 60 fps on a named device | 5 | — (new; OpenCalc has no timeline) | Built (`VS-10`): six transitions and six effects play. Left: effect options, Duration and Delay, triggers, timeline goldens, 60 fps |
| 11 | **Edit rich objects** | Edit a table's cells and a chart's data | Edits round-trip; chart data in an embedded OpenCalc pane (`DEC-15`) | 6 | `casual-calc-sdk`, `editor.charts.js`, embed | Built (`VS-11`): tables and four chart types inserted and edited; left in `VS-20`: typing in a cell through the caret engine, merge and split, OpenCalc pane (`DEC-15`) |
| 12 | **Design** | Change theme, background, slide size; edit a master | Master edit propagates and round-trips | 4 | `ribbon.model.js` galleries | Built (`VS-12`): themes, variants, slide size, background. Left in `DSN-01`: master editing, the slice's gate |
| 13 | **Hosts** | Run it under a company's file store and AI tools | WOPI validator passes; SDK and MCP published | 5 | `server/casual-calc-wopi`, `casual-calc-mcp`, `sdk/` | Not started (`VS-13`) |

Slices 1 to 12 were built on 2026-11-08 and 2026-11-09, at most three sessions at once (rule 22); none has met its end point in full, so none is closed, and each row in [14](14-EXECUTION-TRACKER.md) names what is left. Beside them: the PowerPoint for the web shell (`UX-003`), the shapes gallery (`SHAPE-01`), the File tab (`FILE-01`), the site (`SITE-01`, `SITE-03`) and the parity audit (`FIT-02`, [96](96-POWERPOINT-FIT-GAP.md) §Parity today). **About 61 session-hours** in all, before deepening.

**Deepening passes.** After a slice closes, each pass adds depth to it and has its own row and box: more scripts and RTL for text editing; full Format panes; all transitions, Morph and motion paths; SmartArt and equation editing; ink; media; reading and sorter views; accessibility checker; and so on, from [96](96-POWERPOINT-FIT-GAP.md). Passes are ordered by how many real decks they affect.

**Where the old phase gates went.** Each is now a gate on the slice or pass named.

| Old gate | Now on |
| --- | --- |
| Painter matches the PNG backend; web and desktop agree; browser smoke in CI | Slices 1 and 3 |
| 500-slide budget in the viewer | Slice 1 pass |
| WCAG 2.2 AA audit, screen reader, keyboard-only; XSS and link threats `SEC-E01`, `E03`, `N03` | Every slice for its own verbs; full audit at the release gate |
| Op set inverse-tested, TP1 for every op (`OP-02`), task test with PowerPoint users | Slices 5, 6, 8, 11, 12, for their ops |
| IME matrix, RTL and CJK, typing budget | Slice 5 passes |
| Timeline goldens, 60 fps, presenter sync | Slices 7 and 10 |
| TP1 property tests, wire fuzz target, leader failover, load numbers, `SEC-S01` to `S06` | Slice 9 and its passes |
| `ENT-*` rows, penetration test, SBOM, signed releases, `PV-01`, legal review (`DEC-07`) | **Release gate**, before the first stable release, unchanged |

**Release gate.** Unchanged from the old Phase 10: every `ENT-*` row closed with evidence; third-party penetration test; WOPI validator; SBOM and signed releases; **`PV-01` passed**; no P0 or P1 open; legal review; owner sign-off.

## Claims

| Claim | Requires |
| --- | --- |
| "Supported" | A row at `y`, with a test seen to fail without the code |
| "Retained" | Bytes compared on a fixture |
| "Matches PowerPoint" | Compared against a PowerPoint-produced reference, never LibreOffice |

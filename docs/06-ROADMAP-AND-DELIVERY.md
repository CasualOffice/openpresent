# 06 — Roadmap and delivery

**This is an enterprise product, not an MVP.** The rules for opening and closing are in
[05](05-PHASES-AND-TASKS.md). Each phase below is a charter: what it gives, what opens it, what is in and
out, and what closes it.

**Order follows the owner's priority (2026-11-07):** fidelity is the most critical thing, so import, export and
modelling come first, then layout and rendering, then the editor. **Web and desktop are both first-class** (`DEC-06`).

**PowerPoint verification is a release gate, not a phase blocker** (`DEC-04`, `PV-01`). Until it passes, phases close against LibreOffice, schemas and Microsoft's validator, and nothing may claim to match PowerPoint.

**No dates.** Row counts measure surface, not effort. Tiers 1A to 1D calibrate throughput, and the rest is sized from that.

## Where things stand

| Phase | State | Note |
| --- | --- | --- |
| D Design | **Open** | Blocks nothing structurally. Owner decisions remain |
| 0 Foundation | **Closed** 2026-11-07 | Signed off by the owner. Seeded violations were run locally on macOS, not on GitHub's runners. Fonts measured in `SPK-01` left the repo and return in Phase 2 |
| 1 Model, import, export | **Open** | See the Phase 1 plan below |
| 2 to 10 | Not opened | **No rendering, painting or font work until Phase 1 closes** (owner instruction 20) |

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

**Where it stands (2026-11-08).** Every tier is built and merged; **1A, 1D, 1E and 1F are closed** (2026-11-08), the rest are not. Each tier's gate, item by item, and what it still lacks are in [14](14-EXECUTION-TRACKER.md) §Tier gates. None of the Phase 1 exit criteria below is met yet. Status columns are filled by `tools/spec-audit`; a row needs code, a test and a fixture.

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
- [ ] Corpus includes real decks from PowerPoint on Windows and Mac, Keynote and Google Slides, with provenance recorded.
- [ ] A 500-slide deck opens within its budget, and a hostile corpus run shows no crash or hang.
- [ ] The damage corpus opens, or is refused only under [29](29-RECOVERY.md)'s rules; an edit to a recovered deck writes a valid file, except a part kept as it arrived after a failed checksum, which is reported (owner instruction 34).
- [ ] Owner sign-off.

## Phase 2 — Layout and render

- **Gives:** any deck renders headlessly to PNG and PDF, with fidelity measured, before a UI can hide problems.
- **Opens:** 1D and the tier feeding each sub-tier closed; ADR-004 and ADR-005 settled.
- **In:** tiers below; behaviours BEH-05 to 20. **Out:** UI, canvas, editing, animation playback.

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
| **2J** | SmartArt: from its drawing, then the layout engine for decks with none (`DGM-01`) | 1K |
| **2K** | Equations: math layout | 1K |
| **2G** | PDF with tags, reading order, alt text, links; JPEG; print layouts | 2A to 2K |

**Closes when:**
- [ ] Each corpus deck is within its recorded threshold against its reference: a PowerPoint-exported PDF or image where the deck ships with one, else LibreOffice, labelled as such.
- [ ] Each behaviour row has a passing fixture.
- [ ] Budgets in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) replaced with measurements and asserted.
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

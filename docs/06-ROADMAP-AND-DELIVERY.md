# 06 — Roadmap and delivery

**This is an enterprise product, not an MVP.** There is no "good enough for a demo"
milestone. A phase closes on its exit gate, never because code exists, and it does not
close with a checklist row unaccounted for.

**Order follows the owner's priority (2026-11-07):** fidelity is the most critical thing, so
import, export and modelling come first, then layout and rendering, and only then the editor.
An editor over a lossy engine would have to be rebuilt; the reverse never needs to happen.

```
 0 Foundation ─▶ 1 Model, import, export ─▶ 2 Layout and render ─▶ 3 Viewer
                                                                    │
      10 Enterprise and hosts ◀─ 9 Collaboration ◀─ 8 Slide show ◀─ 4–7 Editing
```

Security ([07](07-SECURITY-THREAT-MODEL.md)) and enterprise requirements
([10](10-ENTERPRISE-REQUIREMENTS.md)) are not a phase at the end. Each phase's gate includes them.

**No dates yet.** Row counts below measure surface, not effort, and an estimate today would be
invented. Tiers 1A to 1D are the calibration: after them, the remaining tiers can be sized from
measured throughput.

## Phase 0 — Foundation

| Deliver | Gate |
| --- | --- |
| Workspace, `-package`, `-text` copied from OpenCalc | Done locally; see `F-001`, `F-002` |
| Spec checklist generated from the schemas | Done; `--check` fails on staleness or an unassigned type |
| Package threats closed | Done for `SEC-P01` to `SEC-P07`; each test seen red |
| CI, `check-*` gates, pre-commit hook, `cargo deny`, `cargo audit` | Green on three platforms; a seeded violation fails each gate |
| Fuzz targets ported from OpenCalc | Campaign with no crash |
| Fixture corpus, with rights review and checksums | Manifest gate; see [51](51-SPEC-CHECKLIST-AND-FIDELITY.md) §Corpus |
| Fidelity oracle pipeline (`SPK-02`) | 5 decks end to end |
| WASM text cost (`SPK-01`) | Numbers recorded; ADR-004 and ADR-005 settled |

## Phase 1 — Model, import, export

**The priority.** Everything in a `.pptx` is read into the model or retained, and written back.
Nothing is drawn yet except by the debug dump. Tiers are ordered by dependency; rows are from
[spec/README.md](spec/README.md).

| Tier | Scope | Rows | Needs |
| --- | --- | ---: | --- |
| **1A** | OPC: content types, relationships, part graph, core/app/custom properties, thumbnails, signatures (retained), shared value types, Strict and Transitional namespaces. Byte-identical repackager | 392 | — |
| **1B** | Presentation part: slide and master lists, sizes, defaults, sections-ready lists, embedded fonts, custom shows, show and print properties, saved view state | 297 | 1A |
| **1C** | Themes, 3 colour models, all colour transforms, colour maps and overrides, format schemes, object defaults | 473 | 1A |
| **1D** | Masters, layouts, slides, notes, handouts, backgrounds, placeholders, shape tree, groups, connectors, locks, links, stable ids. `-style` inheritance resolver | 564 | 1B, 1C |
| **1E** | Geometry (187 presets, custom paths, guides), fills, lines, effects, 3D properties | 1082 | 1D |
| **1F** | Text bodies, paragraphs, runs, fields, lists, tabs, autofit properties | 317 | 1D |
| **1G** | Tables and table styles | 113 | 1F |
| **1H** | Pictures, SVG, crops, audio, video, external media references | 116 | 1E |
| **1I** | Timing tree, transitions (base and extension), Morph, build lists | 522 | 1D |
| **1J** | Charts, chart styles, 2016+ chart types, embedded workbooks retained | 1764 | 1E, 1F |
| **1K** | SmartArt data and drawing parts, equations, ink | 1263 | 1E, 1F |
| **1L** | OLE, ActiveX, VML, 3D models: retained with fallback images | 951 | 1H |
| **1M** | Comments legacy and modern, change tracking, sections, zoom, classification, tags, custom XML | 876 | 1D |
| | **Total** | **8730** | |

**Tiers 1E to 1M can run in parallel** once 1D holds, since each owns a separate area of the model.

**Per-tier gate, every tier:**
1. Every row in scope is `y` or `r` for model, import and export, or carries a named reason.
2. Unedited save is byte-identical. A one-edit save leaves all other parts byte-identical.
3. Import → export → import is a fixed point.
4. Exported XML validates against the vendored schemas.
5. The compatibility report names everything not modelled. A silent drop fails CI.
6. New limits have hostile fixtures; a fuzz target covers each new parser.

**Phase 1 exit:** the corpus passes all of the above, and PowerPoint reopens every exported deck
without a repair prompt. That last check needs PowerPoint (`DEC-04`), and the phase cannot close without it.

## Phase 2 — Layout and render

Headless `deck → PNG` and `deck → PDF`. Fidelity is measured here, per
[51](51-SPEC-CHECKLIST-AND-FIDELITY.md), before any UI hides it.

| Tier | Scope | Needs |
| --- | --- | --- |
| **2A** | Slide size, master and layout composition, background, theme resolution | 1D |
| **2B** | All 187 presets, custom paths, fills, lines, arrowheads, group transforms, flips | 1E |
| **2C** | Shadow, glow, soft edge, reflection, blur, blip effects, flattened 3D | 1E |
| **2D** | **The text engine:** fonts and fallback, shaping, bidi, line breaking, CJK rules, spacing, tabs, bullets, columns, vertical text, autofit, WordArt warps | 1F |
| **2E** | Tables, pictures, crops, SVG, poster frames | 1G, 1H |
| **2F** | Charts, 2016+ charts, SmartArt from its drawing, equations, ink, fallback images | 1J–1L |
| **2G** | PDF: real text, subset fonts, links, tagged structure; JPEG per slide; print layouts for slides, notes, handouts | 2A–2F |

**Exit:** every corpus deck is within its pixel threshold, every behaviour row in 51 has a passing
fixture, and the budgets in [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) hold.

## Phase 3 — Viewer

WASM bridge; Canvas2D painter matching the PNG backend within threshold; shell, thumbnail rail,
status bar, zoom; Normal, Sorter, Reading views; notes pane; accessible DOM of the slide; CSP.
**Exit:** browser smoke on a real deck; painter-versus-PNG gate; 500-slide budget; WCAG 2.2 AA audit of the shell.

## Phases 4 to 7 — Editing

| Phase | Scope | Exit |
| --- | --- | --- |
| **4 Objects** | Op set and undo; select, move, resize, rotate, adjust handles; smart guides; group; arrange; selection pane; insert shape and picture; slide operations; sections; format panes; ribbon Home, Insert, Shape Format, Picture Format; context menus | Every op inverse-tested; drag budget on the adversarial slide; UI status filled for every in-scope row |
| **5 Text** | Caret, selection, IME, rich formatting, lists, autofit on edit, find and replace, spell-check seam, links | IME matrix; RTL and CJK fixtures; typing budget |
| **6 Rich objects** | Tables including merge and split; charts with data editing through the OpenCalc SDK; SmartArt; equations; WordArt; audio and video with trim; ink | Round-trip per construct after editing |
| **7 Design** | Themes, variants, background, slide size, **master and layout editing** | Master edit propagates and round-trips; theme swap fixtures |

## Phase 8 — Slide show

Timeline compiler; all transitions in the checklist including the extension set and Morph; all
animation classes, triggers, motion paths; animation pane; presenter view; custom shows;
rehearsed timings; self-running and kiosk; narration playback; zoom objects. **Exit:** deterministic
timeline goldens; 60 fps on the animation corpus; presenter sync test.

## Phase 9 — Collaboration

Forked server behind an engine trait, opening with a read-only audit; OT per
[24](24-OPERATIONS-AND-COLLABORATION.md); presence; modern comments, mentions, reactions, tasks;
version history; revision highlighting. **Exit:** two real browsers in CI; TP1 property tests; resume with no lost edit; load test.

## Phase 10 — Enterprise and hosts

Everything in [10](10-ENTERPRISE-REQUIREMENTS.md) not already delivered: SSO, audit logs, sensitivity
labels, encrypted files, WOPI, admin, observability, Helm; `<openslide-deck>` SDK; desktop; MCP;
`.odp`; legacy `.ppt` if `DEC-06` says so. **Exit:** third-party penetration test; WOPI validator; published SBOM; first stable release.

## What a phase closing means

| Claim | Requires |
| --- | --- |
| "Supported" | The row is `y` with a test seen to fail without the code |
| "Retained" | Bytes compared on a fixture, not assumed |
| "Matches PowerPoint" | Compared against a PowerPoint-produced reference, not against LibreOffice |

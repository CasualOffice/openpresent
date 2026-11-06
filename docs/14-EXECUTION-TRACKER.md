# 14 — Execution tracker

**The only tracker.** Every unit of work has one row. Ids are stable and never
reused. Status: `Open`, `In progress`, `Blocked`, `Done`.

## Owner decisions

| ID | Decision | St | Recommendation | Why it matters |
| --- | --- | --- | --- | --- |
| DEC-01 | Product name | Open | Rename before anything is published | "OpenSlide" is an established digital-pathology library and "OpenSlides" an established assembly-management product. Crate prefix `casual-slide-` is unaffected |
| DEC-02 | Where to build, and how to share code with OpenCalc | Done | Owner, 2026-11-07: separate repo in this folder; copy from OpenCalc and reuse what fits | Each copied crate carries `PROVENANCE.md` |
| DEC-03 | Accent colour | Open | Rose `#b5336b` (5.75:1 on white) | Must be deliberately not PowerPoint's orange-red, as OpenCalc's green is not Excel's. Taste call |
| DEC-04 | Access to a signed-in PowerPoint for the web, and PowerPoint-rendered reference images | Open | Provide a signed-in Chrome session and, if possible, a desktop PowerPoint for references | [47](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md) is recalled from memory. OpenCalc's shell was measured; this one should be too. LibreOffice alone is a second opinion, not the target |
| DEC-05 | Build our own Designer and Copilot equivalents? | Open | Not now; keep the seam | Large, separate product decision |
| DEC-06 | Feature bar where web and desktop differ | Done | Owner, 2026-11-07: **both web and desktop, as OpenCalc.** Full desktop feature bar, web and Tauri desktop hosts. Legacy `.ppt` is `DEC-08` | [48](48-POWERPOINT-FEATURE-STUDY.md) |
| DEC-08 | Read legacy `.ppt`? | Open | Decide after Phase 2. Large binary format; refused cleanly for now as `CS-OPC-0005` | Microsoft's web converts it |
| DEC-07 | Legal review of bundled schemas, fonts, and Microsoft's patent notice | Open | Have counsel read [schemas/README](../schemas/README.md) before the first release | Enterprise buyers will ask; ENT-24 |

## Phase D

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| DOC-001 | Initial design set | Done | P0 | Written 2026-11-07 from a scan of `../opencalc` at `b23c67c` | Owner review, below |
| DOC-002 | Owner review of the remaining ADRs | Open | P0 | ADR-003 Accepted; the rest Proposed | ADRs Accepted or changed |
| UX-001 | Measure PowerPoint for the web and turn 47 into a measured target | Blocked | P1 | Needs `DEC-04` | Shell prototype approved by owner, as OpenCalc's docs/95 was |

## Phase 0

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| F-001 | Cargo workspace, toolchain 1.96.0 and MSRV matching OpenCalc | Done | P0 | [19](19-WORKSPACE-SCAFFOLD-DESIGN.md). MSRV 1.88.0 is declared but not yet run on that toolchain; F-003 gates it | fmt, clippy `-D warnings`, doc and wasm32 check pass locally |
| F-002 | Fork `-package` and `-text` with provenance | Done | P0 | Copied at `b23c67c` and renamed; `webapp/fonts/cjk` copied because `-text` tests read it. `-secrets` waits for its first consumer in Phase 9 | 12 and 40 tests pass. Disabling the traversal and expansion-ratio guards turned 4 package tests red; restored and green |
| F-003 | CI matrix and repo gates | In progress | P0 | `.github/workflows/ci.yml`, `tools/check-repo.py`, pre-commit hook written. **Has never run: there is no remote** | Green on three platforms |
| F-004 | Fixture corpus with checksums | In progress | P0 | 43 fixtures: generated, hostile, LibreOffice-written. No PowerPoint-written deck yet (`DEC-04`) | Manifest SHA-256 gate passes |
| F-005 | Fuzz targets for admission | Done | P1 | `bounded_package`, `presentation_package`, `xml_walker`; 505k, 968k and 5.4M runs, 46 s each, no crash. Short runs; longer campaigns belong in scheduled CI | No artifact |
| ENV-01 | LibreOffice unusable here | Done | P2 | `/opt/homebrew/bin/soffice` points at `/Applications/LibreOffice.app`, which is gone, though the cask is still recorded. Earlier docs said `soffice` was available; that was wrong | Reinstalled 26.8.2.2; `soffice --version` runs |
| SPEC-01 | Audit tool fills checklist status from the code | Open | P1 | Port `../opencalc/tools/fidelity-audit` | A seeded gap is reported |
| SPEC-02 | Read MS-OI29500 notes per tier | Open | P1 | Where Office departs from the standard | Notes recorded before each tier opens |
| SPEC-03 | Independent count check in CI | Open | P1 | Done by hand 2026-11-07; found 74 missing global attributes | Generator counts compared with raw XSD |
| SPEC-04 | Enumerate extension parts and relationship types | Open | P1 | Not in [spec/parts.md](spec/parts.md) | Listed from MS-PPTX and MS-ODRAWXML |
| SPEC-05 | Checklist generated | Done | P0 | 8730 rows from 77 schemas; an unassigned type fails generation | Staleness and unassigned-namespace failures both seen |
| PKG-01 | Read budget is cumulative | Open | P2 | `read_part` charges a whole-package budget on every read, so an editor re-reading parts can exhaust it on a legitimate deck | A long session does not refuse |
| GEO-01 | `upArrow` missing from Ecma's preset definitions | Open | P1 | The file defines `upDownArrow` twice and `upArrow` never. A generator keyed on names ships 186 of 187 | A fixture draws all 187; `upArrow` matches a reference |
| SEC-010 | Package threats SEC-P04 to SEC-P07 | Done | P1 | Oversized allocation from a forged size; duplicate and case-equivalent names; malformed names. The `zip` crate merges duplicates, so a raw directory walk was added. Inherited from OpenCalc | Each test seen red; guards disabled and seen red again; 19 tests pass |
| SEC-011 | Report package findings to OpenCalc | Open | P1 | Same code at `b23c67c`; that repo is not modified from here | Owner informed |
| SEC-013 | Zip bomb dilution, XML read before size check | Done | P1 | Found by the corpus: 64 MiB of zeros admitted because other parts diluted the ratio | Tests seen red; `SEC-P12`, `SEC-P13` |
| SEC-012 | Remaining threats in [07](07-SECURITY-THREAT-MODEL.md) | Open | P0 | One row per `SEC-*`, closed by the tier named there | Hostile fixture per threat |
| ENT-000 | Requirements in [10](10-ENTERPRISE-REQUIREMENTS.md) | Open | P0 | Gate on each delivering phase | Per requirement |
| SPK-01 | WASM shaping and glyph-path frame time | Open | P0 | [06](06-ROADMAP-AND-DELIVERY.md) | Numbers in 42 |
| SPK-02 | Fidelity oracle pipeline | In progress | P0 | LibreOffice half works: decks convert to PDF and `.pptx`. PowerPoint references need `DEC-04` | 5 decks end to end |
| SPK-03 | Preset shape definition licensing | Done | P0 | Ecma's notice permits use in conformant implementations; vendored unmodified. Formal review is `DEC-07` | Recorded in [schemas/README](../schemas/README.md) |

## Phases 1 to 10

Each tier and phase in [06](06-ROADMAP-AND-DELIVERY.md) is one row, split when opened. Scope per row is [spec/](spec/README.md).

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| T-1A | Package, part graph, repackager | In progress | P0 | Structure, findings and repackager built ([28](28-PACKAGE-AND-PART-GRAPH.md)). Properties parts not modelled yet | Per-tier gate in 06 |
| T-1B | Presentation part, view state | Open | P0 | 297 rows | Per-tier gate |
| T-1C | Colour and theme | Open | P0 | 473 rows | Per-tier gate |
| T-1D | Slides, masters, layouts, shape tree, `-style` | Open | P0 | 564 rows | Per-tier gate |
| T-1E | Geometry, fills, lines, effects, 3D | Open | P0 | 1082 rows | Per-tier gate |
| T-1F | Text model | Open | P0 | 317 rows | Per-tier gate |
| T-1G | Tables | Open | P0 | 113 rows | Per-tier gate |
| T-1H | Pictures and media | Open | P0 | 116 rows | Per-tier gate |
| T-1I | Timing and transitions | Open | P0 | 522 rows | Per-tier gate |
| T-1J | Charts | Open | P0 | 1764 rows | Per-tier gate |
| T-1K | SmartArt, equations, ink | Open | P0 | 1263 rows | Per-tier gate |
| T-1L | OLE, VML, 3D retained | Open | P0 | 951 rows | Per-tier gate |
| T-1M | Comments, review, sections, zoom, labels | Open | P0 | 876 rows | Per-tier gate |
| T-2A | Composition | Open | P0 | Render | 51 ladder |
| T-2B | Shapes | Open | P0 | Render | 51 ladder |
| T-2C | Effects | Open | P1 | Render | 51 ladder |
| T-2D | Text engine | Open | P0 | Render | 51 ladder |
| T-2E | Tables, pictures | Open | P0 | Render | 51 ladder |
| T-2F | Charts, SmartArt, equations, ink | Open | P0 | Render | 51 ladder |
| T-2G | PDF, JPEG, print | Open | P0 | Render | 51 ladder |
| P-3 | Viewer | Open | P0 | 06 | Exit gate |
| P-4 | Object editing | Open | P0 | 06 | Exit gate |
| P-5 | Text editing | Open | P0 | 06 | Exit gate |
| P-6 | Rich object editing | Open | P0 | 06 | Exit gate |
| P-7 | Design surface | Open | P0 | 06 | Exit gate |
| P-8 | Slide show | Open | P0 | 06 | Exit gate |
| P-9 | Collaboration | Open | P0 | 06 | Exit gate |
| P-10 | Enterprise and hosts | Open | P0 | 06 | Exit gate |

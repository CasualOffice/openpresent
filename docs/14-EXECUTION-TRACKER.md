# 14 — Execution tracker

**The only tracker.** Ids are stable and never reused. Status: `Open`, `In progress`,
`Blocked`, `Done`, as defined with Definition of Done in [05](05-PHASES-AND-TASKS.md). Phase state and waivers are in [06](06-ROADMAP-AND-DELIVERY.md).

## Owner decisions

| ID | Decision | St | Recommendation | Why it matters |
| --- | --- | --- | --- | --- |
| DEC-01 | Product name | Done | **OpenPresent**, decided on the owner's delegation, 2026-11-07 | `OpenSlide` is a digital-pathology library and `OpenSlides` an assembly product. One search found no project called OpenPresent, which is **not** trademark clearance (`DEC-07`). Fits OpenDoc and OpenCalc. Cheap to change until first public release. Crates stay `casual-slide-*`; rename is `NAME-01` |
| DEC-02 | Where to build, and how to share code with OpenCalc | Done | Owner, 2026-11-07: separate repo in this folder; copy from OpenCalc and reuse what fits | Each copied crate carries `PROVENANCE.md` |
| DEC-03 | Accent colour | Done | Owner, 2026-11-07: **cyan.** `#0e7490`, the shade chosen for 5.36:1 contrast on white, since brighter cyans fail WCAG AA for text | Distinct from PowerPoint's orange-red and OpenCalc's green. One token |
| DEC-04 | PowerPoint verification | Done | Owner, 2026-11-07: LibreOffice is the working reference, and UI and UX come from online research. **PowerPoint verification moves from Phases 0 to 3 to a release gate** (`PV-01`), run earlier whenever access exists | So work never waits on it. Risk: repair prompts and visual differences found late. Mitigated by schema validation, Microsoft's validator (`F-006`) and real PowerPoint-written decks as input (`F-007`). "Matches PowerPoint" stays unclaimable until then |
| DEC-05 | Designer and Copilot equivalents | Done | Not built now; a host seam keeps the door open; their file metadata is retained | Needs an AI service, against the air-gapped default. Cheap to add later, and costly to design around now |
| DEC-06 | Feature bar and hosts | Done | Owner, 2026-11-07: **both web and desktop, as OpenCalc.** The goal is a clone, so the full desktop feature set, in both hosts | [48](48-POWERPOINT-FEATURE-STUDY.md) |
| DEC-08 | Legacy `.ppt` | Done | **Yes, import-only, in Phase 10** (`PPT-01`), saved as `.pptx`. Until then refused cleanly as `CS-OPC-0005` | Enterprises hold years of these. A bounded binary parser needs its own threat work, after the model is stable. The model needs no change for it |
| DEC-09 | Remote for CI | Done | Owner created private `CasualOffice/openpresent` and pushed, 2026-11-07. `main` and PR #1 added so CI runs | — |
| DEC-10 | Default ribbon | Done | **Classic, with Simplified as a toggle** | Matches OpenCalc's owner-approved shell, and the full desktop feature bar. Microsoft's guide advises Simplified off for keyboard work. Whether the web defaults to Simplified is unconfirmed (`UX-002`); a preference flips it cheaply |
| DEC-11 | Inserting a shape | Done | **Click drops a default-size shape; drag draws one** | Serves both the web's and desktop's habit, with no collision |
| DEC-12 | `Ctrl+Shift+V` | Done | **By selection level:** with a caret, plain paste, as web; with an object, apply format, as desktop | Each keeps its own users' habit. `Ctrl+[` and `]` change layer as on the web, and font size is `Ctrl+Shift+>` and `<` |
| DEC-13 | Copilot slot in the shell | Done | **None in our chrome.** A host may register a command or pane through the seam | Follows `DEC-05`. No dead space on air-gapped deployments |
| DEC-14 | Presenter View target | Done | **The redesigned version on Microsoft's roadmap for November 2026** | A clone should target where the product is going, as we would otherwise build the old one twice. Known only from a roadmap digest, so it is re-checked when Phase 8 opens |
| DEC-15 | Chart data editing | Done | **OpenCalc embedded in a pane, in the same window** | Works identically on web and desktop, and avoids popup blockers. Depends on `CROSS-01` |
| DEC-07 | Legal review of bundled schemas and fonts, Microsoft's patent notice, and the name | Open | Needs a lawyer, so it is the owner's. **Blocks first public release only** | Enterprise buyers will ask; ENT-24 |

## Phase D

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| DOC-001 | Initial design set | Done | P0 | Written 2026-11-07 from a scan of `../opencalc` at `b23c67c` | Owner review, below |
| DOC-003 | Fix doc audit findings | In progress | P0 | Independent audit found 51 issues, with 18 of 31 guards confirmed by mutation. Doc corrections applied 2026-11-07, except rows below. Needs a re-check after the merges | Fresh re-audit finds no P0 |
| DOC-004 | Disposition gap analysis | Done | P0 | 51 gaps reviewed; each has a row below or was applied. Items its author marked unverified stay so in their rows | Rows exist |
| DOC-005 | Phase and task rules | Done | P0 | [05](05-PHASES-AND-TASKS.md), charters in [06](06-ROADMAP-AND-DELIVERY.md). Used to close Phase 0 | Owner signed Phase 0 |
| DOC-006 | Working rules and skills | Done | P1 | `CLAUDE.md` holds standards, one-roadmap-one-tracker, document updates. Six project skills in `.claude/skills`. `check-repo.py` refuses a second tracker or plan | Seeded second tracker seen failing |
| ENG-01 | Enforce standards by lint | Open | P1 | Add `unwrap_used`, `expect_used`, `indexing_slicing`, `missing_docs` for non-test code. Waits for the parallel sessions to merge, since it touches their files. Copied `-text` needs a pass | Clippy passes with the lints on |
| DOC-002 | ADR review | Done | P0 | All Accepted except ADR-004 and ADR-005, which wait on `SPK-01` | [08](08-ADR-REGISTER.md) |
| UX-001 | PowerPoint UI and UX study | In progress | P1 | [49](49-POWERPOINT-COMMAND-INVENTORY.md) done: desktop commands from Microsoft's control-ID workbook, Current Channel 2402, so 32 months old; web mostly unknown (1470 of 1855 rows `?`). [50](50-POWERPOINT-UX-STUDY.md) done, with 24 shell rules; nothing first-hand, and every pixel figure an estimate. [52](52-COMPETITOR-STUDY.md) done. Pixel sizes stay unmeasured without a live session, so geometry starts from OpenCalc's measured ribbon | Shell prototype approved by owner |

## Phase 0

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| F-001 | Cargo workspace, toolchain 1.96.0 and MSRV matching OpenCalc | Done | P0 | [19](19-WORKSPACE-SCAFFOLD-DESIGN.md). MSRV 1.88.0 check passes locally and in CI | fmt, clippy `-D warnings`, doc and wasm32 check pass locally |
| F-002 | Fork `-package` and `-text` with provenance | Done | P0 | Copied at `b23c67c` and renamed; `webapp/fonts/cjk` copied because `-text` tests read it. `-secrets` waits for its first consumer in Phase 9 | 12 and 40 tests pass. Disabling the traversal and expansion-ratio guards turned 4 package tests red; restored and green |
| F-003 | CI matrix and repo gates | Done | P0 | All 9 jobs green on `main` at `ec7e3d8`, on Linux, macOS and Windows. Seeded violations seen failing each job locally on macOS ([15](15-CI-AND-RELEASE-GATES.md)) | Green on `main` |
| F-004 | Fixture corpus with checksums | Done | P0 | 110 fixtures in the manifest, with provenance, including 12 real decks | Manifest SHA-256 gate |
| F-005 | Fuzz targets for admission | Done | P1 | `bounded_package`, `presentation_package`, `xml_walker`; 505k, 968k and 5.4M runs, 46 s each, no crash. Short runs; longer campaigns belong in scheduled CI | No artifact |
| ENV-01 | LibreOffice unusable here | Done | P2 | `/opt/homebrew/bin/soffice` points at `/Applications/LibreOffice.app`, which is gone, though the cask is still recorded. Earlier docs said `soffice` was available; that was wrong | Reinstalled 26.8.2.2; `soffice --version` runs |
| SPEC-01 | Audit reports checklist coverage from code and fixtures | Done | P1 | `tools/spec-audit/audit.py`. Fixtures exercise 1822 of 8730 rows; code 0, since import and export are stubs on this base. Reports rather than fills status columns; reads source text, not behaviour, and owner detection is untested on a real importer | Planted `y` row seen failing by main session; in CI |
| SPEC-02 | Read MS-OI29500 notes per tier | Open | P1 | Where Office departs from the standard | Notes recorded before each tier opens |
| SPEC-03 | Independent count check | Done | P1 | `tools/spec-audit/count.py`: 8730 rows, 0 mismatches | Tests see a dropped row and an added attribute fail; in CI |
| SPEC-04 | Enumerate extension parts and relationship types | Open | P1 | Not in [spec/parts.md](spec/parts.md) | Listed from MS-PPTX and MS-ODRAWXML |
| SPEC-05 | Checklist generated | Done | P0 | 8730 rows from 77 schemas; an unassigned type fails generation | Staleness and unassigned-namespace failures both seen |
| PKG-01 | Read budget was cumulative | Done | P2 | Each part charged once | Refused on first reread before the fix |
| F-006 | Microsoft's Open XML validator | Done | P0 | SDK 3.5.1 pinned, on Linux in CI. 31 decks: 25 clean, 6 only as recorded in `false-positives.tsv`. Installed `dotnet` by formula, since the cask needs `sudo` | Seeded invalid element rejected; not yet run on GitHub |
| F-007 | Real PowerPoint-written decks | Done | P0 | 66 tested, 12 committed: 54 Windows, 11 Mac by `app.xml`, which can be false. All 66 open, import, save unedited and force-rewrite with 4,634 parts byte-identical. **Import reads only presentation, properties and theme parts, so slides are not yet shown to be understood.** None has Zoom or equations; the only Morph deck is local | [fixtures/real](../fixtures/real/README.md) |
| UX-002 | Live-session measurements | Open | P1 | [50](50-POWERPOINT-UX-STUDY.md) §40 lists 21 measurements, and §42 the behaviours only a live session settles: marquee rule, group entry, undo depth, web default ribbon. Run with `PV-01`'s access, earlier if possible | Each recorded, and rules corrected |
| PV-01 | PowerPoint verification | Open | P0 | Release gate from `DEC-04`: no repair prompt on exports (F5), visual comparison (F6), edits (F8). Microsoft publishes no repair-trigger list, and a schema-valid deck can still prompt, so also open the seeded `RR-*` fixtures. Run whenever access exists; earlier is cheaper | Before first stable release |
| PPT-01 | Import legacy `.ppt` | Open | P2 | `DEC-08`. CFB and MS-PPT parsers with limits and fuzz targets | Phase 10 |
| NAME-01 | Rename product to OpenPresent | In progress | P2 | README, AGENTS and docs renamed; docs 19 and 22 follow 1D's merge. GitHub description and topics set. Crate prefix, skill names and folder unchanged | No product use of the old name remains |
| DOC-007 | Owner instruction log | Done | P0 | [03](03-OWNER-INSTRUCTIONS.md), first in authority, cited from `CLAUDE.md` | Exists, indexed |
| SEC-015 | Deck content in error messages | Done | P0 | 7 leaks found, 0 now, by a marker planted through every PKG, XML, OPC, MCE and FND code. Import's own CS-IMP, CS-IMF and CS-CLR codes are not covered: `SEC-018` | `no_document_text.rs` |
| SEC-016 | Caps with no test | Done | P0 | Fixture per cap | Each of 7 caps removed in turn went red |
| SEC-017 | Tests that did not test their claim | Done | P1 | Allocation and `consumed()` now tested through the real path; docs/07 corrected on `zip`'s role | Seen red under mutation |
| FUZZ-01 | `XmlTree::parse` fuzz target | Done | P1 | `xml_tree`. First campaign found a dropped second BOM, fixed. 1.5M runs in 301 s clean, then 562k on final code | No artifact |
| CI-01 | Gate that crates never depend on a server | Done | P2 | `check-crate-deps.py`. Fixed after merge: it read worktree copies of crates as outside `crates/` | Seeded edge fails |
| SPEC-06 | Catch-all area rules removed | Done | P0 | Audit seeded `ZZ_Unassigned`, filed silently under PRES. The final rules now name their types | Seeded type fails generation; output unchanged |
| FID-01 | Autonumber behaviour | Open | P1 | LibreOffice resave changes `arabicParenBoth` to `2)`, and Thai and Hebrew schemes to a bullet. Renders two paragraphs with `startAt=2` as `(b) (b)`; PowerPoint's behaviour unknown. Feeds BEH-11 | Fixture against a reference |
| F4-02 | `buSzPct` form | Open | P0 | **Answered:** real decks write numeric `130000`, and none used `100%`. The SDK accepts numeric and refuses `100%`; our F4 does the opposite, so `area-text.pptx` cannot pass both. Fix F4: allow numeric in Transitional, as PowerPoint writes. Change the deck | Both validators agree on `area-text` |
| FID-02 | Fixture coverage | Open | P1 | No decks for charts, SmartArt, math, ink, media, OLE, comments, VML, signatures or extension namespaces | Coverage per tier closes |
| SPK-04 | Third painting option | Open | P2 | Engine-rasterized bitmaps not measured. **Decided:** `Path2D` proceeds; bitmaps tried only if `TXT-05` shows no-GPU clients need them | `TXT-05` |
| TXT-03 | Verify Akasia against Aptos | Open | P0 | Self-reported OFL metric-compatible face, first released June 2026. If true, it removes the largest reflow risk. Needs metric comparison without bundling Aptos | Advance widths compared; licence read |
| OT-01 | Is character OT needed? | Open | P1 | Figma and Google Docs support server order and per-property rules. ONLYOFFICE ships presentations with object marking and no character OT in the same object. **Kept: a clone must allow two people in one text box**, so character OT stays (ADR-009). Reviewed again at Phase 9's audit | Decision recorded at Phase 9 |
| FID-03 | Fixtures for the 17 ranked problems | Open | P0 | [52](52-COMPETITOR-STUDY.md): fonts, autofit, SmartArt, animation and round-trip validity first. The first five were each reported by several products | Each has a fixture in its tier |
| ADR-014 | Precedent check for ADR-013 | Done | P2 | Study found the need strongly supported and no precedent for a byte-faithful tree, so it is ours to prove (`ARC-02`). Kept | — |
| T1D-03 | Tier 1D leftovers | Open | P1 | Hostile fixtures for `max_slides`, `max_parts`, `max_layout_ids`; codes in docs/20; `bgRef` finding keeps its colour code; verify layout-to-master matching claims that came from memory; fixtures for 46 enum values and lock attributes; `insert_in_order` namespace limit | Per-tier gate |
| FONT-02 | Fonts removed from the repo | Done | P1 | Owner instruction 20. `casual-slide-text` and `webapp/fonts` (1,645 files, 67 MB) were copied once in the first commit and nothing used them. Copy again from OpenCalc at Tier 2D. `TXT-*`, `FONT-01` wait for Phase 2 | Workspace builds and tests pass without them |
| GEO-01 | `upArrow` missing from Ecma's preset definitions | Done | P1 | The two `upDownArrow` copies are identical. `upArrow` derived from `downArrow` by y to h-y; LibreOffice's copy agrees on path, sites, text rectangle, handles | Golden and mirror tests; generator exits 1 if the override goes |
| GEO-02 | Other defects in Ecma's definitions | Done | P1 | `textNoShape` absent from warps (empty definition added); `curvedLeftArrow` uses undefined `cd3`; surplus operands in 8 circular arrow formulas; `pie` text rectangle inside out; `gear6` and `gear9` mirrored guides. Corrected explicitly in the generator. **`pie` and gear corrections are our reading, not a PowerPoint match** | A test per correction; PowerPoint check via `PV-01` |
| GEO-03 | Geometry engine | Done | P0 | `casual-slide-geometry`: 17 operators, 40 built-in guides, all 187 presets and 41 warps, own trig for determinism. Bit-exact fingerprint agrees on aarch64 and wasm32; x86_64 not run. Built-in list partly recalled, since the Ecma prose is not in the repo. 181 of 187 paths agree with LibreOffice's table | Main session merged, gates pass, broke `atan2` and saw red |
| GEO-04 | Finish geometry security | Open | P1 | Skill requires a fuzz target, hostile fixtures, stable codes in 20, and SEC-C08 in 07. Has a 3000-case seeded test, no fuzz campaign | Target runs clean |
| GEO-05 | x86_64 determinism | Open | P1 | Fingerprint unrun on x86_64; CI's Linux and Windows runners are x86_64, so running the suite in CI checks it | Green in CI |
| SEC-010 | Package threats SEC-P04 to SEC-P07 | Done | P1 | Oversized allocation from a forged size; duplicate and case-equivalent names; malformed names. The `zip` crate merges duplicates, so a raw directory walk was added. Inherited from OpenCalc | Each test seen red; guards disabled and seen red again; 19 tests pass |
| SEC-011 | Report package findings to OpenCalc | Open | P1 | Same code at `b23c67c`; that repo is not modified from here | Owner informed |
| SEC-013 | Zip bomb dilution, XML read before size check | Done | P1 | Found by the corpus: 64 MiB of zeros admitted because other parts diluted the ratio | Tests seen red; `SEC-P12`, `SEC-P13` |
| SEC-012 | Remaining threats in [07](07-SECURITY-THREAT-MODEL.md) | Open | P0 | One row per `SEC-*`, closed by the tier named there | Hostile fixture per threat |
| ENT-000 | Requirements in [10](10-ENTERPRISE-REQUIREMENTS.md) | Open | P0 | Gate on each delivering phase | Per requirement |
| SPK-01 | WASM text cost and painting | Done | P0 | [64](64-SPK-01-TEXT-COST.md), on an Apple M4 under load. Full text stack 313 KiB Brotli over a bare bundle; 2,000 characters lay out in under 1 ms warm; dense slide 8 to 13 ms on GPU. `Path2D` is correct, lighter than `fillText`. Windows, Firefox, Safari, low-end devices, a real deck: not measured | Main session viewed the side-by-side PNG |
| SPK-02 | Fidelity oracle pipeline | Done | P0 | `tools/fidelity`, sandboxed, refusing hostile decks. Linux path untested. PowerPoint references wait on `PV-01` | 10 comparisons over 8 decks; a seeded lost shape is named |
| SPK-03 | Preset shape definition licensing | Done | P0 | Ecma's notice permits use in conformant implementations; vendored unmodified. Formal review is `DEC-07` | Recorded in [schemas/README](../schemas/README.md) |

## Phases 1 to 10

Each tier and phase in [06](06-ROADMAP-AND-DELIVERY.md) is one row, split when opened. Scope per row is [spec/](spec/README.md).

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| T-1A | Package, part graph, properties, repackager | In progress | P0 | Structure, properties, repackager, MCE, Strict built and merged. Open: `XML-02`, `SEC-015`, `SEC-016`, `SEC-017`, `PKG-04`, `SPEC-08` | Per-tier gate in 06 |
| T-1B | Presentation part, view state | In progress | P0 | Built and merged, with fuzz target. Removing a whole part refused; `defaultTextStyle`, `penClr`, `clrMru`, `extLst` retained, not modelled; app counts not recalculated | Per-tier gate |
| T-1C | Colour and theme | In progress | P0 | Model, import and export built and merged; 236 workspace tests pass. Audit: code references 314 COLOR and 159 THEME rows partly. Remaining: `COLOR-01`, `COLOR-02`, `COLOR-03`, effect reference, format scheme internals, object defaults, `themeManager` | Per-tier gate in 06 |
| T-1D | Slides, masters, layouts, shape tree, `-style` | In progress | P0 | Built and merged. All 12 committed real decks import through slide import. Open: `T1D-03`. Most spec rows still lack fixtures, so the tier gate is not met | Per-tier gate in 06 |
| T-1E | Geometry, fills, lines, effects, 3D | In progress | P0 | Built and merged: 362 tests, 13 hostile decks, fuzz target, 12 real decks clean. No shape-level gradient or pattern in the real decks, only generated ones. Status marks need the audit to see `token_enum!` values (`STAT-01`) | Per-tier gate in 06 |
| T-1F | Text model | In progress | P0 | Built and merged: text bodies, all properties, 41 autonumber schemes, inheritance chain, fuzz target. Real decks are fixed points. Open: `T1F-04` | Per-tier gate in 06 |
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

## Gap analysis, 2026-11-07 (`DOC-004`)

From an independent review. The percent-encoding, tracker pattern, MCE and dead-reference claims were re-run by the main session and confirmed. Decisions taken under instruction 12 in [03](03-OWNER-INSTRUCTIONS.md).

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| WR-01 | Fidelity gates can pass without the writer running | Done | P0 | Every XML part in the corpus is forced through tree and writer, then compared with bytes, findings and F4 | Mutation of tree writing seen red by the worker |
| WR-02 | Export and save, Tier 1N | Open | P0 | Designed in [36](36-EXPORT-AND-SAVE.md): 118 invariants, each with a specified `RR-*` fixture; none built yet. Thinly sourced: slide-level ChartEx and 3D `AlternateContent`, `.thmx` structure, PowerPoint's own notes master | One repair-risk fixture per invariant |
| WR-03 | Fallbacks go stale on edit | Open | P0 | Policy designed in 36; render-on-save in `-sdk`. Not built | Phase 6 exit |
| WR-04 | Signature dropped explicitly on edit | Open | P1 | Promised in 07, no owner until now. Tier 1N | Signed fixture |
| WR-05 | Save-as matrix | Open | P1 | 121-pair grid with VBA fate designed in 36. Not built | Fixture per pair |
| OPC-01 | Percent-encoded part names misread | Done | P1 | `part_name.rs`: ECMA-376 Part 2 equivalence; refuses what decodes to traversal | Main session broke percent decoding; 4 tests red |
| MCE-01 | Markup compatibility | In progress | P0 | Pure resolver over namespace URIs; both branches kept. Understood list is ECMA-376 alone, so `p14` takes Fallback until the importer maps it. Checklist has no rows, since Part 3 ships no XSD (`SPEC-08`) | Tests and F4 agree on 23 vectors plus 4 decks |
| F4-01 | F4 validation tool | Done | P0 | `tools/f4/validate.py`; in CI on Linux. Microsoft-namespace parts report `no-schema`, `.vml` is unchecked, Dublin Core is stubbed | Seeded invalid part fails |
| STRICT-01 | Strict handling | In progress | P0 | Namespace map and write-back in source form; 14 Strict XSDs vendored. The only Strict deck is synthetic. Import reads Strict percentages; nothing above that uses the map fully yet | PowerPoint-saved Strict deck via `F-007` |
| ARC-01 | ADR-013 wording contradicted the DAG | Done | P0 | **Decided:** `-model` stays plain values. The tree lives with `-export`; ops mark objects dirty, and save applies dirty values to the tree. See [28](28-PACKAGE-AND-PART-GRAPH.md) | Phase 1 tiers follow it |
| ARC-02 | Prove ADR-013 end to end | In progress | P0 | One property through tree, model, edit, save, reopen | Row for each view follows it |
| SKEL-01 | Walking skeleton | Blocked | P1 | **Reversed by owner instruction 20:** no rendering until Phase 1 closes. Started and stopped 2026-11-07; nothing merged. First task of Phase 2 | Phase 1 closed |
| TXT-01 | Font contract | Open | P0 | Session font manifest pinned for collaborators and export, reported on mismatch. Aptos has no substitute. Corpus split by font availability. After `SPK-01` | Design note; fixtures |
| TXT-02 | Split 2D and 2F into smaller tiers | Done | P1 | Applied in [06](06-ROADMAP-AND-DELIVERY.md) | — |
| SAVE-01 | Autosave, recovery, atomic save | Open | P0 | Nothing designed. docs/83, exit criterion of Phase 4 | Crash test loses nothing |
| OP-01 | Op design: binary assets and retained XML | Open | P0 | Pasting or inserting has no wire form. Asset ops, subtree payload, deep-clone rules in 24 | Design before Phase 4 |
| OP-02 | TP1 and inverse tests at each editing phase | Done | P0 | Applied in 06 rather than waiting for Phase 9 | — |
| CAP-01 | Capacity targets | Open | P1 | docs/30: memory ceiling in wasm32 against 1 GiB input and 4 GiB expansion, media by range or Blob, save time, low-end device | `benchmark-smoke` with baselines |
| API-01 | SDK async and worker-ready | Open | P1 | ADR before `-sdk`. OpenCalc measured 196 ms stalls on save | ADR Accepted |
| VER-01 | Schema, snapshot and protocol versioning | Open | P1 | Version fields and golden-tested migrations in 22; mixed-version policy in 24 | Migration test |
| I18N-01 | Localization seam from Phase 3 | Open | P1 | Pseudo-locale gate, RTL mirroring, date formats for fields | Phase 3 exit |
| BRW-01 | Browser matrix and degraded modes | Open | P1 | Named browsers, Tauri webviews, per-browser thresholds | Multi-engine smoke |
| DET-01 | Determinism across platforms | Open | P1 | Math library, float rules, native-vs-WASM display-list hash | CI job |
| MODEL-01 | "No floats in the model" is false | Open | P1 | Charts and timing use doubles; coordinates admit units such as `1in`. Keep source lexemes | Fixtures; fix 22 |
| THR-01 | Threshold ceilings | Open | P1 | Class ceilings, a ratchet, unwaivable structural checks | Applied in 51 |
| DT-01 | Desktop plan | Open | P1 | Tauri notes, signing identities, updater, file associations. Fix 19's Phase 10 against 06's Phase 3 | Phase 3 |
| PROC-02 | Mistake during a merge | Done | P2 | A stray `git checkout --theirs .` overwrote conflict resolutions mid-merge. Nothing committed was affected; conflicts restored with `checkout -m` and redone. Do not run wildcards in a merge | — |
| PROC-01 | Check worker gate workarounds | Done | P1 | A worker met the README link failure and temporarily symlinked a sibling in a shared directory. It used no `--no-verify`, but was not authorised to, and it worked around a gate. Root cause fixed in `77354d1`, link removed and checked gone. Worker brief now says a failing gate is reported, never routed around | `openslide-worker` skill |
| COLOR-01 | Finish Tier 1C security | Open | P1 | Skill requires a fuzz target, hostile fixtures and limits in `OoxmlLimits`; caps are constants so far (64 transforms). In-test byte mutation is no substitute | Target runs clean; fixtures seen red |
| COLOR-02 | Colour arithmetic left open by the standard | Open | P1 | `inv` encoded against LibreOffice's linear, `gray` weights, gamma transfer, `hueMod` clamp, half-up rounding, HSL state, `sysClr` fallback from Windows defaults recalled from memory, `masterClrMapping` chain. Each pinned by an `open_*` test. BEH-03, BEH-04 | Settled by `PV-01` or `F-007` |
| COLOR-03 | Four `lumMod` 50% values differ by one step from recalled PowerPoint palette | Open | P2 | `ED7D31`, `A5A5A5`, `FFC000`, `70AD47`. The PowerPoint values are from memory. The standard's own examples contradict each other on rounding | A real Office theme via `F-007` |
| XML-01 | `XmlTree::attribute` ignored namespace | Done | P1 | Removed. `attr()`, used for `.rels` and content types, has the same flaw: `XML-04` | compile_fail doctest |
| XML-02 | Walker accepted malformed and duplicate attributes | Done | P0 | Refused in `walk`, so all readers agree. Existing decks still open and round-trip | Main session removed the repeat check and saw red; 8 `xml-*` hostile decks |
| SPEC-07 | Standard's internal inconsistencies recorded | Done | P2 | `dkSeaGreen` differs from `darkSeaGreen`, `ltGoldenrodYellow` from `lightGoldenrodYellow`, followed as written. Worked examples for `hslClr` 240/100/50, `lumMod` and `blueOff` contradict computation or themselves | Tests pin each |
| GEOM-01 | Path boolean engine for merge shapes | Open | P1 | In `-geometry`, Phase 4. Must be deterministic like the rest | Golden shapes |
| D3-01 | 3D model rendering and editing | Open | P2 | **Decided, clone goal:** Edit in Phase 10, Retain with fallback until then. Spec level changed to match | Phase 10 |
| PKG-02 | Part name comparison | In progress | P1 | One rule in the package crate. Import's `locate_part` and export's `add_part` still use `eq_ignore_ascii_case`, so percent-encoded targets can differ from the graph | Both use `part_name_key` |
| PKG-03 | Fixed timestamp | Done | P2 | Set explicitly | Red with `zip`'s `time` feature on |
| PKG-04 | Uncapped `read_part` | Done | P0 | `read_xml_part`, used by import and export | Refuses with nothing decompressed |
| PKG-05 | Duplicate security rule | Done | P1 | `resolve_internal` exported, import's copy deleted | Existing tests cover the rule |
| XML-03 | Walker admitted non-XML | Done | P0 | 56 of 58 hostile vectors were admitted, all refused now | Hostile fixtures |
| WR-06 | Signature parts not dropped on edit | Open | P1 | `SaveOutcome.signature_invalidated` reports it; docs/07 says drop. Part of `WR-04` | Signed fixture |
| SPEC-08 | Checklist area for markup compatibility | Open | P0 | Hand-written rows for `mc:Ignorable`, `ProcessContent`, `MustUnderstand`, `AlternateContent`, `Choice`, `Requires`, `Fallback`, tied to BEH-18 | Rows in `docs/spec` |
| TXT-04 | White-on-dark text weight | Open | P1 | `fillText` has 25 to 68% more ink on macOS dark backgrounds. Viewed: `Path2D` is correct and legible but visibly lighter. Try weight compensation; compare to PowerPoint via `PV-01` | Compared on dark slides |
| TXT-05 | Low-end and no-GPU clients | Open | P1 | Software rendering of dense CJK reaches 27 to 36 ms, over a frame. Known weak case | Measured on a named device |
| TXT-06 | Windows, Firefox, Safari check | Open | P1 | Windows Chrome is the platform most likely to favour `fillText`. Before Phase 3 closes | Same PNG viewed on each |
| TXT-07 | Thai and similar line breaking | Open | P1 | `unicode-linebreak` has no dictionary; icu_segmenter adds 297 KiB. Load per script, on demand | Fixture passes |
| XML-04 | `attr()` matches any namespace | Open | P1 | Used for `.rels` and content types. Same flaw as `XML-01` | Foreign attribute test |
| SEC-018 | Marker test for import codes | Open | P1 | CS-IMP, CS-IMF and CS-CLR not covered by `no_document_text.rs` | Marker finds none |
| DOC-009 | Confirm docs/20 wording on relationship ids | Open | P2 | Now permits them in messages, though file-supplied and unchecked as NCName. Tighten by checking the form | Check added |
| F4-03 | F4 against what Office actually writes | Open | P1 | Empty `c:ext` refused in 3 decks, `c:showDLblsOverMax` position, `c:showLeaderLines`. PowerPoint-written output should not fail F4 by default | Real decks pass or each failure is recorded |
| T1J-04 | Signed chart axis ids | Open | P1 | Mac PowerPoint writes negative `c:axId` and `c:crossAx`. Importer must accept and keep them. Both validators reject it | `poi-bar-chart-mac` round-trips |
| STRICT-02 | Genuine PowerPoint-saved Strict deck | Open | P1 | `sdk-strict-*` decks report PowerPoint 14 with numeric values and look converted by a tool (inferred) | One real Strict deck |
| F-008 | `vt:array` bounds | Open | P2 | SDK and vendored XSD disagree on `lBound` and `lBounds`; export writes Ecma's form; no real deck uses it | Real deck, or `PV-01` |
| SEC-019 | Geometry diagnostics keep file text | Open | P1 | `MAX_SUBJECT` keeps 64 bytes of file text, breaking docs/20's contract | Marker test |
| SEC-020 | `max_path_bytes` has no test | Open | P2 | Missed by `SEC-016` | Fixture seen red |
| DOC-010 | Register missing codes | Open | P1 | `CS-IMP`, `CS-IMF`, `CS-EXP` not in docs/20; `GeometryError` has no stable codes; `GEO-06` cited with no row | In docs/20 |
| F-009 | Clear rights on a Morph deck, and find Zoom and equation decks | Open | P1 | The only Morph deck stays local; Zoom and equations have no deck | Committed decks |
| GEO-06 | Geometry defect labels mismatch | Open | P2 | The generator labels its corrections `GEO-02` to `GEO-06`, which do not mean what this tracker's rows of those ids mean. Relabel in code to match `GEO-02` | One meaning per id |
| T1E-06 | Tier 1E leftovers | Open | P1 | `prstTxWarp` reader not called, `CT_FlatText`, `CT_Connection`, `DrawingLimits` not threaded through `SlideLimits`, colour findings lose `CS-CLR` on slide parts, older `2007/7/7` namespace reported not modelled | Per-tier gate |
| DRAW-01 | `ST_PositiveCoordinate` wider than PowerPoint accepts | Open | P1 | MS-OI29500 2.1.1330: PowerPoint stops at 2147483647. We keep the schema's range, so a huge bevel could be written that it may reject | Writer refuses or clamps; test |
| T1F-04 | Tier 1F leftovers | Open | P1 | Picture bullet with 1H; `bodyPr` `scene3d`, `sp3d`, `flatTx`; warp `avLst`; 5 bodies refused on full rewrite, assumed picture bullets, unchecked; `every_text_body_can_be_written_from_the_model` takes 75 s | Per-tier gate |
| STYLE-01 | Inheritance gaps | Open | P0 | Theme object defaults (`txDef`, `lnDef`, `spDef`) and `p:style/fontRef` not in the chain, so text colour from a shape style resolves wrongly. MS-OI29500 2.1.1379, 1388, 1399 | BEH-02 table cases |
| COLOR-04 | One rule for colour finding codes | Open | P2 | **Decided:** colour findings keep `CS-CLR` everywhere. Today 1E folds them to `CS-IMF` while `highlight` keeps `CS-CLR` | Same bad colour gives one code |
| GOV-01 | Governance and gates | Done | P1 | SECURITY, CONTRIBUTING, CODE_OF_CONDUCT, GOVERNANCE, CHANGELOG, NOTICE; docs 15 and 21; 7 repo gates with 22 seeded violations; post-merge and scheduled fuzz workflows | `check-gate-selftest.py` |
| FIT-01 | Fit-gap for every PowerPoint feature | Done | P1 | [96](96-POWERPOINT-FIT-GAP.md): 318 rows; Edit 230, Host seam 49, Refuse 23, Retain 14, Render 2. 252 from read pages, 41 from index titles only, 15 inferred; none from running PowerPoint | Every non-Edit row has a reason |
| ASSET-01 | Blank deck, default theme and templates | Open | P1 | File New needs an authored master and layouts. Our own; rights recorded | Before Phase 4 |
| DGM-01 | SmartArt and equations are Edit | Done | P1 | **Decided**, clone goal. Needs a diagram layout engine and math layout, in 2J and 2K. Missing drawing part defined in docs/36 | Fixture |
| PDF-01 | Tagged PDF | Done | P1 | Added to 2G | — |
| A11Y-01 | Accessibility checker, reading order pane | Open | P1 | Phase 4, with a rule list | Phase 4 exit |
| CROSS-01 | Chart data editor depends on OpenCalc's SDK | Open | P1 | Version policy, bundle cost, spike before Phase 6 | Spike |
| FONT-01 | Embedded fonts decode path | Open | P1 | Spike and design: decode, rights, cache isolation, retain, default policy | Design note |
| CFB-01 | Compound File reader tier | Open | P1 | Distinct codes for encrypted, IRM and `.ppt`; limits, fuzz, KDF iteration cap | Fixtures |
| SEC-014 | Threat classes missing from 07 | Open | P1 | Stale thumbnails and orphan data, PDF output, Tauri IPC and updater, `.ppsx` auto-show, hover actions, fullscreen spoofing, web embeds, add-in parts, polyglot ZIPs, malicious client snapshots, MCP scope | Rows in 07 |
| OPS-01 | Operations | Open | P1 | docs/65 before Phase 9: backup, restore, SLOs, RPO, RTO, alerts, content-free crash reports | Before Phase 9 |
| CORP-01 | Corpus targets | Open | P1 | Counts by producer, version, language, script and size | Phase 1 exit |
| UXT-01 | User test and UX sweep | Open | P1 | Defined separately from F8; scripted sweep and reachability audit, as OpenCalc | Each editing phase |
| LEG-01 | Legal scope widened | Open | P1 | `DEC-07` covers UI imitation, trademark, codec patents; third-party inventory beyond Rust | Before public release |
| DOC-008 | Dead references | In progress | P2 | Docs 15 and 21 now exist. 14 dead references remain, listed in `tools/doc-references.known`; a new one fails the gate | Known list empty |
| MOB-01 | Mobile and touch | Open | P2 | **Decided in scope** as a responsive shell after Phase 7, as OpenCalc's phone shell | Phase 10 |

## Phase 1 tasks

Order and waves are in [06](06-ROADMAP-AND-DELIVERY.md). Tier rows above close when their tasks do.

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| STAT-01 | Fill checklist status columns | Open | P0 | No row is marked `y`, so no tier can show its gate. Teach `tools/spec-audit` to propose marks from code and fixtures; a mark needs both. Apply to 1A, 1B, 1C, 1D | Marks pass `audit.py --check`; a false mark fails |
| T1A-02 | Close Tier 1A | Open | P0 | `SPEC-08`, `PKG-02`, `XML-04`, `SEC-018`, `DOC-009`, `F4-02` | Per-tier gate |
| T1B-02 | Close Tier 1B | Open | P1 | Whole-part removal rule; `defaultTextStyle` follows `T1F-01`; app counts follow `T1N-05` | Per-tier gate |
| T1C-01 | Tier 1C leftovers | Open | P1 | `COLOR-01`; effect reference; `themeManager`; export setters for panose, pitch, charset, style reference index | Tests seen red |
| T1C-02 | Close Tier 1C | Open | P0 | Format scheme internals, after 1E; object defaults, after 1F | Per-tier gate |
| T1D-02 | Merge Tier 1D | Done | P0 | Merged onto the parser fixes; 372 tests pass | Gates pass |
| SKEL-02 | Skeleton: minimal layout and PNG | Blocked | P1 | Waits with `SKEL-01` | Phase 1 closed |
| T1E-01 | Geometry binding | Done | P0 | `prstGeom`, `custGeom`, `avLst`; compiled, not evaluated. `GEO-04` is separate | Enum values read from the checklist; edits change only named nodes |
| T1E-02 | Fills | Done | P0 | All kinds. `read_fill` ready for `T1C-02` | Enum values read from the checklist; edits change only named nodes |
| T1E-03 | Lines | Done | P0 | All properties; sketch extension `uri` Inferred | Enum values read from the checklist; edits change only named nodes |
| T1E-04 | Effects | Done | P0 | Lists, DAGs, blip and `a14` picture effects; `imgProps` cannot be added | Enum values read from the checklist; edits change only named nodes |
| T1E-05 | 3D properties | Done | P0 | `scene3d`, `sp3d`. `CT_FlatText` type only | Enum values read from the checklist; edits change only named nodes |
| T1F-01 | Text structure | Done | P0 | `txBody`, `bodyPr`, `lstStyle`, paragraphs, runs, breaks, fields; `defaultTextStyle`, master text styles. Fills, lines and effects on runs go through 1E | Fixed point on real decks |
| T1F-02 | Text properties | Done | P0 | Every run and paragraph property; fonts as the file states them, with no metrics | All enum values read from the checklist |
| T1F-03 | Autofit properties | Done | P0 | Stored as written; recomputation is 2D6 | Stored values round-trip |
| T1G-01 | Tables | In progress | P0 | `tbl`, grid, rows, cells, merges, cell properties, text via 1F | Fixed point; merged-cell fixtures |
| T1G-02 | Table styles | In progress | P1 | `tableStyles.xml`, unknown style GUID reported (BEH-29) | Unknown GUID never crashes |
| T1H-01 | Pictures | In progress | P0 | Blip references, crop, tile, stretch, recolour; `SEC-C01` | External blips not fetched |
| T1H-02 | SVG with PNG fallback | In progress | P0 | `asvg` extension kept with its fallback; `SEC-C03` | Script-bearing SVG fixture is inert |
| T1H-03 | Audio and video | In progress | P1 | Media references, trim, bookmarks, tracks, poster; embedded and linked | Linked media not fetched |
| T1I-01 | Timing tree | Open | P0 | `p:timing`: all node types, behaviours, conditions, build lists; caps, `SEC-C10`. BEH-23 semantics are Phase 8 | Fixed point; depth caps |
| T1I-02 | Transitions | Open | P0 | 21 base, 19 from 2010, `prstTrans`, Morph, in `mc:AlternateContent` | Both branches kept; fallback resolved |
| T1J-01 | Classic charts | Open | P0 | `c:chartSpace`: 16 families, series, axes, data labels, caches; embedded workbook retained, never opened; `SEC-P09` | Fixed point on real decks |
| T1J-02 | Chart extensions and styles | Open | P1 | `c14`, `c15`, `c16`, chart style and colour parts. BEH-26 | Retained with base chart |
| T1J-03 | 2016+ charts | Open | P1 | `cx:` with `AlternateContent` fallback, 8 layouts | Fallback kept with chart |
| T1K-01 | SmartArt parts | Open | P0 | Data, layout, style, colours and drawing as linked parts; layout never run on load; `SEC-C12` | 5 parts stay consistent |
| T1K-02 | Equations | Open | P1 | OMML, 326 rows; fallback retained | Fixed point |
| T1K-03 | Ink | Open | P2 | `contentPart`, InkML, actions; fallback retained | Fixed point |
| T1L-01 | OLE and controls | Open | P1 | Retained with fallback image; never opened; `SEC-C07` | Bytes compared |
| T1L-02 | VML | Open | P2 | Legacy drawing parts retained; schema-checked in F4 | Bytes compared |
| T1L-03 | 3D models | Open | P2 | Retained with fallback until `D3-01` | Bytes compared |
| T1M-01 | Comments | Open | P1 | Legacy and modern, authors, threads, reactions, tasks | Both forms round-trip |
| T1M-02 | Sections, zoom, classification | Open | P1 | `sectionLst`, zoom objects with fallback, labels (ENT-04) | Section ids stay in sync |
| T1M-03 | Review and custom data | Open | P2 | Change tracking, revision info, Designer, tags, custom XML retained | Bytes compared |
| T1N-01 | Writer: part naming and rIds | In progress | P0 | [36](36-EXPORT-AND-SAVE.md) rules, with `RR-*` fixtures | Each seeded break refused |
| T1N-02 | Writer: content types and shared parts | In progress | P0 | Default against Override, reference counts | Each seeded break refused |
| T1N-03 | Writer: id rules and sections | In progress | P0 | Slide, master and layout ids; section sync; notes master | Each seeded break refused |
| T1N-04 | Writer: add, duplicate, delete slide at file level | Open | P0 | What Phase 4 needs beneath its ops. Deep clone rules from `OP-01` | Reopens with F4 and validator clean |
| T1N-05 | Writer: docProps, thumbnails, signature drop | Open | P1 | App counts recalculated; `WR-06` | Fixtures |
| T1N-06 | Writer: save-as matrix | Open | P1 | `WR-05`: 121 pairs | Fixture per pair |
| P1X-01 | Phase 1 exit run | Open | P0 | `CORP-01` corpus at target, 500-slide budget, hostile corpus with no crash or hang, all tiers closed | Evidence assembled for owner sign-off |

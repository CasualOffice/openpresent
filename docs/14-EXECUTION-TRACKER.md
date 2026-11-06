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
| DOC-005 | Phase and task rules | In progress | P0 | [05](05-PHASES-AND-TASKS.md), charters in [06](06-ROADMAP-AND-DELIVERY.md). Needs owner approval | Owner approves |
| DOC-006 | Working rules and skills | Done | P1 | `CLAUDE.md` holds standards, one-roadmap-one-tracker, document updates. Six project skills in `.claude/skills`. `check-repo.py` refuses a second tracker or plan | Seeded second tracker seen failing |
| ENG-01 | Enforce standards by lint | Open | P1 | Add `unwrap_used`, `expect_used`, `indexing_slicing`, `missing_docs` for non-test code. Waits for the parallel sessions to merge, since it touches their files. Copied `-text` needs a pass | Clippy passes with the lints on |
| DOC-002 | ADR review | Done | P0 | All Accepted except ADR-004 and ADR-005, which wait on `SPK-01` | [08](08-ADR-REGISTER.md) |
| UX-001 | PowerPoint UI and UX study | In progress | P1 | [49](49-POWERPOINT-COMMAND-INVENTORY.md) done: desktop commands from Microsoft's control-ID workbook, Current Channel 2402, so 32 months old; web mostly unknown (1470 of 1855 rows `?`). [50](50-POWERPOINT-UX-STUDY.md) done, with 24 shell rules; nothing first-hand, and every pixel figure an estimate. [52](52-COMPETITOR-STUDY.md) done. Pixel sizes stay unmeasured without a live session, so geometry starts from OpenCalc's measured ribbon | Shell prototype approved by owner |

## Phase 0

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| F-001 | Cargo workspace, toolchain 1.96.0 and MSRV matching OpenCalc | Done | P0 | [19](19-WORKSPACE-SCAFFOLD-DESIGN.md). MSRV 1.88.0 check passes locally and in CI | fmt, clippy `-D warnings`, doc and wasm32 check pass locally |
| F-002 | Fork `-package` and `-text` with provenance | Done | P0 | Copied at `b23c67c` and renamed; `webapp/fonts/cjk` copied because `-text` tests read it. `-secrets` waits for its first consumer in Phase 9 | 12 and 40 tests pass. Disabling the traversal and expansion-ratio guards turned 4 package tests red; restored and green |
| F-003 | CI matrix and repo gates | In progress | P0 | `.github/workflows/ci.yml`, `tools/check-repo.py`, pre-commit hook written. First run on PR #1, 2026-11-07 | Green on three platforms |
| F-004 | Fixture corpus with checksums | In progress | P0 | 43 fixtures: generated, hostile, LibreOffice-written. No PowerPoint-written deck yet (`DEC-04`) | Manifest SHA-256 gate passes |
| F-005 | Fuzz targets for admission | Done | P1 | `bounded_package`, `presentation_package`, `xml_walker`; 505k, 968k and 5.4M runs, 46 s each, no crash. Short runs; longer campaigns belong in scheduled CI | No artifact |
| ENV-01 | LibreOffice unusable here | Done | P2 | `/opt/homebrew/bin/soffice` points at `/Applications/LibreOffice.app`, which is gone, though the cask is still recorded. Earlier docs said `soffice` was available; that was wrong | Reinstalled 26.8.2.2; `soffice --version` runs |
| SPEC-01 | Audit reports checklist coverage from code and fixtures | Done | P1 | `tools/spec-audit/audit.py`. Fixtures exercise 1822 of 8730 rows; code 0, since import and export are stubs on this base. Reports rather than fills status columns; reads source text, not behaviour, and owner detection is untested on a real importer | Planted `y` row seen failing by main session; in CI |
| SPEC-02 | Read MS-OI29500 notes per tier | Open | P1 | Where Office departs from the standard | Notes recorded before each tier opens |
| SPEC-03 | Independent count check | Done | P1 | `tools/spec-audit/count.py`: 8730 rows, 0 mismatches | Tests see a dropped row and an added attribute fail; in CI |
| SPEC-04 | Enumerate extension parts and relationship types | Open | P1 | Not in [spec/parts.md](spec/parts.md) | Listed from MS-PPTX and MS-ODRAWXML |
| SPEC-05 | Checklist generated | Done | P0 | 8730 rows from 77 schemas; an unassigned type fails generation | Staleness and unassigned-namespace failures both seen |
| PKG-01 | Read budget is cumulative | Open | P2 | `read_part` charges a whole-package budget on every read, so an editor re-reading parts can exhaust it on a legitimate deck | A long session does not refuse |
| F-006 | Microsoft's Open XML validator in the fidelity tools | Open | P0 | Needs `dotnet`. Closest available stand-in for PowerPoint's strictness | Rejects a seeded invalid part; corpus exports pass |
| F-007 | Real PowerPoint-written decks as input | Open | P0 | Find decks with clear reuse rights, record provenance. Unclear rights: test locally, never commit | At least 20 decks, Windows and Mac, open and round-trip |
| UX-002 | Live-session measurements | Open | P1 | [50](50-POWERPOINT-UX-STUDY.md) §40 lists 21 measurements, and §42 the behaviours only a live session settles: marquee rule, group entry, undo depth, web default ribbon. Run with `PV-01`'s access, earlier if possible | Each recorded, and rules corrected |
| PV-01 | PowerPoint verification | Open | P0 | Release gate from `DEC-04`: no repair prompt on exports (F5), visual comparison (F6), edits (F8). Microsoft publishes no repair-trigger list, and a schema-valid deck can still prompt, so also open the seeded `RR-*` fixtures. Run whenever access exists; earlier is cheaper | Before first stable release |
| PPT-01 | Import legacy `.ppt` | Open | P2 | `DEC-08`. CFB and MS-PPT parsers with limits and fuzz targets | Phase 10 |
| NAME-01 | Rename product to OpenPresent in docs and site | Open | P2 | After parallel sessions merge. Crate prefix and folder unchanged | No product use of the old name remains |
| DOC-007 | Owner instruction log | Done | P0 | [03](03-OWNER-INSTRUCTIONS.md), first in authority, cited from `CLAUDE.md` | Exists, indexed |
| SEC-015 | Deck content leaks into error messages | Open | P0 | **Confirmed by audit:** `unknown target mode {other:?}` in `rels.rs`, repeated keys in `content_types.rs`, and quick-xml's messages carry file text. Log contract broken | A fixture with a marker string; no message contains it |
| SEC-016 | Caps with no test | Open | P0 | Removing element, attribute, walker size, per-part and total relationship, and content-type caps left all tests green | Hostile fixture per cap, each seen red |
| SEC-017 | Tests that do not test their claim | Open | P1 | `read_part` could drop `initial_capacity`; a media read at open would pass `opening_a_deck_does_not_read_its_media`; removing our declared-size bound passes because `zip` also enforces it | Tests through the real path; assert `consumed()` |
| FUZZ-01 | `XmlTree::parse` has no fuzz target | Open | P1 | A second reader, with different config | Target exists, campaign clean |
| CI-01 | Gate that crates never depend on a server | Open | P2 | Stated in 19; OpenCalc has it, we do not | Seeded edge fails |
| SPEC-06 | Catch-all area rules removed | Done | P0 | Audit seeded `ZZ_Unassigned`, filed silently under PRES. The final rules now name their types | Seeded type fails generation; output unchanged |
| FID-01 | Autonumber behaviour | Open | P1 | LibreOffice resave changes `arabicParenBoth` to `2)`, and Thai and Hebrew schemes to a bullet. Renders two paragraphs with `startAt=2` as `(b) (b)`; PowerPoint's behaviour unknown. Feeds BEH-11 | Fixture against a reference |
| F4-02 | `buSzPct` form | Open | P1 | Vendored XSD rejects `100000` and takes `100%`. PowerPoint may write the numeric form (unverified), which would make F4 reject real decks | Checked on a real deck via `F-007` |
| FID-02 | Fixture coverage | Open | P1 | No decks for charts, SmartArt, math, ink, media, OLE, comments, VML, signatures or extension namespaces | Coverage per tier closes |
| SPK-04 | Third painting option for ADR-005 | Open | P0 | None of 4 editors whose text path could be established paints glyph paths interactively (ONLYOFFICE bitmaps, inferred from identifiers; Google SVG text at engine positions). Compare `Path2D`, `fillText` at engine positions, and engine-rasterized bitmaps. Extends `SPK-01` | ADR-005 settled on all three |
| TXT-03 | Verify Akasia against Aptos | Open | P0 | Self-reported OFL metric-compatible face, first released June 2026. If true, it removes the largest reflow risk. Needs metric comparison without bundling Aptos | Advance widths compared; licence read |
| OT-01 | Is character OT needed? | Open | P1 | Figma and Google Docs support server order and per-property rules. ONLYOFFICE ships presentations with object marking and no character OT in the same object. **Kept: a clone must allow two people in one text box**, so character OT stays (ADR-009). Reviewed again at Phase 9's audit | Decision recorded at Phase 9 |
| FID-03 | Fixtures for the 17 ranked problems | Open | P0 | [52](52-COMPETITOR-STUDY.md): fonts, autofit, SmartArt, animation and round-trip validity first. The first five were each reported by several products | Each has a fixture in its tier |
| ADR-014 | Precedent check for ADR-013 | Done | P2 | Study found the need strongly supported and no precedent for a byte-faithful tree, so it is ours to prove (`ARC-02`). Kept | — |
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
| SPK-01 | WASM shaping and glyph-path frame time | Open | P0 | [06](06-ROADMAP-AND-DELIVERY.md) | Numbers in 42 |
| SPK-02 | Fidelity oracle pipeline | Done | P0 | `tools/fidelity`, in a network-denied sandbox, refusing hostile decks. Whole-page SSIM misranks: a 0.06 pt shift scored worse than a lost shape, so regions and word positions decide. Linux path untested. PowerPoint references wait on `PV-01` | 10 comparisons over 8 decks; a seeded lost shape is named |
| SPK-03 | Preset shape definition licensing | Done | P0 | Ecma's notice permits use in conformant implementations; vendored unmodified. Formal review is `DEC-07` | Recorded in [schemas/README](../schemas/README.md) |

## Phases 1 to 10

Each tier and phase in [06](06-ROADMAP-AND-DELIVERY.md) is one row, split when opened. Scope per row is [spec/](spec/README.md).

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| T-1A | Package, part graph, repackager | In progress | P0 | Structure, findings and repackager built ([28](28-PACKAGE-AND-PART-GRAPH.md)). Properties parts not modelled yet | Per-tier gate in 06 |
| T-1B | Presentation part, view state | Open | P0 | 297 rows | Per-tier gate |
| T-1C | Colour and theme | In progress | P0 | Model, import and export built and merged; 236 workspace tests pass. Audit: code references 314 COLOR and 159 THEME rows partly. Remaining: `COLOR-01`, `COLOR-02`, `COLOR-03`, effect reference, format scheme internals, object defaults, `themeManager` | Per-tier gate in 06 |
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

## Gap analysis, 2026-11-07 (`DOC-004`)

From an independent review. The percent-encoding, tracker pattern, MCE and dead-reference claims were re-run by the main session and confirmed. Decisions taken under instruction 12 in [03](03-OWNER-INSTRUCTIONS.md).

| ID | Title | St | Sev | Mechanism | Gate |
| --- | --- | --- | --- | --- | --- |
| WR-01 | Fidelity gates can pass without the writer running | In progress | P0 | Unedited save returns input bytes, so F1, F3 and F5 prove nothing about it. Forced-rewrite mode: touch every modelled part | F3 and validators run on forced-rewrite output |
| WR-02 | Export and save, Tier 1N | Open | P0 | Designed in [36](36-EXPORT-AND-SAVE.md): 118 invariants, each with a specified `RR-*` fixture; none built yet. Thinly sourced: slide-level ChartEx and 3D `AlternateContent`, `.thmx` structure, PowerPoint's own notes master | One repair-risk fixture per invariant |
| WR-03 | Fallbacks go stale on edit | Open | P0 | Policy designed in 36; render-on-save in `-sdk`. Not built | Phase 6 exit |
| WR-04 | Signature dropped explicitly on edit | Open | P1 | Promised in 07, no owner until now. Tier 1N | Signed fixture |
| WR-05 | Save-as matrix | Open | P1 | 121-pair grid with VBA fate designed in 36. Not built | Fixture per pair |
| OPC-01 | Percent-encoded part names misread | In progress | P1 | **Confirmed by running:** space in a target against `%20` in the entry gives a false dangling and a false orphan | Fixture passes |
| MCE-01 | Markup compatibility missing | In progress | P0 | **Confirmed:** 0 checklist rows and no code. Needs Part 3 rules: Ignorable, ProcessContent, MustUnderstand, Choice order, both branches kept on edit | Checklist area, tests, F4 preprocess |
| F4-01 | F4 validation tool | In progress | P0 | Listed as unblocked but does not exist. `xmllint` likely rejects real parts without MCE preprocessing | Runs on corpus exports |
| STRICT-01 | Strict is detected, not handled | In progress | P0 | **Decided:** normalise namespaces at the read boundary, write back in source form. Vendor Strict XSDs; PowerPoint-saved Strict fixture via `F-007` | Strict deck round-trips |
| ARC-01 | ADR-013 wording contradicted the DAG | Done | P0 | **Decided:** `-model` stays plain values. The tree lives with `-export`; ops mark objects dirty, and save applies dirty values to the tree. See [28](28-PACKAGE-AND-PART-GRAPH.md) | Phase 1 tiers follow it |
| ARC-02 | Prove ADR-013 end to end | In progress | P0 | One property through tree, model, edit, save, reopen | Row for each view follows it |
| SKEL-01 | Walking skeleton | Open | P0 | **Decided:** 5 real decks through 2A, 2B and basic text right after 1D, so semantics errors appear early. Adds to, not replaces, import-export-first | 5 decks render |
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
| PROC-01 | Check worker gate workarounds | Done | P1 | A worker met the README link failure and temporarily symlinked a sibling in a shared directory. It used no `--no-verify`, but was not authorised to, and it worked around a gate. Root cause fixed in `77354d1`, link removed and checked gone. Worker brief now says a failing gate is reported, never routed around | `openslide-worker` skill |
| COLOR-01 | Finish Tier 1C security | Open | P1 | Skill requires a fuzz target, hostile fixtures and limits in `OoxmlLimits`; caps are constants so far (64 transforms). In-test byte mutation is no substitute | Target runs clean; fixtures seen red |
| COLOR-02 | Colour arithmetic left open by the standard | Open | P1 | `inv` encoded against LibreOffice's linear, `gray` weights, gamma transfer, `hueMod` clamp, half-up rounding, HSL state, `sysClr` fallback from Windows defaults recalled from memory, `masterClrMapping` chain. Each pinned by an `open_*` test. BEH-03, BEH-04 | Settled by `PV-01` or `F-007` |
| COLOR-03 | Four `lumMod` 50% values differ by one step from recalled PowerPoint palette | Open | P2 | `ED7D31`, `A5A5A5`, `FFC000`, `70AD47`. The PowerPoint values are from memory. The standard's own examples contradict each other on rounding | A real Office theme via `F-007` |
| XML-01 | `XmlTree::attribute` ignores namespace | Open | P1 | A foreign `x:val` shadows `val`, a parser differential. `plain_attribute` added and used by import; the old one remains for others to misuse | Removed or renamed; test |
| XML-02 | Walker accepts malformed and duplicate attributes | Open | P0 | `<r a/>` passes `walk` and `XmlTree::parse`. `attributes_valid` guards colour reads only, and other readers can disagree about the same bytes | Refused in `walk`; hostile fixture |
| SPEC-07 | Standard's internal inconsistencies recorded | Done | P2 | `dkSeaGreen` differs from `darkSeaGreen`, `ltGoldenrodYellow` from `lightGoldenrodYellow`, followed as written. Worked examples for `hslClr` 240/100/50, `lumMod` and `blueOff` contradict computation or themselves | Tests pin each |
| GEOM-01 | Path boolean engine for merge shapes | Open | P1 | In `-geometry`, Phase 4. Must be deterministic like the rest | Golden shapes |
| D3-01 | 3D model rendering and editing | Open | P2 | **Decided, clone goal:** Edit in Phase 10, Retain with fallback until then. Spec level changed to match | Phase 10 |
| PKG-02 | Part name comparison uses Unicode lowercase | Open | P1 | `Package` uses `to_lowercase`. Read, no failing test yet. Must follow OPC's equivalence, shared with `OPC-01` | Fixture |
| PKG-03 | Rewrite's fixed timestamp relies on a feature staying off | Open | P2 | If `zip`'s `time` feature enters through unification, determinism breaks silently | Set explicitly; test |
| GOV-01 | Gates, governance and release | Open | P1 | Port OpenCalc's self-test, doc-claims, doc-references, tracker-shape, fuzz-lock, host-seams and post-merge. Add SECURITY, CONTRIBUTING, CHANGELOG, NOTICE. docs/15 for test strategy and release | Phase 0 exit |
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
| DOC-008 | Doc drift named by review | Open | P2 | Dead `docs/21` and `docs/15` citations; AGENTS state; 48 pending against 14 Done; mobile undecided | `check-doc-references` |
| MOB-01 | Mobile and touch | Open | P2 | **Decided in scope** as a responsive shell after Phase 7, as OpenCalc's phone shell | Phase 10 |

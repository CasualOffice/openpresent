# 19 — Workspace and layer division

A lower layer never knows about a higher one. Changing a boundary here needs an ADR.

## Dependency DAG

```
 casual-slide-wasm ──┐
                     ▼
 casual-slide-sdk ─▶ export, import, transaction, render, anim
 render ───────────▶ layout
 layout ───────────▶ style, geometry, text, chart
 anim ─────────────▶ model
 transaction ──────▶ model
 export ───────────▶ ooxml, model, package
 import ───────────▶ ooxml, model, geometry
 text ─────────────▶ style, model   (until -layout exists; ADR-016)
 style, chart ─▶ model
 ooxml ────────────▶ package
 model, package, geometry: no internal dependencies
```

`-import` depends on `-geometry` from Tier 1E (`GEO-04`), to check a preset's token and adjust names and to compile custom geometry as it is read. It evaluates nothing at a size. `-geometry` parses no XML and depends on nothing.

`-ooxml` has `-geometry` as a dev-dependency only: `tests/geometry_fixtures.rs` holds the hostile custom-geometry decks (`GEO-04`) to the geometry caps, and nothing ships it. It is no layer dependency, and the diagram above stays true.

`-style` exists from Tier 1D and depends on `-model` alone; its tests read fixtures through `-import` and `-ooxml` as dev-dependencies only, which the DAG allows because nothing ships them. `-layout` is its first product consumer (Phase 2).

**Ten crates exist today** (2026-11-08): `-package`, `-ooxml`, `-model`, `-style`, `-geometry`, `-import`, `-export`, and `-text`, `-render` and `-wasm` from Phase 2. The rest in the table below are planned and arrive with a first consumer.

**Added in Phase 2.** `-render` (`SKEL-01`, owner instruction 43), started from OpenCalc's `casual-calc-render` with a `PROVENANCE.md`. It holds the display list, the scene that builds one from a slide's resolved shapes, and the tiny-skia PNG backend; its `render-deck` example writes one PNG per slide. Its dependencies are `-model`, `-style`, `-geometry`, and, as an **interim edge** that the diagram does not show, `-import` and `-ooxml`: `Deck` opens a package for drawing, and the scene compiles a custom geometry with import's `compile_custom`. When `-layout` arrives, the scene and the custom-geometry conversion move down into it and `Deck` to `-sdk`, and the edge goes (`SKEL-03`). External: `tiny-skia` 0.11 (BSD-3-Clause) and `image` 0.25 with only `jpeg` (MIT or Apache-2.0, bringing `zune-jpeg` and `moxcms`), per ADR-019. `-render` also depends on `-text`, which lays out text bodies into glyph runs and outlines their glyphs through the `GlyphOutlines` trait.

**`-wasm` added for the first vertical slice** (`VS-01`, owner instruction 44), started from OpenCalc's `casual-calc-wasm` with a `PROVENANCE.md`: open a deck, count and draw its slides, and report what each leaves out, for `webapp/`. Its first consumer is `webapp/render.worker.js`. Until `-sdk` exists it depends on `-render` directly, an **interim edge** the diagram does not show, which goes when `-sdk` takes `Deck` (`SKEL-03`). External: `wasm-bindgen` 0.2 (MIT or Apache-2.0), as OpenCalc's, and `tiny-skia`, already in the tree through `-render`, to decode its PNG for RGBA output. It compiles natively too, so its tests run on CI without a browser.

**Added in Phase 1, by tier.**
- `-package` takes `flate2` (`zlib-rs` backend, Apache-2.0 or MIT, already in the lockfile through `zip`) as a direct dependency for the recovering reader, `salvage.rs` (ADR-015, Tier 1R).
- `-ooxml` gains `recover_xml.rs`, `recovery.rs` and `repair.rs`: the recovering XML reader and the repair of a damaged package into one clean image (Tier 1R).
- `-model`, `-import` and `-export` gain `chart/`, `diagram/`, `math/` and `ink/`: in `chart/` one module per schema namespace, in the others one `schema.rs`, generated from the vendored schemas by `tools/chart-codegen` and `tools/spec-codegen`, and held to them by `check-chart-codegen.py` (Tiers 1J, 1K). Charts therefore live in these three crates, not in a separate `-chart`, which stays planned for plotting in Phase 2.
- `retained.rs` (Tier 1L), `comment.rs`, `section.rs`, `custom_data.rs` (Tier 1M), `table.rs` (1G), `media.rs` and `picture.rs` (1H), `timing.rs` and `transition.rs` (1I) in the same three crates.
- `-export` gains `writer/`: the save plan, preflight, part naming, bookkeeping, save-as and the ZIP check (Tier 1N).

**Added in Phase 2.**
- `-text` (Tier 2D, ADR-016), copied from OpenCalc's `casual-calc-text` with its shaper from `casual-calc-render` (`PROVENANCE.md` in the crate). Dependencies: `skrifa`, `brotli-decompressor` (as OpenCalc), `harfrust` (as OpenCalc's `-render`, always on here), `unicode-linebreak` (new: OpenCalc has no line breaker). `all-fonts` is on by default, so every host lays out with the same faces. Its `body.rs` reads a model text body through `-style`'s chain, so `-text` depends on `-model` and `-style` until `-layout` exists; then `body.rs` moves there and `-text` returns to no internal dependencies (`T2D4-02`). Its first consumer is `-render`, which names each run's face by the key `outline.rs` gives and asks it for outlines. Its corpus test reads decks through `-import` and `-ooxml`, as dev-dependencies only.

`server/*` depends on `crates/*`; nothing in `crates/` depends on a server. CI does not check this yet (`CI-01`).

## Crates

| Crate | Owns | Must never own | Source |
| --- | --- | --- | --- |
| `-package` | Bounded ZIP/OPC admission, path safety, limits | Anything presentation-specific | Fork of calc's (667 lines, no internal deps) |
| `-ooxml` | PresentationML part graph, relationships, immutable source snapshot | Model mapping | New, after calc's shape |
| `-model` | Presentation, themes, masters, layouts, slides, shape tree, text bodies, tables, retained-source references, snapshot I/O | XML, geometry, layout | New |
| `-style` | Resolving effective properties through the inheritance chain and theme | Text layout, mutation | New |
| `-geometry` | Preset and custom geometry to paths; adjust-value formulas; connection sites | Fills, text | New; generated tables, `SPK-03` |
| `-text` | Fonts, advances, outlines, shaping, bidi, line breaking, paragraph and body layout to glyph runs until `-layout` exists | Painting, I/O | Copied from OpenCalc's `-text` and `-render/shape.rs` at `b23c67c`, Tier 2D |
| `-chart` | Chart part model and plotting to display items | Workbook logic | Port from calc's `-layout` chart code |
| `-layout` | Paragraph and text-body layout, autofit, caret and hit-test geometry, scene to display list | Painting, I/O | New; display-list shape from calc |
| `-anim` | Timing tree to timeline; sampling at time *t* | Painting, wall clock | New |
| `-transaction` | Ops, inverses, transform, session, wire protocol | Layout, transport | New; protocol and session pattern from calc |
| `-import` | PresentationML to model, report, retained source | Export | New |
| `-export` | Applying dirty model values to the part tree, and package bookkeeping. The byte repackager itself is `Package::rewrite` | Parsing | New |
| `-render` | PNG (tiny-skia) and PDF backends; for now also the display list and the scene that builds it, until `-layout` exists | Layout decisions, once `-layout` exists | Started from calc's `casual-calc-render`, 2026-11-08 |
| `-sdk` | Host-facing session API and format dispatch | Internals exposure | New |
| `-wasm` | `wasm-bindgen` transport over the SDK; over `-render` until `-sdk` exists | Logic | Started from calc's `casual-calc-wasm`, 2026-11-08 |

Not created: `-selection` (lives in `-transaction`, since presence must map it through
ops), and `-odp` until Phase 10. No crate exists without a first consumer, except skeletons created to let parallel tiers start, which must gain one when their tier lands.

```
server/    casual-slide-collab-server, -host, -wopi, -mcp, -secrets
webapp/    editor and site; ribbon, command registry and tokens ported from opencalc (webapp/README.md)
desktop/   the Tauri app (VS-03): its own cargo workspace around webapp/, from opencalc (desktop/PROVENANCE.md)
sdk/       @openslide/deck and friends, Phase 10
schemas/   vendored standards, see schemas/README.md
tools/     spec-checklist, fidelity, benchmark, check-* gates
fixtures/  checksummed corpus
fuzz/      separate workspace
```

## The desktop app

`desktop/` (`VS-03`) is a separate cargo workspace, as `fuzz/` is and as OpenCalc's is
(OpenCalc's ADR-023): Tauri's tree is large and no engine build should carry it. It depends on no
`casual-slide-*` crate. The engine runs in the webview as WebAssembly from the same `webapp/`
the site serves, so the desktop draws what the web draws by construction. Its dependencies are
`tauri`, `tauri-plugin-dialog` (the platform's Open panel, called from Rust only), `serde`, and
`tauri-build` at build time; its lockfile is not yet under `cargo deny` and `cargo audit`
(`DESK-01`). `desktop.yml` builds, lints, tests and bundles it ([15](15-CI-AND-RELEASE-GATES.md)).

## Provenance

Copied code stays honest about where it came from. Each forked crate carries a
`PROVENANCE.md` naming the OpenCalc commit and any local divergence.

**Why copy rather than extract a shared repo (`DEC-02`, decided by the owner).** Only `-package`, `-text` and
`-secrets` are free of calc types today; I measured no references to its model,
formula or transaction crates in them. Extraction would mean changing OpenCalc, which
this repo's boundary forbids, and would fix shared seams before a second consumer has
shown what they are. OpenCalc and OpenDoc already share a design rather than a crate
this way (its ADR-006). Trigger to revisit: the same bug fixed by hand in two repos
twice.

**The collaboration server is not free to copy.** At 18.3k lines it imports calc's
`-transaction`, `-import`, `-export`, `-formula`, `-model` and `-eval`. See
[24](24-OPERATIONS-AND-COLLABORATION.md) §The server.

## Dependency policy

`unsafe_code = "forbid"`, `overflow-checks = true` in release, `cargo deny` and
`cargo audit`, as OpenCalc. Expected engine additions over its set are a pure-Rust
shaper (`harfrust`, which OpenCalc's `-render` moved to in its `DEP-14`) and bidi and line-break tables; each enters through ADR-004 with a measured
bundle cost. `harfrust` and `unicode-linebreak` entered with `-text` under ADR-016; the bundle cost of them and of the faces is `T2D1-02`. No charting, layout or canvas crate.

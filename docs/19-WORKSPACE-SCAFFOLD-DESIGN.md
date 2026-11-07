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
 import ───────────▶ ooxml, model
 style, chart ─▶ model
 ooxml ────────────▶ package
 model, text, package, geometry: no internal dependencies
```

`-style` exists from Tier 1D and depends on `-model` alone; its tests read fixtures through `-import` and `-ooxml` as dev-dependencies only, which the DAG allows because nothing ships them. `-layout` is its first product consumer (Phase 2).

`server/*` depends on `crates/*`; nothing in `crates/` depends on a server. CI does not check this yet (`CI-01`).

## Crates

| Crate | Owns | Must never own | Source |
| --- | --- | --- | --- |
| `-package` | Bounded ZIP/OPC admission, path safety, limits | Anything presentation-specific | Fork of calc's (667 lines, no internal deps) |
| `-ooxml` | PresentationML part graph, relationships, immutable source snapshot | Model mapping | New, after calc's shape |
| `-model` | Presentation, themes, masters, layouts, slides, shape tree, text bodies, tables, retained-source references, snapshot I/O | XML, geometry, layout | New |
| `-style` | Resolving effective properties through the inheritance chain and theme | Text layout, mutation | New |
| `-geometry` | Preset and custom geometry to paths; adjust-value formulas; connection sites | Fills, text | New; generated tables, `SPK-03` |
| `-text` | Bundled faces, advances, outlines, shaping, bidi, line-break data | Slide concepts | Fork of calc's (2.8k lines, no internal deps), plus shaping |
| `-chart` | Chart part model and plotting to display items | Workbook logic | Port from calc's `-layout` chart code |
| `-layout` | Paragraph and text-body layout, autofit, caret and hit-test geometry, scene to display list | Painting, I/O | New; display-list shape from calc |
| `-anim` | Timing tree to timeline; sampling at time *t* | Painting, wall clock | New |
| `-transaction` | Ops, inverses, transform, session, wire protocol | Layout, transport | New; protocol and session pattern from calc |
| `-import` | PresentationML to model, report, retained source | Export | New |
| `-export` | Applying dirty model values to the part tree, and package bookkeeping. The byte repackager itself is `Package::rewrite` | Parsing | New |
| `-render` | PNG (tiny-skia) and PDF backends | Layout decisions | New; backend pattern from calc |
| `-sdk` | Host-facing session API and format dispatch | Internals exposure | New |
| `-wasm` | `wasm-bindgen` transport over the SDK | Logic | New |

Not created: `-selection` (lives in `-transaction`, since presence must map it through
ops), and `-odp` until Phase 10. No crate exists without a first consumer, except skeletons created to let parallel tiers start, which must gain one when their tier lands.

```
server/    casual-slide-collab-server, -host, -wopi, -mcp, -secrets
webapp/    editor; ribbon, command registry and tokens ported from opencalc
desktop/   Tauri, Phase 10
sdk/       @openslide/deck and friends, Phase 10
schemas/   vendored standards, see schemas/README.md
tools/     spec-checklist, fidelity, benchmark, check-* gates
fixtures/  checksummed corpus
fuzz/      separate workspace
```

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
bundle cost. No charting, layout or canvas crate.

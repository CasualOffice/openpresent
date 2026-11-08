# 08 — ADR register

ADR ids here are this repository's own; OpenCalc's are always cited as "its ADR-n".
ADR-001 to ADR-015 are Accepted, on the owner's go-ahead and delegation of 2026-11-07; ADR-016 and ADR-019 are Accepted under owner instruction 43; ADR-017 is a Phase 2 proposal; The number between them is skipped, so that OpenCalc's, cited in 02 and 42 as its ADR-018, is never read as ours. ADR-004 and ADR-005 followed `SPK-01`. **Proposed unless marked.** Status moves to Accepted when the owner agrees, or when
a spike named in the row has confirmed it.

**Write an ADR when a decision touches:** public API; a crate boundary or the DAG; a
serialization format; the op set or undo; the model's representation; what enters
the WASM bundle; the fidelity contract; a new dependency in the engine.

| ADR | Decision | Status | Reasoning in |
| --- | --- | --- | --- |
| ADR-001 | Crates `casual-slide-*`; product name pending `DEC-01` | Accepted | — |
| ADR-002 | Follow OpenCalc's layered engine and host split | Accepted | [02](02-ARCHITECTURE.md) |
| ADR-003 | Separate repo; copy shared crates with provenance (`DEC-02`) | Accepted | [19](19-WORKSPACE-SCAFFOLD-DESIGN.md) |
| ADR-004 | Text shaping and layout live in the engine and in WASM, reversing its ADR-018 here | Accepted | [64](64-SPK-01-TEXT-COST.md) |
| ADR-005 | Canvas2D backend paints glyph paths from the display list | Accepted, with conditions in `TXT-04`, `TXT-06` | [64](64-SPK-01-TEXT-COST.md) |
| ADR-006 | Inheritance resolved at read time; never flattened | Accepted | [22](22-NORMALIZED-SCHEMA.md) |
| ADR-007 | Fidelity contract: part-level verbatim retention | Accepted | [22](22-NORMALIZED-SCHEMA.md) |
| ADR-008 | Op set is per-element and per-property; stable object ids; values on the wire | Accepted | [24](24-OPERATIONS-AND-COLLABORATION.md) |
| ADR-009 | Text edits are character OT within a text body, not whole-body replace | Accepted | [24](24-OPERATIONS-AND-COLLABORATION.md) |
| ADR-010 | Fork the collaboration server behind an engine trait | Accepted | [24](24-OPERATIONS-AND-COLLABORATION.md) |
| ADR-011 | Animations compile to a deterministic sampled timeline in the engine | Accepted | [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) |
| ADR-012 | Shell target: PowerPoint for the web; replace only what Microsoft owns | Accepted | [47](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md) |
| ADR-013 | Edited parts are patched as a byte-faithful XML tree held by export. The model holds plain values; save writes dirty values to their nodes, and never re-serializes a part | Accepted | [28](28-PACKAGE-AND-PART-GRAPH.md) |
| ADR-015 | Recover before refusing: malformed input gets a best reading, and hostile constructs are neutralized, not obeyed. A host may choose strict. Damage is repaired into one clean image in memory, so readers and the writer have no second path. `flate2` joins as a direct dependency of `casual-slide-package`: `zip` merges duplicate names and refuses damage, and `flate2` with `zlib-rs` is its own deflate backend, already in the lockfile, pure Rust, and Apache-2.0 or MIT | Accepted, owner instruction 24 | [29](29-RECOVERY.md) |
| ADR-016 | Fonts that ship, and the font seam for faces we may not ship. **OpenCalc's set** (`DEC-16`): Roboto, Caladea, Carlito, Liberation Sans, Serif and Mono, and the CJK slices a host registers; no Microsoft font; a host supplies others with `register_face`. In `casual-slide-text`, copied from OpenCalc, with **every face on by default on every host**, because the engine breaks lines and a host without the metric-compatible faces would wrap differently (cost: `T2D1-02`). Its engine dependencies: `skrifa` and `brotli-decompressor` as OpenCalc's `-text`, `harfrust` as OpenCalc's `-render` but not behind a feature, and `unicode-linebreak` for UAX #14, which OpenCalc lacks (Apache-2.0, no dependencies). Until `-layout` exists, `-text` also reads a model text body through `-style` (`T2D4-02`) | Accepted, owner instruction 43 | [06](06-ROADMAP-AND-DELIVERY.md) §Phase 2 plan, [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) |
| ADR-017 | Rendering reference and fidelity method until `PV-01`: LibreOffice on GitHub's runner, labelled; PowerPoint PDF exports where the owner supplies them | Proposed, waits on `DEC-17` | [06](06-ROADMAP-AND-DELIVERY.md) §Phase 2 plan, [51](51-SPEC-CHECKLIST-AND-FIDELITY.md) |
| ADR-019 | Render dependencies: tiny-skia, harfrust, skrifa, `image`, resvg; our own PDF writer and EMF reader. Each joins with its first consumer: `casual-slide-render` takes tiny-skia 0.11 (PNG) and `image` 0.25 with only its `jpeg` feature | Accepted, owner instruction 43 (`DEC-18` as recommended) | [06](06-ROADMAP-AND-DELIVERY.md) §Phase 2 plan, [19](19-WORKSPACE-SCAFFOLD-DESIGN.md) |
| ADR-020 | Collaboration's seam and wire (`VS-09`): `casual-slide-transaction::collab` defines `Engine` (wire op, transform) and `Replica` (apply, snapshot) and holds OpenCalc's client and server sessions made generic over them; the server sees only `Engine` and never reads a deck. The engine takes `serde` and `serde_json` for the wire, as OpenCalc's transaction crate does. Stop and wait, one chunk in flight, since chained pipelining diverged (`COL-02`) | Accepted, owner instruction 42 (`VS-09` brief: copy OpenCalc's server behind a trait seam) | [24](24-OPERATIONS-AND-COLLABORATION.md), [19](19-WORKSPACE-SCAFFOLD-DESIGN.md) |

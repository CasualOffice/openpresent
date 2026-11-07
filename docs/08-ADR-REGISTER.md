# 08 — ADR register

ADR ids here are this repository's own; OpenCalc's are always cited as "its ADR-n".
All Accepted, on the owner's go-ahead and delegation of 2026-11-07. ADR-004 and ADR-005 followed `SPK-01`. **Proposed unless marked.** Status moves to Accepted when the owner agrees, or when
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
| ADR-015 | Recover before refusing: malformed input gets a best reading, and hostile constructs are neutralized, not obeyed. A host may choose strict | Accepted, owner instruction 24 | [29](29-RECOVERY.md) |

# Provenance

Copied from OpenCalc (`../opencalc`) at commit `b23c67c`, on 2026-11-08, under owner
instruction 42. Same owner, Apache-2.0. The fonts keep their own licences: see
`fonts/README.md`, `fonts/LICENSES/` and the repository's `NOTICE`.

| Here | From OpenCalc | Changes |
| --- | --- | --- |
| `fonts/` | `crates/casual-calc-text/fonts/` | None to the font files or licence texts. `README.md` gains a note on where it now lives |
| `src/faces.rs`, `src/substitution.rs`, `src/cjk.rs`, `src/woff2.rs`, `src/tests.rs`, `src/cjk_tests.rs` | `crates/casual-calc-text/src/` | None |
| `src/lib.rs` | `crates/casual-calc-text/src/lib.rs` | Crate doc rewritten for slides; new modules declared and re-exported. `advance_width` and `char_advance` unchanged |
| `src/shape.rs` | `crates/casual-calc-render/src/shape.rs` | Always compiled, not behind a `shaping` feature; each glyph carries its cluster; `crate::fonts` renamed `crate::faces` |
| `Cargo.toml` | `crates/casual-calc-text/Cargo.toml` | Crate renamed; `harfrust`, `unicode-linebreak`, `-model` and `-style` added; `all-fonts` on by default (ADR-016) |

New here, with no OpenCalc counterpart: `src/breaks.rs` (UAX #14 break
opportunities), `src/layout.rs` and `src/layout/tests.rs` (paragraph and body layout to
glyph runs), `src/body.rs` (a model text body through the style chain to that layout),
`src/outline.rs` (face keys and glyph outlines; its pen follows OpenCalc's `GlyphPen`
in `casual-calc-render/src/lib.rs`), and `tests/corpus.rs`.
OpenCalc has no line breaker: a cell does not wrap as a text body does.

Tracker ids, ADR numbers and `docs/` links in the copied modules refer to OpenCalc's
documents unless they exist in this repository; the gates skip this crate's comments
for that reason (`tools/check-doc-references.py`, `tools/check-parser-limits.py`).
Comments in the copied modules still speak of cells, workbooks and Excel; they are
OpenCalc's reasoning and are kept as written so a later diff against OpenCalc stays
small.

# Provenance

Copied from `crates/casual-calc-package` in OpenCalc (`../opencalc`) at commit
`b23c67c`, on 2026-11-07. Same owner, Apache-2.0.

Local changes: crate renamed; `repository` key removed.

Tracker ids (such as `XL-FEAT-140`) and `docs/` links in comments refer to
OpenCalc's documents unless they exist in this repository.

Divergence from OpenCalc:

- Crate doc comment now names `.pptx` as the consumer.
- Duplicate and case-equivalent part names are refused (`CS-PKG-0007`).
- OPC part-name grammar enforced: empty and dot-ending segments, control
  characters, and percent-encoded dots and separators are refused.
- `read_part` no longer sizes its allocation from the declared size.

- Added `directory.rs` (raw central-directory walk), a per-entry ratio check, `read_part_capped`, `rewrite`, `source_bytes`, `find_part`, and codes CS-PKG-0008 and 0009.

The guards above are defects OpenCalc's copy also has at `b23c67c`, reported to the owner rather than changed there (`SEC-011`).

# 20 — Error code registry

Stable, never reused. **The contract:** messages and findings carry part names, relationship ids and counts, never document text. Tested for every code below except import's (`SEC-015`, `no_message_or_finding_carries_document_text`). `CS-OPC-0004` and `CS-MCE-0001` no longer carry the content type or namespace they refused; the package's content types and `resolve_compatibility` have them.

| Code | Meaning | | Code | Meaning |
| --- | --- | --- | --- | --- |
| CS-PKG-0001 | Input too large | | CS-XML-0001 | Too many elements |
| CS-PKG-0002 | Too many entries | | CS-XML-0002 | Too deep |
| CS-PKG-0003 | Expansion too large | | CS-XML-0003 | Too many attributes |
| CS-PKG-0004 | Unsafe or long path | | CS-XML-0004 | Malformed, or not namespace-well-formed |
| CS-PKG-0005 | Not a package | | CS-XML-0005 | DOCTYPE refused |
| CS-PKG-0006 | Part not found | | CS-XML-0006 | XML part too large |
| CS-PKG-0007 | Duplicate part | | CS-OPC-0001 | Required part missing |
| CS-PKG-0008 | Rewrite failed | | CS-OPC-0002 | Too many relationships |
| CS-PKG-0009 | Part over reader's cap | | CS-OPC-0003 | Too many content types |
| | | | CS-OPC-0004 | Not a presentation |
| | | | CS-OPC-0005 | Compound file |
| | | | CS-MCE-0001 | A must-understand namespace is not understood |

Findings, which do not stop a deck opening: CS-FND-0001 external content; 0002 unsafe link; 0003 target escapes package; 0004 dangling target; 0005 orphan part; 0006 undeclared content type; 0007 unknown relationship type; 0008 macro project; 0009 signature; 0010 embedded object; 0011 target or entry name that decodes to traversal or to no valid name; 0012 markup compatibility non-conformance.

Colour and theme import findings: CS-CLR-0001, CS-CLR-0002, CS-CLR-0003, CS-CLR-0004, CS-CLR-0005, CS-CLR-0006, CS-CLR-0007, CS-CLR-0008, CS-CLR-0009, CS-CLR-0010. Defined in `crates/casual-slide-import/src/color.rs` and `theme.rs`.

Drawing property findings, on a slide part's report: CS-DRW-0001 a list over a `DrawingLimits` bound or a `casual-slide-geometry` cap, whole property left out and kept in the tree; CS-DRW-0002 a dash stop of zero or less, kept (`MS-OI29500` 2.1.1292); CS-DRW-0003 `effectDag` beside a fill, both kept (2.1.1295); CS-DRW-0004 custom geometry the geometry engine's compile repaired, or an adjust naming none of the preset's, kept as written; CS-DRW-0005 a repeated container name, or an `effect/@ref` naming nothing (2.1.1291, 2.1.1294), kept; CS-DRW-0006 a percentage with more precision than the model holds, rounded in the model and kept in the tree. Defined in `crates/casual-slide-import/src/drawing.rs`. A colour in a shape's properties reports under these and `CS-IMF`, not `CS-CLR`, as a slide part has one report. Export refusals for these properties use `CS-EXP-0001` (a character XML cannot carry), `CS-EXP-0002` (an extension with no known `uri`, or properties on a kind of node that has none), `CS-EXP-0003` (a relationship the part does not have), and `CS-EXP-0007` (a value the reader would not believe, by the read-back), and `CS-SLE-0002` as for any node in alternate content.

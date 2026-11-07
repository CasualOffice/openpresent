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

Text bodies, refused on crossing a bound in `TextLimits` ([21](21-PARSER-LIMITS.md)): CS-TXT-0001 paragraphs in a text body; CS-TXT-0002 runs in a paragraph; CS-TXT-0003 paragraphs in a part; CS-TXT-0004 runs in a part; CS-TXT-0005 tab stops in a list. Defined in `crates/casual-slide-import/src/properties.rs`. Export refuses an edit it does not write as CS-EXP-0009, defined in `crates/casual-slide-export/src/properties.rs`, and nothing is saved. Text content (`a:t`, `@typeface`, `@char`, `@tooltip` and the rest) never reaches a message or a finding: both carry element and attribute names only.

Tables, refused on crossing a bound in `TableLimits` ([21](21-PARSER-LIMITS.md)): CS-TBL-0001 rows in a table; CS-TBL-0002 columns in a grid; CS-TBL-0003 cells in a table; CS-TBL-0004 table cells in a part; CS-TBL-0005 styles in the `tableStyles` part; CS-TBL-0006 headers in a cell. Table findings, on a slide part's report, none of which stops a deck opening and all of which stay in the tree: CS-TBF-0001 a row whose cells are not one per grid column (`MS-OI29500` 2.1.1424 c); CS-TBF-0002 a `gridSpan` or `rowSpan` under 1 or leaving the table (2.1.1426 a, b), left out of the model; CS-TBF-0003 a cell in a merge without the flags saying so, or merges overlapping (2.1.1424 d, 2.1.1426 c); CS-TBF-0004 `hMerge` or `vMerge` on a cell no merge covers (2.1.1426 c); CS-TBF-0005 a covered cell whose spans disagree with its merge's (2.1.1424 d); CS-TBF-0006 no rows or columns, or more than Office's 1000 (2.1.1424 a, b; 2.1.1425); CS-TBF-0007 a negative column width, or a row height negative or over `0x3FFFFFFF` (2.1.1427 a); CS-TBF-0008 a `tableStyleId` or `@def` naming no style in the `tableStyles` part (BEH-29), which may be a built-in style (2.1.1423). Defined in `crates/casual-slide-import/src/table.rs`. Export refuses a table, row, cell or grid column added or removed as CS-EXP-0009, and a span the reader would not believe as CS-EXP-0007, with nothing saved.

Colour and theme import findings: CS-CLR-0001, CS-CLR-0002, CS-CLR-0003, CS-CLR-0004, CS-CLR-0005, CS-CLR-0006, CS-CLR-0007, CS-CLR-0008, CS-CLR-0009, CS-CLR-0010. Defined in `crates/casual-slide-import/src/color.rs` and `theme.rs`.

Drawing property findings, on a slide part's report: CS-DRW-0001 a list over a `DrawingLimits` bound or a `casual-slide-geometry` cap, whole property left out and kept in the tree; CS-DRW-0002 a dash stop of zero or less, kept (`MS-OI29500` 2.1.1292); CS-DRW-0003 `effectDag` beside a fill, both kept (2.1.1295); CS-DRW-0004 custom geometry the geometry engine's compile repaired, or an adjust naming none of the preset's, kept as written; CS-DRW-0005 a repeated container name, or an `effect/@ref` naming nothing (2.1.1291, 2.1.1294), kept; CS-DRW-0006 a percentage with more precision than the model holds, rounded in the model and kept in the tree. Defined in `crates/casual-slide-import/src/drawing.rs`. A colour in a shape's properties reports under these and `CS-IMF`, not `CS-CLR`, as a slide part has one report. Export refusals for these properties use `CS-EXP-0001` (a character XML cannot carry), `CS-EXP-0002` (an extension with no known `uri`, or properties on a kind of node that has none), `CS-EXP-0003` (a relationship the part does not have), and `CS-EXP-0007` (a value the reader would not believe, by the read-back), and `CS-SLE-0002` as for any node in alternate content.

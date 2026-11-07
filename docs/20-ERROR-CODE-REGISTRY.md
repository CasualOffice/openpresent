# 20 — Error code registry

Stable, never reused. **The contract:** messages and findings carry part names, relationship ids and counts, never document text. A relationship id is only repeated if it is an `xsd:NCName` of at most 64 bytes, `Relationship::reportable_id` (`DOC-009`): a part chooses its ids, so another form is left out of the finding and still works as an id. Tested for every code below: `no_message_or_finding_carries_document_text` (`SEC-015`) for the package and XML layers, `no_error_or_finding_carries_document_text` (`SEC-018`) for import. An element or attribute name in a finding is one the schemas have, from a table generated from the checklist; any other reads `(unknown)`, since a part chooses its names. `CS-OPC-0004` and `CS-MCE-0001` no longer carry the content type or namespace they refused; the package's content types and `resolve_compatibility` have them.

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

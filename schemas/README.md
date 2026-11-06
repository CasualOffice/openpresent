# Vendored schemas

The authoritative inventory for [the spec checklist](../docs/51-SPEC-CHECKLIST-AND-FIDELITY.md).
Downloaded 2026-11-07 directly from the publishers. Rights position recorded below; **a formal rights
review is still outstanding** (`DEC-07`).

| Directory | What | Source | Archive SHA-256 |
| --- | --- | --- | --- |
| `ooxml/` | PresentationML, DrawingML, charts, diagrams, math, VML and properties. **Transitional**, which is what real decks use | `https://ecma-international.org/wp-content/uploads/ECMA-376-4_5th_edition_december_2016.zip` | `bd25da1109f73762356596918bf5ff8b74a1331642dba5f1c1d1dfc6bed34ecd` |
| `ooxml-strict/` | The same vocabularies in **Strict**: the 14 files reachable from what a presentation's parts use. `shared-math.xsd` is left out, since nothing here reaches it and it imports WordprocessingML. Used by `tools/f4/validate.py`, and `the_strict_map_agrees_with_the_vendored_schemas` holds the namespace map to them | `https://ecma-international.org/wp-content/uploads/ECMA-376-1_5th_edition_december_2016.zip`, inner `OfficeOpenXML-XMLSchema-Strict.zip` | `9d0bcad9cf06054785b03762fcfadbf6bab7e54a5f9d69434e34b7fd464d4129` |
| `geometry/` | Preset shape and text warp definitions | `https://ecma-international.org/wp-content/uploads/ECMA-376-1_5th_edition_december_2016.zip` | `9d0bcad9cf06054785b03762fcfadbf6bab7e54a5f9d69434e34b7fd464d4129` |
| `opc/` | Content types, relationships, core properties, signatures | `https://ecma-international.org/wp-content/uploads/ECMA-376-2_5th_edition_december_2021.zip` | `1d489dc491168ea1f9e9a59063acc8dd5f02b4ad1d21aa7ec19ba9a58d020c70` |
| `ms-extensions/` | 62 schemas from Appendix A of [MS-PPTX](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-pptx/) and [MS-ODRAWXML](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-odrawxml/). `index.json` maps file to namespace | Extracted from the documentation pages | n/a: extracted from HTML, then parsed to confirm well-formedness |

Word and spreadsheet schemas are not vendored; they describe other formats.

## Rights, as read on 2026-11-07

**Not legal advice, and not a substitute for the review.**

- **Ecma:** the [copyright notice](https://ecma-international.org/policies/by-ipr/ecma-text-copyright-policy/) permits copying and
  distribution with the notice retained, and derivative works "by making use of this specification in standard conformant products by
  implementing (e.g. by copy and paste wholly or partly) the functionality therein". It forbids modifying the document's content.
  Vendored files are unmodified.
- **Microsoft:** the Open Specifications notice on those pages permits distributing "with or without modification, any schemas" in an
  implementation. It also says **Microsoft has patents that might cover implementations** and points to its patent promises. That is the
  part most in need of review.

## Known defects

- `geometry/presetShapeDefinitions.xml` defines `upDownArrow` twice and never defines `upArrow`, which `ST_ShapeType` requires (`GEO-01`).
- MS-PPTX and MS-ODRAWXML schemas were extracted from web pages, and could carry extraction faults that still parse. Compare against a published `.xsd` if one appears.

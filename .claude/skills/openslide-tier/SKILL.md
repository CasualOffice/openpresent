---
name: openslide-tier
description: Open or close a Phase 1 or Phase 2 tier of OpenSlide. Use when starting a tier such as 1D, splitting it into tasks, or checking whether it can close.
---
**Opening**
1. Confirm the dependencies in `docs/06` are closed or waived.
2. Get the scope: `grep -P '\tAREA\t' docs/spec/checklist.tsv` for each area, and the matching `BEH-*` rows in `docs/51`.
3. Read the relevant `MS-OI29500` notes (`SPEC-02`) and record departures from the standard.
4. Split into tasks of at most days, as rows in `docs/14`, each with an acceptance test. Plan fixtures, including hostile ones and at least one per row.

**Closing**: check every item of the per-tier gate in `docs/06`, by running it.
- Every in-scope row is `y` or `r`, or has a named reason. `tools/spec-audit` agrees, and a `y` row has a fixture.
- Unedited save byte-identical; edits change only named nodes; import → export → import is a fixed point; edited XML validates with `xmllint` against `schemas/`.
- Everything not modelled appears in the report. Limits and fuzz targets exist.

Record the evidence in the tier row. A LibreOffice result stays labelled as a second opinion. Phase closure belongs to the owner; propose it with evidence.

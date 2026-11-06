# 01 — Objectives and requirements

## Outcome

Someone who uses PowerPoint for the web opens a `.pptx` in OpenSlide and carries
on working without being taught anything. The deck looks right, edits like
PowerPoint, presents like PowerPoint, and saves back to a file PowerPoint opens
with nothing lost.

## The reference product

**PowerPoint for the web sets the UI style and flows; desktop sets the feature bar
and file semantics** (working position, `DEC-06`; [48](48-POWERPOINT-FEATURE-STUDY.md)).
This is an enterprise product and not an MVP ([10](10-ENTERPRISE-REQUIREMENTS.md)). Fidelity
is its most critical property ([51](51-SPEC-CHECKLIST-AND-FIDELITY.md)).

That gives three support levels, stated per feature in
[47](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md):

| Level | Meaning | Example |
| --- | --- | --- |
| **Edit** | Modelled, rendered, editable, written back | Shapes, text, tables |
| **Render** | Modelled or resolved, drawn, retained on save | SmartArt through its stored drawing |
| **Retain** | Not drawn, kept verbatim, named in the compatibility report | VBA project, OLE payloads |

## In scope

- `.pptx`, `.ppsx`, `.potx`, `.pptm` (VBA retained, never run), and later `.odp`.
- The full editing surface: masters, layouts, themes, shapes, text, tables,
  charts, images, media, ink, sections, notes, comments, transitions, animations.
- Slide show with presenter view; PDF export; print.
- Co-editing with presence; embeddable SDK; WOPI; desktop shell; MCP access —
  the same host targets OpenCalc has.

## Out of scope until the owner says otherwise

- **Microsoft's cloud-backed AI**: Designer and Copilot have no local equivalent.
  `DEC-05` decides whether to build our own.
- **Stock image and icon libraries** that are Microsoft's licensed content. A
  host seam supplies a library; we ship an open icon set.
- **Running macros.** Never.

## Quality targets

Proposed, not measured; the Phase 1 spikes confirm or change them
([42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) §Budgets).

- A corpus deck renders within a per-fixture pixel threshold of its reference.
- An opened deck saved unedited comes back byte for byte.
- An edited deck leaves every untouched part byte for byte.
- Drag, resize and typing hold 60 fps on a 200-shape slide; opening a 500-slide
  deck paints the first slide before the rest is parsed.
- No crash, hang or unbounded allocation on any input.

# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/), as OpenCalc's does. Semantic versioning
applies once a published line begins; until then everything is under **Unreleased**,
grouped by date. Each entry cites the tracker id in
[docs/14](docs/14-EXECUTION-TRACKER.md) that drove it.

Nothing has been released. Release tags and their dry-run default are described in
[docs/15](docs/15-CI-AND-RELEASE-GATES.md).

## Unreleased

### 2026-11-07

**Added**

- Governance: `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `GOVERNANCE.md`,
  this changelog, and `NOTICE` with the Ecma notice and a third-party inventory (`GOV-01`).
- [docs/15](docs/15-CI-AND-RELEASE-GATES.md), CI and release gates, and
  [docs/21](docs/21-PARSER-LIMITS.md), the parser limits contract (`GOV-01`, `DOC-008`).
- Repository gates ported from OpenCalc, each with a seeded violation in a self-test:
  tracker row shape, doc references, crates never depending on a server (`CI-01`),
  fuzz lockfile, Conventional Commits. Post-merge and scheduled fuzz workflows (`GOV-01`).
- Microsoft's Open XML SDK validator in `tools/ooxml-sdk` (`F-006`).
- Real PowerPoint-written decks with recorded rights in `fixtures/real/`, and a runner over
  66 decks in `tools/real-decks` (`F-007`).
- Text cost and painting measurements (`SPK-01`); ADR-004 and ADR-005 accepted.
- Markup compatibility, part-name equivalence, the Strict namespace map, F4 schema
  validation and forced rewrite through the tree (`MCE-01`, `OPC-01`, `STRICT-01`,
  `F4-01`, `WR-01`).
- Document properties, the presentation part and view state: model, import, export
  (Tier 1B).
- Colour and theme: model, import, export, resolver (Tier 1C).
- Preset and custom geometry engine with all 187 shapes and 41 text warps (`GEO-03`).
- Spec audit, count check, fidelity pipeline and area decks (`SPEC-01`, `SPEC-03`,
  `SPK-02`).
- Part graph, hostile-input corpus, fuzz targets and CI (`F-003`, `F-004`, `F-005`).
- Byte-faithful XML tree (ADR-013).
- Foundation: design set, spec checklist of 8730 rows, and the package crate forked
  from OpenCalc with provenance (`DOC-001`, `SPEC-05`, `F-002`).
- Design documents: phases and tasks, owner instruction log, PowerPoint command
  inventory, UX study, competitor study, fit-gap, export and save design.

**Fixed**

- Defects in Ecma's preset shape definitions (`GEO-01`, `GEO-02`).
- Zip bomb dilution and XML read before the size check (`SEC-013`).

# 00 — OpenPresent documentation index

`docs/` is the source of truth for the design. Code follows docs. Numbers mirror
OpenCalc's where a document plays the same role, so the siblings read alike, and
are never reused.

Read in this order: 03 → 01 → 02 → 05 → 06 → 51 → 07. Then the pillar you are working on.

Docs are true as of `8011d28` (2026-11-09). Outside `docs/`: [README](../README.md) (what works today, how to run it), [AGENTS.md](../AGENTS.md) (the contract and current state), [CHANGELOG](../CHANGELOG.md) (what landed, by date), [webapp/README.md](../webapp/README.md) (the editor's files and how to serve it).

| Doc | What it is | Status |
| --- | --- | --- |
| [01-ORD](01-ORD.md) | What and why; scope; targets | Draft for owner |
| [02-ARCHITECTURE](02-ARCHITECTURE.md) | The shape; what is inherited from OpenCalc and what must differ | Draft for owner |
| [03-OWNER-INSTRUCTIONS](03-OWNER-INSTRUCTIONS.md) | **Everything the owner instructed. Binding, and first in authority** | Live |
| [04-MISTAKES-AND-RULES](04-MISTAKES-AND-RULES.md) | **Mistakes made, and the rule that stops each. Binds every session** | Live |
| [05-PHASES-AND-TASKS](05-PHASES-AND-TASKS.md) | **What a phase, tier and task are; how each opens and closes** | Draft for owner |
| [06-ROADMAP-AND-DELIVERY](06-ROADMAP-AND-DELIVERY.md) | The full roadmap: phases, tiers, exit gates | Draft for owner |
| [07-SECURITY-THREAT-MODEL](07-SECURITY-THREAT-MODEL.md) | Every threat, its control, its test | Live |
| [08-ADR-REGISTER](08-ADR-REGISTER.md) | Decisions | Draft for owner |
| [10-ENTERPRISE-REQUIREMENTS](10-ENTERPRISE-REQUIREMENTS.md) | The enterprise bar | Draft for owner |
| [14-EXECUTION-TRACKER](14-EXECUTION-TRACKER.md) | The only tracker | Live |
| [19-WORKSPACE-SCAFFOLD-DESIGN](19-WORKSPACE-SCAFFOLD-DESIGN.md) | Crates, dependency DAG, provenance | Draft for owner |
| [20-ERROR-CODE-REGISTRY](20-ERROR-CODE-REGISTRY.md) | Every diagnostic code | Live |
| [22-NORMALIZED-SCHEMA](22-NORMALIZED-SCHEMA.md) | The presentation model and fidelity contract | Draft for owner |
| [24-OPERATIONS-AND-COLLABORATION](24-OPERATIONS-AND-COLLABORATION.md) | Op set, undo, OT | Draft for owner |
| [28-PACKAGE-AND-PART-GRAPH](28-PACKAGE-AND-PART-GRAPH.md) | Tier 1A | Built; tier closed 2026-11-08 |
| [42-TEXT-AND-RENDERING-ARCHITECTURE](42-TEXT-AND-RENDERING-ARCHITECTURE.md) | Text engine, display list, canvas, slide show, budgets | Draft for owner |
| [47-POWERPOINT-SHELL-AND-FEATURE-MAP](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md) | The UI target and the feature inventory | Draft; **recalled, not yet measured** |
| [48-POWERPOINT-FEATURE-STUDY](48-POWERPOINT-FEATURE-STUDY.md) | What PowerPoint offers, from three views, with conflicts | Draft for owner |
| [51-SPEC-CHECKLIST-AND-FIDELITY](51-SPEC-CHECKLIST-AND-FIDELITY.md) | How fidelity is measured | Draft for owner |
| [spec/](spec/README.md) | **Every tag, property and enum value (8730 rows)**, generated; parts and web shortcuts, recorded by hand | Live |
| [49-POWERPOINT-COMMAND-INVENTORY](49-POWERPOINT-COMMAND-INVENTORY.md) | Every PowerPoint command, from Microsoft's control-ID workbook (2024) and public sources. Desktop strong; web thin | Live |
| [50-POWERPOINT-UX-STUDY](50-POWERPOINT-UX-STUDY.md) | How PowerPoint behaves, and 24 rules for our shell. From public sources; no live session | Live |
| [53-POWERPOINT-WEB-SHELL-TARGET](53-POWERPOINT-WEB-SHELL-TARGET.md) | The web editor's target, compact: every tab, group and command in PowerPoint's order; File page, status bar, right-click menus, Format Shape pane, keys (`UX-003`) | Live; from documented sources and recall, not measured |
| [52-COMPETITOR-STUDY](52-COMPETITOR-STUDY.md) | What other editors learned; 17 ranked fidelity problems as fixture candidates | Live |
| [96-POWERPOINT-FIT-GAP](96-POWERPOINT-FIT-GAP.md) | Every PowerPoint feature (352) with a disposition, phase and reason, and §Parity today: the audit of what the editor does (`FIT-02`, at `7539ea5`) | Live |
| [36-EXPORT-AND-SAVE](36-EXPORT-AND-SAVE.md) | The writer: 118 invariants, each with a repair-risk fixture; save-as grid; fallback policy. Tier 1N closed 2026-11-08; what is built and what is designed is marked per section | Draft for owner |
| [64-SPK-01-TEXT-COST](64-SPK-01-TEXT-COST.md) | Measurements behind ADR-004 and ADR-005: bundle, layout, frame time, appearance | Measured |
| [15-CI-AND-RELEASE-GATES](15-CI-AND-RELEASE-GATES.md) | Test levels, the CI job contract, repository gates, no-retry flake policy, gates per phase, tag-only releases with dry-run default | Live |
| [21-PARSER-LIMITS](21-PARSER-LIMITS.md) | Every parser limit the code holds, its default, its code and whether a test proves it; checked against the code | Live |
| [29-RECOVERY](29-RECOVERY.md) | **Opening and editing damaged decks, including what Microsoft refuses** | Accepted; Tier 1R closed 2026-11-08 |

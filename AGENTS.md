# AGENTS.md — Agent contract for OpenPresent

Read this in full before doing anything else. Owner instructions are in [docs/03](docs/03-OWNER-INSTRUCTIONS.md) and come first. Working rules, engineering standards and skills are in [CLAUDE.md](CLAUDE.md). The product is OpenPresent (`DEC-01`); crates use the prefix `casual-slide-`, which does not depend on it.

## Repository boundary

- **This repository is the target.** All work happens here.
- **`../opencalc` is the sister repo and is reference-only.** It is the
  architecture this project follows. Read it, copy from it with provenance
  ([19](docs/19-WORKSPACE-SCAFFOLD-DESIGN.md) §Provenance), **do not modify it.**

## Mission

An **enterprise-grade** presentation editor, not an MVP or a demo: a deterministic, embeddable, loss-aware
`.pptx` engine in Rust, running client-side in WebAssembly, under an editor that
works like PowerPoint for the web — same ribbon, flows, shortcuts and dialogs —
so a PowerPoint user needs no teaching.

**Fidelity is the most critical property, and import, export and modelling come first**
([06](docs/06-ROADMAP-AND-DELIVERY.md), [51](docs/51-SPEC-CHECKLIST-AND-FIDELITY.md)). Coverage is
tracked per tag and property in [docs/spec](docs/spec/README.md). Every file is hostile until
proven otherwise ([07](docs/07-SECURITY-THREAT-MODEL.md)).

Only what Microsoft owns is replaced: name, logo, colour scheme, icon artwork,
fonts ([47](docs/47-POWERPOINT-SHELL-AND-FEATURE-MAP.md)).

## The cycle

Every task, none of it optional:

    design → execute → check → find issues → fix → push, with tests

- **Check** means running the thing, not re-reading it.
- **Find issues** asks what would have to be true for this to be wrong, then
  goes and looks.
- **With tests** means a test seen to fail without the fix.
- **Discuss and finalize** substantial designs before implementing. An ADR
  trigger ([08](docs/08-ADR-REGISTER.md)) needs an Accepted ADR first.
- **Track everything** in [14](docs/14-EXECUTION-TRACKER.md). No untracked work.

## Priorities (the earlier wins)

1. **Correctness and safety** — never corrupt a deck.
2. **Determinism** — same input and version ⇒ same model, layout, display list, bytes.
3. **Security and resource bounds** — every parser bounded; no macro execution;
   no automatic network fetches (linked images and media included).
4. **Fidelity** — preserve intent and unknown data; measured, not asserted.
5. **Performance** — designed-in targets ([42](docs/42-TEXT-AND-RENDERING-ARCHITECTURE.md)).
6. **API stability** — public surfaces narrower than internals.
7. **UX** — PowerPoint for the web is the specification.
8. **Maintainability.**

## Design rules

- **Mutation only through operations.** Every edit is an op returning its inverse.
- **The engine owns layout.** Line breaks, caret positions and autofit come from
  the engine, never from the browser, so collaborators, export and PNG agree.
- **Inheritance is resolved at read time, never flattened into the model.**
- **The host owns policy.** Fonts, I/O, network, persistence, transport.
- **No DOM or canvas as source of truth.** Views project the display list.
- **No silent data loss.** Anything unmodelled is retained verbatim or named in
  the compatibility report. The word "lossless" is banned unless the exact
  fidelity dimension is named.
- **A verb we lack is left off the ribbon**, not drawn dead.
- **A document stating a contract the code does not keep is a tracker row**, not
  a doc edit.
- **Report outcomes faithfully.** Preserved-but-not-modelled and
  parsed-but-not-rendered are said exactly that way.

## Current state

Docs are true as of `8011d28` (2026-11-09).

**Phases 0 and 1 are closed** (Phase 1 signed off 2026-11-08, instruction 53) ([06](docs/06-ROADMAP-AND-DELIVERY.md)). Every Phase 1 tier, 1A to 1N and 1R, is closed. Of 8,739 checklist rows, 7,843 are marked `y` or `r` in model, import and export, 469 are `n/a`, and the other 427 each have a named reason or are accounted for by their members; `tools/spec-audit --check` passes. What each tier carried out is in [14](docs/14-EXECUTION-TRACKER.md) §Tier gates. Every registered ADR from ADR-001 to ADR-016, ADR-019 and ADR-020 are Accepted; ADR-017 is proposed, waiting on `DEC-17`.

Phase 2 is open under instruction 43's waiver; its tiers are the render backlog. Work runs as **vertical slices** (instruction 44, [06](docs/06-ROADMAP-AND-DELIVERY.md) §After Phase 1): slices 1 to 12 are built and merged (viewer, render, desktop, save and export, edit text, move and resize, present, build slides, co-editing with `server/casual-slide-collab-server`, animate, tables and charts, design); **none is closed**, each row naming what is left, most often a CI browser run; slice 13 (hosts) has not started. Beside them: PowerPoint for the web's shell (`UX-003`), the 187-shape gallery (`SHAPE-01`), the File tab and autosave (`FILE-01`, `SAVE-01`), side panes with comments, notes and the Selection Pane (`UX-011`), and the site.

Measured: gallery agreement with LibreOffice 26.2.6.3 on CI at `1ba6443`, 5.4% of pixels differ on mean over 111 slides (94.6% agree) ([51](docs/51-SPEC-CHECKLIST-AND-FIDELITY.md)); this is not a PowerPoint measure, and nothing claims a PowerPoint match before `PV-01`. Parity audit, kept current at `PAR-01`: 48 of 352 PowerPoint features work (13.6%), from 35 (9.9%) at the first audit ([96](docs/96-POWERPOINT-FIT-GAP.md) §Parity today). The site and editor are live at https://casualoffice.github.io/openpresent/ (`SITE-01`, deployed by `pages.yml`). The `collab` workflow is red on `main` (`CI-23`).
